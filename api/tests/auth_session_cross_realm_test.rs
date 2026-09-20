#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
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
    use serde_json::{Value, json};
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const CONSOLE_CLIENT_ID: &str = "security-admin-console";
    const MASTER_REALM: &str = "master";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const S256_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-Adm!";
    const BOB_PASSWORD: &str = "B0b-Tenant-Adm!";

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
        tenant_a: String,
        tenant_b: String,
        alice_username: String,
        alice_id: String,
        bob_username: String,
        pool: PgPool,
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

        let schema = format!("auth_session_cross_realm_{}", Uuid::new_v4().simple());

        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = PgPool::connect(&admin_url)
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

        let pool = PgPool::connect(&schema_url)
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
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        let server = TestServer::new(app.clone()).expect("create fixture server");
        let admin_token = direct_grant(&server, MASTER_REALM, "admin", "admin").await;

        let suffix = Uuid::new_v4().simple().to_string();
        let tenant_a = format!("tenant-a-{}", &suffix[..8]);
        let tenant_b = format!("tenant-b-{}", &suffix[..8]);

        create_realm(&server, &admin_token, &tenant_a).await;
        create_realm(&server, &admin_token, &tenant_b).await;

        let alice_username = format!("alice-{}", &suffix[..8]);
        let alice_id = create_user(&server, &admin_token, &tenant_a, &alice_username).await;
        set_password(&server, &admin_token, &tenant_a, &alice_id, ALICE_PASSWORD).await;

        let bob_username = format!("bob-{}", &suffix[..8]);
        let bob_id = create_user(&server, &admin_token, &tenant_b, &bob_username).await;
        set_password(&server, &admin_token, &tenant_b, &bob_id, BOB_PASSWORD).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            tenant_a,
            tenant_b,
            alice_username,
            alice_id,
            bob_username,
            pool,
        }
    }

    fn make_server() -> TestServer {
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn tenant_a() -> &'static str {
        shared_ctx().tenant_a.as_str()
    }

    fn tenant_b() -> &'static str {
        shared_ctx().tenant_b.as_str()
    }

    fn alice_username() -> &'static str {
        shared_ctx().alice_username.as_str()
    }

    fn bob_username() -> &'static str {
        shared_ctx().bob_username.as_str()
    }

    fn alice_id() -> Uuid {
        Uuid::parse_str(shared_ctx().alice_id.as_str()).expect("alice carries a uuid")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    async fn direct_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "token request for {username}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .unwrap_or_else(|| panic!("access_token in response for {username}@{realm}: {body}"))
            .to_string()
    }

    async fn create_realm(server: &TestServer, token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": name }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "create realm {name} failed: {}",
            response.text()
        );
    }

    async fn create_user(server: &TestServer, token: &str, realm: &str, username: &str) -> String {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{}@test.local", username),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "create user {username} in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("created user id in response: {body}"))
            .to_string()
    }

    async fn set_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "set password for {user_id} in {realm} failed: {}",
            response.text()
        );
    }

    fn console_callback(realm: &str) -> String {
        format!("{WEBAPP_URL}/realms/{realm}/authentication/callback")
    }

    async fn start_console_authorization(server: &TestServer, realm: &str) -> TestResponse {
        let redirect_uri = console_callback(realm);

        let response = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", CONSOLE_CLIENT_ID)
            .add_query_param("redirect_uri", redirect_uri.as_str())
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;

        let status = response.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth on {realm}, got {status}: {}",
            response.text()
        );

        response
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test auth_session_cross_realm_test -- --ignored"]
    fn a_session_opened_in_another_realm_cannot_be_authenticated() {
        rt().block_on(async {
            let server = make_server();
            let authorize = start_console_authorization(&server, tenant_b()).await;

            let login = server
                .post(&format!(
                    "/realms/{}/login-actions/authenticate",
                    tenant_a()
                ))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", CONSOLE_CLIENT_ID)
                .json(&json!({ "username": alice_username(), "password": ALICE_PASSWORD }))
                .await;

            let status = login.status_code().as_u16();
            let body: Value = login.json();

            assert_eq!(
                status, 401,
                "an authorization request opened in another realm must be refused: {body}"
            );
            assert_eq!(
                body["message"],
                json!("Invalid session"),
                "the refusal must come from the realm binding: {body}"
            );
            assert!(
                body["url"].is_null(),
                "no authorization code may be handed back: {body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test auth_session_cross_realm_test -- --ignored"]
    fn a_session_opened_in_the_realm_it_is_used_in_still_authenticates() {
        rt().block_on(async {
            let server = make_server();
            let authorize = start_console_authorization(&server, tenant_b()).await;

            let login = server
                .post(&format!(
                    "/realms/{}/login-actions/authenticate",
                    tenant_b()
                ))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", CONSOLE_CLIENT_ID)
                .json(&json!({ "username": bob_username(), "password": BOB_PASSWORD }))
                .await;

            let status = login.status_code().as_u16();
            let body: Value = login.json();

            assert_eq!(status, 200, "the nominal login must keep working: {body}");

            let url = body["url"]
                .as_str()
                .unwrap_or_else(|| panic!("a successful login must carry a redirect url: {body}"));

            assert!(
                url.starts_with(&console_callback(tenant_b())),
                "the browser must go back to the client that started the flow: {url}"
            );
            assert!(
                url.contains("code="),
                "the redirect must carry an authorization code: {url}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test auth_session_cross_realm_test -- --ignored"]
    fn a_code_bound_to_a_user_of_another_realm_is_refused_at_the_token_endpoint() {
        rt().block_on(async {
            let server = make_server();
            let authorize = start_console_authorization(&server, tenant_b()).await;
            let session_id = Uuid::parse_str(authorize.cookie("FERRISKEY_SESSION").value())
                .expect("the session cookie carries a uuid");
            let code = format!("planted-{}", Uuid::new_v4().simple());

            sqlx::query("UPDATE auth_sessions SET user_id = $1, code = $2 WHERE id = $3")
                .bind(alice_id())
                .bind(&code)
                .bind(session_id)
                .execute(&shared_ctx().pool)
                .await
                .expect("bind a foreign identity to the tenant-b authorization request");

            let redirect_uri = console_callback(tenant_b());

            let response = server
                .post(&format!(
                    "/realms/{}/protocol/openid-connect/token",
                    tenant_b()
                ))
                .form(&[
                    ("grant_type", "authorization_code"),
                    ("client_id", CONSOLE_CLIENT_ID),
                    ("code", code.as_str()),
                    ("redirect_uri", redirect_uri.as_str()),
                    ("code_verifier", S256_VERIFIER),
                ])
                .await;

            let status = response.status_code().as_u16();
            let body: Value = response.json();

            assert_eq!(
                status, 400,
                "a realm must never sign a token for an account it does not own: {body}"
            );
            assert_eq!(body["error"], json!("invalid_grant"), "{body}");
            assert!(
                body["access_token"].is_null(),
                "no token may be minted for a foreign identity: {body}"
            );
        });
    }
}
