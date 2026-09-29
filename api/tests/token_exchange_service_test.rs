//! The RFC 8693 exchange against a real database: tokens are signed with the
//! realm key, persisted, and verified again. The token endpoint does not route
//! the grant yet (#1054), so the tests call the application service directly.

#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::TestServer;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::Args,
    };
    use ferriskey_core::{
        application::{create_service, services::ApplicationService},
        domain::{
            authentication::token_exchange::{
                TokenExchangeError, TokenExchangeInput, TokenExchangeOutput,
            },
            common::{
                DatabaseConfig, FerriskeyConfig, entities::StartupConfig, ports::CoreService,
            },
        },
    };
    use serde_json::{Value, json};
    use sqlx::Executor;
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const MASTER_REALM: &str = "master";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const MASTER_ADMIN_USERNAME: &str = "admin";
    const MASTER_ADMIN_PASSWORD: &str = "admin";
    const REALM: &str = "exchange";
    const USERNAME: &str = "alice";
    const PASSWORD: &str = "Exchange-Passw0rd!";
    const ACCESS_TOKEN_URN: &str = "urn:ietf:params:oauth:token-type:access_token";

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        service: ApplicationService,
    }

    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    static CTX: std::sync::OnceLock<SharedContext> = std::sync::OnceLock::new();

    fn rt() -> &'static tokio::runtime::Runtime {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("build shared runtime")
        })
    }

    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(init_shared_ctx())),
            Err(_) => rt().block_on(init_shared_ctx()),
        })
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("token_exchange_service_test_{}", Uuid::new_v4().simple());

        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );
        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{}\"",
                schema
            )))
            .await
            .expect("create schema");

        let schema_url = format!(
            "postgres://{}:{}@{}:{}/{}?options=-c search_path={}",
            db_user,
            db_password,
            db_host,
            db_port,
            db_name,
            urlencoding::encode(&schema)
        );
        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let service = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: false,
            webapp_url: WEBAPP_URL.to_string(),
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema: schema.clone(),
            },
        })
        .await
        .expect("create service");

        service
            .initialize_application(StartupConfig {
                webapp_url: WEBAPP_URL.to_string(),
                master_realm_name: MASTER_REALM.to_string(),
                admin_username: MASTER_ADMIN_USERNAME.to_string(),
                admin_password: MASTER_ADMIN_PASSWORD.to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let state = AppState::new(Arc::new(Args::default()), service.clone());
        let app = router(state).expect("build router");

        let ctx = SharedContext {
            app: std::sync::Mutex::new(app),
            service,
        };

        seed(&ctx).await;

        ctx
    }

    async fn seed(ctx: &SharedContext) {
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
        let server = TestServer::new(app).expect("create test server");
        let master = password_token(
            &server,
            MASTER_REALM,
            "admin-cli",
            None,
            MASTER_ADMIN_USERNAME,
            MASTER_ADMIN_PASSWORD,
        )
        .await;

        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(&master))
            .json(&json!({ "name": REALM, "display_name": REALM }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());

        let response = server
            .post(&format!("/realms/{REALM}/users"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "username": USERNAME,
                "firstname": USERNAME,
                "lastname": "Exchange",
                "email": format!("{USERNAME}@exchange.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let user_id = response.json::<Value>()["data"]["id"]
            .as_str()
            .expect("user id")
            .to_string();

        let response = server
            .put(&format!("/realms/{REALM}/users/{user_id}/reset-password"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "value": PASSWORD,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
    }

    fn make_server() -> TestServer {
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("bearer header is valid ASCII")
    }

    async fn password_token(
        server: &TestServer,
        realm: &str,
        client_id: &str,
        client_secret: Option<&str>,
        username: &str,
        password: &str,
    ) -> String {
        let mut form = vec![
            ("grant_type", "password"),
            ("client_id", client_id),
            ("username", username),
            ("password", password),
            ("scope", "openid profile email"),
        ];
        if let Some(secret) = client_secret {
            form.push(("client_secret", secret));
        }

        let response = server
            .post(&format!("/realms/{realm}/protocol/openid-connect/token"))
            .form(&form)
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "token request for {username}@{realm} failed: {}",
            response.text()
        );
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    async fn master_token(server: &TestServer) -> String {
        password_token(
            server,
            MASTER_REALM,
            "admin-cli",
            None,
            MASTER_ADMIN_USERNAME,
            MASTER_ADMIN_PASSWORD,
        )
        .await
    }

    struct TestClient {
        uuid: String,
        client_id: String,
        secret: String,
    }

    async fn create_client(server: &TestServer, prefix: &str, token_exchange: bool) -> TestClient {
        let master = master_token(server).await;
        let client_id = format!("{prefix}-{}", Uuid::new_v4().simple());

        let response = server
            .post(&format!("/realms/{REALM}/clients"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "client_id": client_id,
                "name": prefix,
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": false,
                "direct_access_grants_enabled": true,
                "enabled": true,
                "oauth_device_code_grant_enabled": false,
                "token_exchange_enabled": token_exchange,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let body: Value = response.json();

        TestClient {
            uuid: body["id"].as_str().expect("client id").to_string(),
            client_id,
            secret: body["client_secret"]
                .as_str()
                .expect("creation hands back the plaintext secret")
                .to_string(),
        }
    }

    async fn create_policy(
        server: &TestServer,
        client: &TestClient,
        audience: &TestClient,
        allowed_scopes: &[&str],
    ) {
        let master = master_token(server).await;
        let response = server
            .post(&format!(
                "/realms/{REALM}/clients/{}/token-exchange-policies",
                client.uuid
            ))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "target_audience": audience.client_id,
                "allowed_scopes": allowed_scopes,
                "allow_impersonation": true,
                "allow_delegation": false,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
    }

    async fn subject_token_for(server: &TestServer, client: &TestClient) -> String {
        password_token(
            server,
            REALM,
            &client.client_id,
            Some(&client.secret),
            USERNAME,
            PASSWORD,
        )
        .await
    }

    async fn exchange(
        client: &TestClient,
        subject_token: &str,
        scope: Option<&str>,
        audience: Option<&str>,
    ) -> Result<TokenExchangeOutput, TokenExchangeError> {
        shared_ctx()
            .service
            .exchange_subject_token(
                REALM.to_string(),
                client.client_id.clone(),
                Some(client.secret.clone()),
                TokenExchangeInput {
                    subject_token: subject_token.to_string(),
                    subject_token_type: ACCESS_TOKEN_URN.to_string(),
                    requested_token_type: None,
                    audience: audience.map(str::to_string),
                    resource: None,
                    scope: scope.map(str::to_string),
                },
            )
            .await
    }

    fn claims(token: &str) -> Value {
        let payload = token.split('.').nth(1).expect("a JWT has a payload");
        let bytes = URL_SAFE_NO_PAD
            .decode(payload)
            .expect("the payload is base64url");
        serde_json::from_slice(&bytes).expect("the payload is JSON")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_service_test -- --ignored"]
    fn a_client_narrows_its_own_token_and_the_result_verifies() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let subject = claims(&subject_token);

            let output = exchange(&gateway, &subject_token, Some("profile"), None)
                .await
                .expect("a client may narrow its own token");
            assert_eq!(output.scope.as_deref(), Some("profile"));

            let issued = claims(&output.access_token);
            assert_eq!(issued["sub"], subject["sub"]);
            assert_eq!(issued["sid"], subject["sid"]);
            assert_eq!(issued["aud"], subject["aud"]);
            assert_eq!(issued["azp"], json!(gateway.client_id));
            assert_eq!(issued["scope"], json!("profile"));
            assert!(issued["exp"].as_i64() <= subject["exp"].as_i64());
            assert_ne!(issued["jti"], subject["jti"]);

            // The issued token is signed with the realm key and persisted, so it
            // is itself a valid subject token.
            exchange(&gateway, &output.access_token, None, None)
                .await
                .expect("the exchanged token verifies like any access token");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_service_test -- --ignored"]
    fn an_audience_needs_a_policy_and_is_capped_by_it() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let orders = create_client(&server, "orders", false).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let refused = exchange(&gateway, &subject_token, None, Some(&orders.client_id)).await;
            assert_eq!(refused.err(), Some(TokenExchangeError::InvalidTarget));

            create_policy(&server, &gateway, &orders, &["profile"]).await;

            let output = exchange(&gateway, &subject_token, None, Some(&orders.client_id))
                .await
                .expect("the policy allows this audience");
            let issued = claims(&output.access_token);
            assert_eq!(issued["aud"], json!([orders.client_id]));
            assert_eq!(issued["scope"], json!("profile"));

            let above = exchange(
                &gateway,
                &subject_token,
                Some("email"),
                Some(&orders.client_id),
            )
            .await;
            assert_eq!(above.err(), Some(TokenExchangeError::InvalidScope));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_service_test -- --ignored"]
    fn a_client_without_the_flag_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", false).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let refused = exchange(&gateway, &subject_token, None, None).await;
            assert_eq!(refused.err(), Some(TokenExchangeError::UnauthorizedClient));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_service_test -- --ignored"]
    fn a_client_cannot_exchange_a_token_it_is_not_a_party_to() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let intruder = create_client(&server, "intruder", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let refused = exchange(&intruder, &subject_token, None, None).await;
            assert_eq!(refused.err(), Some(TokenExchangeError::UnauthorizedClient));
        });
    }
}
