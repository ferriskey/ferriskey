//! The RFC 8693 exchange against a real database: tokens are signed with the
//! realm key, persisted, and verified again. The service-level tests check the
//! exchange rules with typed errors; the `token_endpoint_*` tests check the
//! wire format of `POST .../protocol/openid-connect/token` (#1054).

#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::TestServer;
    use base64::{
        Engine,
        engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
    };
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

        let schema = format!("token_exchange_test_{}", Uuid::new_v4().simple());

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
        create_client_with(server, prefix, token_exchange, false).await
    }

    async fn create_client_with(
        server: &TestServer,
        prefix: &str,
        token_exchange: bool,
        service_account: bool,
    ) -> TestClient {
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
                "service_account_enabled": service_account,
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

    /// Creates a policy letting `client` target `audience`, and returns its id.
    async fn create_policy(
        server: &TestServer,
        client: &TestClient,
        audience: &TestClient,
        allowed_scopes: &[&str],
        allow_impersonation: bool,
        allow_delegation: bool,
    ) -> String {
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
                "allow_impersonation": allow_impersonation,
                "allow_delegation": allow_delegation,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        response.json::<Value>()["id"]
            .as_str()
            .expect("policy id")
            .to_string()
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
                    actor_token: None,
                    actor_token_type: None,
                    ip_address: None,
                    user_agent: None,
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
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

            // The protocol mappers run again for the narrowed scope: the email
            // mapper belongs to the `email` scope, so its claim is gone.
            assert_eq!(
                subject["email"],
                json!(format!("{USERNAME}@exchange.local"))
            );
            assert!(issued.get("email").is_none(), "{issued}");
            assert_eq!(issued["preferred_username"], json!(USERNAME));

            // The issued token is signed with the realm key and persisted, so it
            // is itself a valid subject token.
            exchange(&gateway, &output.access_token, None, None)
                .await
                .expect("the exchanged token verifies like any access token");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn an_audience_needs_a_policy_and_is_capped_by_it() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let orders = create_client(&server, "orders", false).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let refused = exchange(&gateway, &subject_token, None, Some(&orders.client_id)).await;
            assert_eq!(refused.err(), Some(TokenExchangeError::InvalidTarget));

            create_policy(&server, &gateway, &orders, &["profile"], true, false).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
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

    const TOKEN_EXCHANGE_GRANT: &str = "urn:ietf:params:oauth:grant-type:token-exchange";

    fn token_path() -> String {
        format!("/realms/{REALM}/protocol/openid-connect/token")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_answers_with_the_rfc8693_body_and_no_cookie() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let response = server
                .post(&token_path())
                .form(&[
                    ("grant_type", TOKEN_EXCHANGE_GRANT),
                    ("client_id", gateway.client_id.as_str()),
                    ("client_secret", gateway.secret.as_str()),
                    ("subject_token", subject_token.as_str()),
                    ("subject_token_type", ACCESS_TOKEN_URN),
                    ("scope", "profile"),
                ])
                .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert!(response.maybe_header("set-cookie").is_none());
            assert_eq!(response.header("cache-control"), "no-store");

            let body: Value = response.json();
            assert_eq!(body["issued_token_type"], json!(ACCESS_TOKEN_URN));
            assert_eq!(body["token_type"], json!("Bearer"));
            assert_eq!(body["scope"], json!("profile"));
            assert!(body["expires_in"].as_i64().is_some_and(|secs| secs > 0));
            assert!(body.get("refresh_token").is_none());
            assert!(body.get("id_token").is_none());
            let issued = claims(body["access_token"].as_str().expect("access_token"));
            assert_eq!(issued["azp"], json!(gateway.client_id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_accepts_basic_client_authentication() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let basic = format!(
                "Basic {}",
                STANDARD.encode(format!("{}:{}", gateway.client_id, gateway.secret))
            );

            let response = server
                .post(&token_path())
                .add_header(
                    "Authorization",
                    basic.parse::<HeaderValue>().expect("header"),
                )
                .form(&[
                    ("grant_type", TOKEN_EXCHANGE_GRANT),
                    ("subject_token", subject_token.as_str()),
                    ("subject_token_type", ACCESS_TOKEN_URN),
                ])
                .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_refuses_a_bad_secret_with_a_401_challenge() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let response = server
                .post(&token_path())
                .form(&[
                    ("grant_type", TOKEN_EXCHANGE_GRANT),
                    ("client_id", gateway.client_id.as_str()),
                    ("client_secret", "not-the-secret"),
                    ("subject_token", subject_token.as_str()),
                    ("subject_token_type", ACCESS_TOKEN_URN),
                ])
                .await;

            assert_eq!(response.status_code(), 401, "{}", response.text());
            assert_eq!(response.header("www-authenticate"), "Basic");
            assert_eq!(response.json::<Value>()["error"], json!("invalid_client"));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_errors_use_the_rfc6749_body() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let id_token_urn = "urn:ietf:params:oauth:token-type:id_token";

            let cases: [(&[(&str, &str)], &str); 4] = [
                (&[], "invalid_request"),
                (
                    &[
                        ("subject_token", subject_token.as_str()),
                        ("subject_token_type", id_token_urn),
                    ],
                    "unsupported_token_type",
                ),
                (
                    &[
                        ("subject_token", subject_token.as_str()),
                        ("subject_token_type", ACCESS_TOKEN_URN),
                        ("scope", "admin"),
                    ],
                    "invalid_scope",
                ),
                (
                    &[
                        ("subject_token", subject_token.as_str()),
                        ("subject_token_type", ACCESS_TOKEN_URN),
                        ("audience", "nobody"),
                    ],
                    "invalid_target",
                ),
            ];

            for (extra, code) in cases {
                let mut form = vec![
                    ("grant_type", TOKEN_EXCHANGE_GRANT),
                    ("client_id", gateway.client_id.as_str()),
                    ("client_secret", gateway.secret.as_str()),
                ];
                form.extend_from_slice(extra);

                let response = server.post(&token_path()).form(&form).await;
                assert_eq!(response.status_code(), 400, "{code}: {}", response.text());
                let body: Value = response.json();
                assert_eq!(body["error"], json!(code));
                assert!(body["error_description"].is_string());
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn the_openapi_document_describes_the_rfc8693_request_and_response() {
        rt().block_on(async {
            let server = make_server();
            let doc: Value = server.get("/api-docs/openapi.json").await.json();
            let schemas = &doc["components"]["schemas"];

            let request = &schemas["TokenRequestValidator"]["properties"];
            for field in [
                "subject_token",
                "subject_token_type",
                "requested_token_type",
                "audience",
                "resource",
                "actor_token",
                "actor_token_type",
            ] {
                assert!(
                    request.get(field).is_some(),
                    "TokenRequestValidator lacks {field}"
                );
            }

            let variants = schemas["TokenResponse"]["oneOf"].to_string();
            assert!(variants.contains("JwtToken"), "{variants}");
            assert!(variants.contains("TokenExchangeOutput"), "{variants}");
            assert!(schemas.get("TokenExchangeOutput").is_some());

            let unauthorized = schemas["TokenUnauthorizedResponse"]["oneOf"].to_string();
            assert!(unauthorized.contains("ApiErrorResponse"), "{unauthorized}");
            assert!(
                unauthorized.contains("OAuth2ErrorResponse"),
                "{unauthorized}"
            );
        });
    }

    /// The full password-grant response of `client` for the test user: access,
    /// refresh and ID tokens.
    async fn password_grant(server: &TestServer, client: &TestClient) -> Value {
        let response = server
            .post(&token_path())
            .form(&[
                ("grant_type", "password"),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("username", USERNAME),
                ("password", PASSWORD),
                ("scope", "openid profile email"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json()
    }

    async fn exchange_over_http(
        server: &TestServer,
        client: &TestClient,
        subject_token: &str,
    ) -> axum_test::TestResponse {
        server
            .post(&token_path())
            .form(&[
                ("grant_type", TOKEN_EXCHANGE_GRANT),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("subject_token", subject_token),
                ("subject_token_type", ACCESS_TOKEN_URN),
            ])
            .await
    }

    fn assert_oauth_error(response: &axum_test::TestResponse, status: u16, code: &str) {
        assert_eq!(
            response.status_code(),
            status,
            "{code}: {}",
            response.text()
        );
        assert_eq!(response.json::<Value>()["error"], json!(code));
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_refuses_a_public_client() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let master = master_token(&server).await;
            let public_id = format!("spa-{}", Uuid::new_v4().simple());
            let response = server
                .post(&format!("/realms/{REALM}/clients"))
                .add_header("Authorization", auth_header(&master))
                .json(&json!({
                    "client_id": public_id,
                    "name": "spa",
                    "client_type": "public",
                    "protocol": "openid-connect",
                    "public_client": true,
                    "service_account_enabled": false,
                    "direct_access_grants_enabled": false,
                    "enabled": true,
                    "oauth_device_code_grant_enabled": false,
                    "token_exchange_enabled": true,
                }))
                .await;
            assert_eq!(response.status_code(), 201, "{}", response.text());

            let response = server
                .post(&token_path())
                .form(&[
                    ("grant_type", TOKEN_EXCHANGE_GRANT),
                    ("client_id", public_id.as_str()),
                    ("subject_token", subject_token.as_str()),
                    ("subject_token_type", ACCESS_TOKEN_URN),
                ])
                .await;

            assert_oauth_error(&response, 401, "invalid_client");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_refuses_an_unusable_subject_token() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let tokens = password_grant(&server, &gateway).await;
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            let revoked = password_grant(&server, &gateway).await;
            let revoked_token = revoked["access_token"].as_str().expect("access_token");
            let response = server
                .post(&format!("/realms/{REALM}/protocol/openid-connect/revoke"))
                .form(&[
                    ("token", revoked_token),
                    ("token_type_hint", "access_token"),
                    ("client_id", gateway.client_id.as_str()),
                    ("client_secret", gateway.secret.as_str()),
                ])
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());

            for (label, subject_token) in [
                ("malformed", "not-a-jwt"),
                ("refresh token", refresh_token),
                ("revoked", revoked_token),
            ] {
                let response = exchange_over_http(&server, &gateway, subject_token).await;
                assert_eq!(response.status_code(), 400, "{label}: {}", response.text());
                assert_eq!(
                    response.json::<Value>()["error"],
                    json!("invalid_request"),
                    "{label}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn token_endpoint_refuses_a_subject_token_of_another_realm() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            // Signed by the master realm key, not this realm's.
            let foreign = master_token(&server).await;

            let response = exchange_over_http(&server, &gateway, &foreign).await;

            assert_oauth_error(&response, 400, "invalid_request");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn an_exchanged_token_dies_with_the_subject_session() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let tokens = password_grant(&server, &gateway).await;
            let subject_token = tokens["access_token"].as_str().expect("access_token");
            let id_token = tokens["id_token"].as_str().expect("id_token");

            let response = exchange_over_http(&server, &gateway, subject_token).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let exchanged = response.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();

            let introspect = || {
                server
                    .post(&format!(
                        "/realms/{REALM}/protocol/openid-connect/token/introspect"
                    ))
                    .form(&[
                        ("token", exchanged.as_str()),
                        ("client_id", gateway.client_id.as_str()),
                        ("client_secret", gateway.secret.as_str()),
                    ])
            };

            let before = introspect().await;
            assert_eq!(before.status_code(), 200, "{}", before.text());
            assert_eq!(before.json::<Value>()["active"], json!(true));

            let response = server
                .post(&format!("/realms/{REALM}/protocol/openid-connect/logout"))
                .form(&[("id_token_hint", id_token)])
                .await;
            assert!(
                response.status_code().is_success() || response.status_code().is_redirection(),
                "logout failed: {}",
                response.text()
            );

            let after = introspect().await;
            assert_eq!(after.status_code(), 200, "{}", after.text());
            assert_eq!(
                after.json::<Value>()["active"],
                json!(false),
                "the exchanged token shares the subject's sid and must die with it"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn discovery_advertises_every_live_grant() {
        rt().block_on(async {
            let server = make_server();
            let response = server
                .get(&format!("/realms/{REALM}/.well-known/openid-configuration"))
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let body: Value = response.json();

            let grants: Vec<&str> = body["grant_types_supported"]
                .as_array()
                .expect("grant_types_supported is an array")
                .iter()
                .filter_map(Value::as_str)
                .collect();
            for grant in [
                "authorization_code",
                "refresh_token",
                "client_credentials",
                "password",
                "urn:ietf:params:oauth:grant-type:device_code",
                TOKEN_EXCHANGE_GRANT,
            ] {
                assert!(grants.contains(&grant), "{grant} missing from {grants:?}");
            }

            let device_endpoint = body["device_authorization_endpoint"]
                .as_str()
                .expect("device_authorization_endpoint is advertised");
            assert!(
                device_endpoint.ends_with(&format!(
                    "/realms/{REALM}/protocol/openid-connect/auth/device"
                )),
                "{device_endpoint}"
            );
        });
    }

    async fn exchange_for_audience(
        server: &TestServer,
        client: &TestClient,
        subject_token: &str,
        audience: &TestClient,
    ) -> axum_test::TestResponse {
        server
            .post(&token_path())
            .form(&[
                ("grant_type", TOKEN_EXCHANGE_GRANT),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("subject_token", subject_token),
                ("subject_token_type", ACCESS_TOKEN_URN),
                ("audience", audience.client_id.as_str()),
            ])
            .await
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn a_policy_without_impersonation_refuses_a_plain_exchange() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let orders = create_client(&server, "orders", false).await;
            create_policy(&server, &gateway, &orders, &["profile"], false, false).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let response = exchange_for_audience(&server, &gateway, &subject_token, &orders).await;

            assert_oauth_error(&response, 400, "unauthorized_client");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn deleting_the_policy_stops_the_next_exchange() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client(&server, "gateway", true).await;
            let orders = create_client(&server, "orders", false).await;
            let policy_id =
                create_policy(&server, &gateway, &orders, &["profile"], true, false).await;
            let subject_token = subject_token_for(&server, &gateway).await;

            let allowed = exchange_for_audience(&server, &gateway, &subject_token, &orders).await;
            assert_eq!(allowed.status_code(), 200, "{}", allowed.text());

            let master = master_token(&server).await;
            let deleted = server
                .delete(&format!(
                    "/realms/{REALM}/clients/{}/token-exchange-policies/{policy_id}",
                    gateway.uuid
                ))
                .add_header("Authorization", auth_header(&master))
                .await;
            assert!(deleted.status_code().is_success(), "{}", deleted.text());

            let refused = exchange_for_audience(&server, &gateway, &subject_token, &orders).await;
            assert_oauth_error(&refused, 400, "invalid_target");
        });
    }

    /// A `client_credentials` token of `client`: the client itself as actor.
    async fn service_token(server: &TestServer, client: &TestClient) -> String {
        let response = server
            .post(&token_path())
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    async fn delegated_exchange(
        server: &TestServer,
        client: &TestClient,
        subject_token: &str,
        actor_token: &str,
        audience: &TestClient,
    ) -> axum_test::TestResponse {
        server
            .post(&token_path())
            .form(&[
                ("grant_type", TOKEN_EXCHANGE_GRANT),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("subject_token", subject_token),
                ("subject_token_type", ACCESS_TOKEN_URN),
                ("actor_token", actor_token),
                ("actor_token_type", ACCESS_TOKEN_URN),
                ("audience", audience.client_id.as_str()),
            ])
            .await
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn a_delegated_exchange_carries_the_act_claim() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client_with(&server, "gateway", true, true).await;
            let orders = create_client(&server, "orders", false).await;
            create_policy(&server, &gateway, &orders, &["profile"], false, true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let actor_token = service_token(&server, &gateway).await;
            let actor = claims(&actor_token);

            let response =
                delegated_exchange(&server, &gateway, &subject_token, &actor_token, &orders).await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            let issued = claims(
                response.json::<Value>()["access_token"]
                    .as_str()
                    .expect("token"),
            );
            assert_eq!(issued["sub"], claims(&subject_token)["sub"]);
            assert_eq!(
                issued["act"],
                json!({ "sub": actor["sub"], "client_id": gateway.client_id })
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn delegation_is_refused_without_allow_delegation() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client_with(&server, "gateway", true, true).await;
            let orders = create_client(&server, "orders", false).await;
            create_policy(&server, &gateway, &orders, &["profile"], true, false).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let actor_token = service_token(&server, &gateway).await;

            let response =
                delegated_exchange(&server, &gateway, &subject_token, &actor_token, &orders).await;

            assert_oauth_error(&response, 400, "unauthorized_client");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn an_actor_token_of_another_client_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client_with(&server, "gateway", true, true).await;
            let other = create_client_with(&server, "other", false, true).await;
            let orders = create_client(&server, "orders", false).await;
            create_policy(&server, &gateway, &orders, &["profile"], false, true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let foreign_actor = service_token(&server, &other).await;

            let response =
                delegated_exchange(&server, &gateway, &subject_token, &foreign_actor, &orders)
                    .await;

            assert_oauth_error(&response, 400, "invalid_request");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_test -- --ignored"]
    fn a_chained_delegation_nests_the_previous_act() {
        rt().block_on(async {
            let server = make_server();
            let gateway = create_client_with(&server, "gateway", true, true).await;
            let orders = create_client(&server, "orders", false).await;
            create_policy(&server, &gateway, &orders, &["profile"], false, true).await;
            let subject_token = subject_token_for(&server, &gateway).await;
            let actor_token = service_token(&server, &gateway).await;
            let actor_sub = claims(&actor_token)["sub"].clone();

            let first =
                delegated_exchange(&server, &gateway, &subject_token, &actor_token, &orders).await;
            assert_eq!(first.status_code(), 200, "{}", first.text());
            let first_token = first.json::<Value>()["access_token"]
                .as_str()
                .expect("token")
                .to_string();

            let second =
                delegated_exchange(&server, &gateway, &first_token, &actor_token, &orders).await;
            assert_eq!(second.status_code(), 200, "{}", second.text());
            let issued = claims(
                second.json::<Value>()["access_token"]
                    .as_str()
                    .expect("token"),
            );

            let link = json!({ "sub": actor_sub, "client_id": gateway.client_id });
            let mut expected = link.clone();
            expected["act"] = link;
            assert_eq!(issued["act"], expected);
        });
    }
}
