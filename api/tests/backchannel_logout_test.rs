//! OIDC Back-Channel Logout 1.0 end to end: a fake relying party listens on
//! loopback, and every way a session ends must reach it with a logout token
//! signed by the realm key.

#[cfg(test)]
mod tests {
    use std::{
        env,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use axum::{Form, Router, extract::State, http::HeaderValue, routing::post};
    use axum_test::TestServer;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::Args,
    };
    use ferriskey_core::{
        application::create_service,
        domain::common::{
            DatabaseConfig, FerriskeyConfig, entities::StartupConfig, ports::CoreService,
        },
    };
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use serde::Deserialize;
    use serde_json::{Value, json};
    use sqlx::Executor;
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const MASTER_REALM: &str = "master";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const MASTER_ADMIN_USERNAME: &str = "admin";
    const MASTER_ADMIN_PASSWORD: &str = "admin";
    const REALM: &str = "backchannel";
    const PASSWORD: &str = "Backchannel-Passw0rd!";
    const LOGOUT_EVENT: &str = "http://schemas.openid.net/event/backchannel-logout";

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

        let schema = format!("backchannel_logout_test_{}", Uuid::new_v4().simple());

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

        // The fake relying party listens on loopback, which only a deployment
        // that allows private endpoints may reach.
        let service = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: true,
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

        let state = AppState::new(Arc::new(Args::default()), service);
        let app = router(state).expect("build router");
        let ctx = SharedContext {
            app: std::sync::Mutex::new(app),
        };

        let server = TestServer::new(ctx.app.lock().expect("router mutex").clone())
            .expect("create test server");
        let master = master_token(&server).await;
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(&master))
            .json(&json!({ "name": REALM, "display_name": REALM }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());

        ctx
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

    fn token_path(realm: &str) -> String {
        format!("/realms/{realm}/protocol/openid-connect/token")
    }

    async fn master_token(server: &TestServer) -> String {
        let response = server
            .post(&token_path(MASTER_REALM))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", MASTER_ADMIN_USERNAME),
                ("password", MASTER_ADMIN_PASSWORD),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    /// A relying party that records every logout token POSTed to it.
    struct FakeRelyingParty {
        endpoint: String,
        received: Arc<Mutex<Vec<String>>>,
    }

    #[derive(Deserialize)]
    struct LogoutForm {
        logout_token: String,
    }

    async fn record(
        State(received): State<Arc<Mutex<Vec<String>>>>,
        Form(form): Form<LogoutForm>,
    ) -> &'static str {
        received
            .lock()
            .expect("received lock")
            .push(form.logout_token);
        "ok"
    }

    async fn fake_relying_party() -> FakeRelyingParty {
        let received = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .route("/backchannel-logout", post(record))
            .with_state(received.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind the fake relying party");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        FakeRelyingParty {
            endpoint: format!("http://{addr}/backchannel-logout"),
            received,
        }
    }

    impl FakeRelyingParty {
        /// Waits up to five seconds for `count` logout tokens.
        async fn wait_for(&self, count: usize) -> Vec<String> {
            for _ in 0..50 {
                let received = self.received.lock().expect("received lock").clone();
                if received.len() >= count {
                    return received;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            self.received.lock().expect("received lock").clone()
        }

        /// Gives a delivery that should not happen the time to happen anyway.
        async fn stays_silent(&self) -> bool {
            tokio::time::sleep(Duration::from_secs(1)).await;
            self.received.lock().expect("received lock").is_empty()
        }
    }

    struct TestClient {
        uuid: String,
        client_id: String,
        secret: String,
    }

    async fn create_client(
        server: &TestServer,
        backchannel_logout_uri: Option<&str>,
    ) -> TestClient {
        let master = master_token(server).await;
        let client_id = format!("app-{}", Uuid::new_v4().simple());
        let response = server
            .post(&format!("/realms/{REALM}/clients"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "client_id": client_id,
                "name": "app",
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": false,
                "direct_access_grants_enabled": true,
                "enabled": true,
                "oauth_device_code_grant_enabled": false,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let body: Value = response.json();
        let client = TestClient {
            uuid: body["id"].as_str().expect("client id").to_string(),
            client_id,
            secret: body["client_secret"]
                .as_str()
                .expect("creation hands back the plaintext secret")
                .to_string(),
        };

        if let Some(uri) = backchannel_logout_uri {
            let response =
                update_client(server, &client, json!({ "backchannel_logout_uri": uri })).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
        }
        client
    }

    async fn update_client(
        server: &TestServer,
        client: &TestClient,
        body: Value,
    ) -> axum_test::TestResponse {
        let master = master_token(server).await;
        server
            .patch(&format!("/realms/{REALM}/clients/{}", client.uuid))
            .add_header("Authorization", auth_header(&master))
            .json(&body)
            .await
    }

    struct TestUser {
        id: String,
        username: String,
    }

    async fn create_user(server: &TestServer) -> TestUser {
        let master = master_token(server).await;
        let username = format!("user-{}", Uuid::new_v4().simple());
        let response = server
            .post(&format!("/realms/{REALM}/users"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "username": username,
                "firstname": "Back",
                "lastname": "Channel",
                "email": format!("{username}@backchannel.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let id = response.json::<Value>()["data"]["id"]
            .as_str()
            .expect("user id")
            .to_string();

        let response = server
            .put(&format!("/realms/{REALM}/users/{id}/reset-password"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({ "value": PASSWORD, "temporary": false, "credential_type": "password" }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        TestUser { id, username }
    }

    /// Logs `user` in through `client`; the password grant opens a session.
    async fn sign_in(server: &TestServer, client: &TestClient, user: &TestUser) -> Value {
        let response = server
            .post(&token_path(REALM))
            .form(&[
                ("grant_type", "password"),
                ("client_id", client.client_id.as_str()),
                ("client_secret", client.secret.as_str()),
                ("username", user.username.as_str()),
                ("password", PASSWORD),
                ("scope", "openid profile"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json()
    }

    fn payload(token: &str) -> Value {
        let part = token.split('.').nth(1).expect("a JWT has a payload");
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).expect("base64url")).expect("JSON")
    }

    fn header(token: &str) -> Value {
        let part = token.split('.').next().expect("a JWT has a header");
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(part).expect("base64url")).expect("JSON")
    }

    /// Verifies `token` against the realm JWKS and returns its claims.
    async fn verified(server: &TestServer, token: &str, audience: &str) -> Value {
        let jwks: Value = server
            .get(&format!(
                "/realms/{REALM}/protocol/openid-connect/jwks.json"
            ))
            .await
            .json();
        let kid = header(token)["kid"].as_str().expect("kid").to_string();
        let key = jwks["keys"]
            .as_array()
            .expect("keys")
            .iter()
            .find(|key| key["kid"] == json!(kid))
            .expect("the signing key is published");
        let decoding = DecodingKey::from_rsa_components(
            key["n"].as_str().expect("n"),
            key["e"].as_str().expect("e"),
        )
        .expect("RSA key");
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[audience]);
        validation.set_required_spec_claims(&["iss", "aud", "iat", "exp"]);
        jsonwebtoken::decode::<Value>(token, &decoding, &validation)
            .expect("the logout token verifies against the realm JWKS")
            .claims
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn rp_initiated_logout_sends_a_signed_logout_token() {
        rt().block_on(async {
            let server = make_server();
            let rp = fake_relying_party().await;
            let client = create_client(&server, Some(&rp.endpoint)).await;
            let user = create_user(&server).await;
            let tokens = sign_in(&server, &client, &user).await;
            let access = payload(tokens["access_token"].as_str().expect("access_token"));

            let response = server
                .post(&format!("/realms/{REALM}/protocol/openid-connect/logout"))
                .form(&[(
                    "id_token_hint",
                    tokens["id_token"].as_str().expect("id_token"),
                )])
                .await;
            assert!(
                response.status_code().is_success() || response.status_code().is_redirection(),
                "{}",
                response.text()
            );

            let received = rp.wait_for(1).await;
            assert_eq!(received.len(), 1, "exactly one logout token");
            let token = &received[0];
            assert_eq!(header(token)["typ"], json!("logout+jwt"));

            let claims = verified(&server, token, &client.client_id).await;
            assert_eq!(claims["iss"], access["iss"]);
            assert_eq!(claims["aud"], json!(client.client_id));
            assert_eq!(claims["sub"], json!(user.id));
            assert_eq!(claims["sid"], access["sid"]);
            assert!(claims["jti"].is_string());
            assert_eq!(claims["events"], json!({ LOGOUT_EVENT: {} }));
            assert!(claims.get("nonce").is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn an_admin_revoking_a_session_notifies_its_clients() {
        rt().block_on(async {
            let server = make_server();
            let rp = fake_relying_party().await;
            let client = create_client(&server, Some(&rp.endpoint)).await;
            let user = create_user(&server).await;
            let tokens = sign_in(&server, &client, &user).await;
            let sid = payload(tokens["access_token"].as_str().expect("access_token"))["sid"]
                .as_str()
                .expect("sid")
                .to_string();

            let master = master_token(&server).await;
            let response = server
                .delete(&format!("/realms/{REALM}/users/{}/sessions/{sid}", user.id))
                .add_header("Authorization", auth_header(&master))
                .await;
            assert!(response.status_code().is_success(), "{}", response.text());

            let received = rp.wait_for(1).await;
            assert_eq!(received.len(), 1, "{received:?}");
            assert_eq!(payload(&received[0])["sid"], json!(sid));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn disabling_a_user_notifies_the_clients_of_every_session() {
        rt().block_on(async {
            let server = make_server();
            let rp = fake_relying_party().await;
            let client = create_client(&server, Some(&rp.endpoint)).await;
            let user = create_user(&server).await;
            let first = sign_in(&server, &client, &user).await;
            let second = sign_in(&server, &client, &user).await;

            let master = master_token(&server).await;
            let response = server
                .put(&format!("/realms/{REALM}/users/{}", user.id))
                .add_header("Authorization", auth_header(&master))
                .json(&json!({
                    "firstname": "Back",
                    "lastname": "Channel",
                    "email": format!("{}@backchannel.local", user.username),
                    "email_verified": true,
                    "enabled": false,
                }))
                .await;
            assert!(response.status_code().is_success(), "{}", response.text());

            let received = rp.wait_for(2).await;
            let mut sids: Vec<Value> = received
                .iter()
                .map(|token| payload(token)["sid"].clone())
                .collect();
            sids.sort_by_key(|sid| sid.to_string());
            let mut expected = vec![
                payload(first["access_token"].as_str().expect("token"))["sid"].clone(),
                payload(second["access_token"].as_str().expect("token"))["sid"].clone(),
            ];
            expected.sort_by_key(|sid| sid.to_string());
            assert_eq!(sids, expected);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn only_clients_of_the_session_with_an_endpoint_are_notified() {
        rt().block_on(async {
            let server = make_server();
            let notified = fake_relying_party().await;
            let bystander = fake_relying_party().await;
            let used = create_client(&server, Some(&notified.endpoint)).await;
            // Registered an endpoint but never took part in the session.
            create_client(&server, Some(&bystander.endpoint)).await;
            // Took part in the session but registered no endpoint.
            let silent = create_client(&server, None).await;
            let user = create_user(&server).await;
            let tokens = sign_in(&server, &used, &user).await;
            sign_in(&server, &silent, &user).await;

            server
                .post(&format!("/realms/{REALM}/protocol/openid-connect/logout"))
                .form(&[(
                    "id_token_hint",
                    tokens["id_token"].as_str().expect("id_token"),
                )])
                .await;

            assert_eq!(notified.wait_for(1).await.len(), 1);
            assert!(
                bystander.stays_silent().await,
                "a client outside the session is not told"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn the_endpoint_is_validated_stored_and_cleared() {
        rt().block_on(async {
            let server = make_server();
            let client = create_client(&server, None).await;

            let refused = update_client(
                &server,
                &client,
                json!({ "backchannel_logout_uri": "not a url" }),
            )
            .await;
            assert_eq!(refused.status_code(), 422, "{}", refused.text());

            let stored = update_client(
                &server,
                &client,
                json!({
                    "backchannel_logout_uri": "https://app.example/backchannel-logout",
                    "backchannel_logout_session_required": false,
                }),
            )
            .await;
            assert_eq!(stored.status_code(), 200, "{}", stored.text());
            let body: Value = stored.json();
            assert_eq!(
                body["data"]["backchannel_logout_uri"],
                json!("https://app.example/backchannel-logout")
            );
            assert_eq!(
                body["data"]["backchannel_logout_session_required"],
                json!(false)
            );

            let untouched = update_client(&server, &client, json!({ "name": "renamed" })).await;
            assert_eq!(
                untouched.json::<Value>()["data"]["backchannel_logout_uri"],
                json!("https://app.example/backchannel-logout"),
                "an absent field keeps the stored endpoint"
            );

            let cleared =
                update_client(&server, &client, json!({ "backchannel_logout_uri": null })).await;
            assert_eq!(
                cleared.json::<Value>()["data"]["backchannel_logout_uri"],
                Value::Null
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test backchannel_logout_test -- --ignored"]
    fn discovery_advertises_back_channel_logout() {
        rt().block_on(async {
            let server = make_server();
            let body: Value = server
                .get(&format!("/realms/{REALM}/.well-known/openid-configuration"))
                .await
                .json();
            assert_eq!(body["backchannel_logout_supported"], json!(true));
            assert_eq!(body["backchannel_logout_session_supported"], json!(true));
        });
    }
}
