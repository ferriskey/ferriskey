/// Integration tests for the OAuth 2.0 Device Authorization Grant (RFC 8628).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test device_flow_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
///
/// ### Why a shared runtime and not `#[tokio::test]`?
///
/// `axum_prometheus::PrometheusMetricLayer::pair()` (called inside
/// `ferriskey_api::application::http::server::http_server::router`) installs a
/// **process-wide** metrics recorder that panics on a second call (see #1086).
/// Using a single long-lived `tokio::runtime::Runtime` ensures `router()` is
/// called exactly once and all async I/O (sqlx pool, Axum handlers) stays on the
/// same executor — required for correct scheduling.
///
/// `axum_test::TestServer::new(Router)` uses mock transport (no TCP listener,
/// no `tokio::spawn`), so it can be constructed synchronously and reused
/// across test functions without needing a runtime context at construction time.
#[cfg(test)]
mod tests {
    use std::{collections::HashSet, env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
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
    use sqlx::Executor;
    use uuid::Uuid;

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    /// Shared context built exactly once per process.
    struct SharedContext {
        /// The `Router` is `!Sync` (it holds boxed services), so it is guarded by a
        /// `Mutex` to let `SharedContext` live in a `OnceLock` static without
        /// `unsafe`. Each test briefly locks only to clone a fresh server.
        app: std::sync::Mutex<Router>,
        realm_name: String,
        /// Pool in the isolated test schema (for direct SQL in the expiry test).
        pool: sqlx::PgPool,
    }

    /// Single long-lived multi-threaded runtime shared by all tests.
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

    /// Returns the shared context, initialising it on first call.
    ///
    /// Must be called from a synchronous context (not inside `rt().block_on()`)
    /// because init itself calls `rt().block_on(init_shared_ctx())`.
    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("device_flow_test_{}", Uuid::new_v4().simple());

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

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());

        // Use a non-"admin-cli" default_client_id so that the dedicated
        // admin-cli seeding path (direct_access_grants_enabled=true,
        // client_type=System) is not short-circuited by the generic
        // confidential-client creation that runs first for the default client
        // (see #1086).
        service
            .initialize_application(StartupConfig {
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            pool,
        }
    }

    /// Build a fresh `TestServer` from the shared router clone.
    ///
    /// `TestServer::new(Router)` uses mock (in-memory) transport and does not
    /// call `tokio::spawn`, so it is safe to call synchronously before entering
    /// `rt().block_on()`.
    fn make_server() -> TestServer {
        // Ensure the shared context exists before building a TestServer.
        let ctx = shared_ctx();
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
    }

    fn realm() -> &'static str {
        shared_ctx().realm_name.as_str()
    }

    async fn get_admin_token(server: &TestServer) -> String {
        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "admin token request failed: {}",
            response.text()
        );
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    /// Create a public client with the device authorization grant enabled.
    /// Returns the generated `client_id` string.
    async fn create_device_client(server: &TestServer, admin_token: &str) -> String {
        let client_id = format!("device-client-{}", Uuid::new_v4().simple());

        let response = server
            .post(&format!("/realms/{}/clients", realm()))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "client_id": client_id,
                "name": "Device Test Client",
                "client_type": "public",
                "protocol": "openid-connect",
                "public_client": true,
                "service_account_enabled": false,
                "direct_access_grants_enabled": false,
                "enabled": true,
                "oauth_device_code_grant_enabled": true
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "client creation failed: {}",
            response.text()
        );
        client_id
    }

    /// POST to the device authorization endpoint. Returns the parsed JSON body.
    async fn initiate(server: &TestServer, client_id: &str, scope: Option<&str>) -> Value {
        let mut fields = vec![("client_id", client_id)];
        let scope_str;
        if let Some(s) = scope {
            scope_str = s.to_string();
            fields.push(("scope", scope_str.as_str()));
        }

        let resp = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/auth/device",
                realm()
            ))
            .form(&fields)
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "device authorization initiate failed: {}",
            resp.text()
        );
        resp.json()
    }

    /// POST to the token endpoint with the device_code grant. Returns the raw response.
    async fn poll(server: &TestServer, client_id: &str, device_code: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("device_code", device_code),
                ("client_id", client_id),
            ])
            .await
    }

    /// POST to the device verify endpoint (approve or deny).
    /// The admin token is supplied via the FERRISKEY_IDENTITY cookie.
    async fn verify(
        server: &TestServer,
        admin_token: &str,
        user_code: &str,
        action: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/device/verify", realm()))
            // The handler reads the identity from the FERRISKEY_IDENTITY cookie.
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_IDENTITY={admin_token}")).unwrap(),
            )
            .json(&json!({
                "user_code": user_code,
                "action": action
            }))
            .await
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    #[test]
    #[ignore]
    fn happy_path_initiate_verify_poll_returns_token() {
        // Build the server synchronously (mock transport, no runtime needed).
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_id, Some("openid")).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");
            let user_code = init_body["user_code"].as_str().expect("user_code");

            let verify_resp = verify(&server, &admin_token, user_code, "approve").await;
            assert_eq!(
                verify_resp.status_code(),
                200,
                "approve failed: {}",
                verify_resp.text()
            );
            let verify_body: Value = verify_resp.json();
            assert_eq!(verify_body["status"], "approved");

            let poll_resp = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll_resp.status_code(),
                200,
                "poll after approve failed: {}",
                poll_resp.text()
            );
            let token_body: Value = poll_resp.json();
            let access_token = token_body["access_token"]
                .as_str()
                .expect("access_token in poll response");
            assert!(!access_token.is_empty(), "access_token must not be empty");
        });
    }

    #[test]
    #[ignore]
    fn poll_before_verify_returns_authorization_pending() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_id, None).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");

            let poll_resp = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll_resp.status_code(),
                400,
                "expected 400, got: {}",
                poll_resp.text()
            );
            let body: Value = poll_resp.json();
            assert_eq!(
                body["error"], "authorization_pending",
                "unexpected error body: {body}"
            );
        });
    }

    #[test]
    #[ignore]
    fn two_polls_within_interval_returns_slow_down() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_id, None).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");

            // First poll records last_polled_at → authorization_pending.
            let poll1 = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll1.status_code(),
                400,
                "poll #1 expected 400: {}",
                poll1.text()
            );
            let body1: Value = poll1.json();
            assert_eq!(
                body1["error"], "authorization_pending",
                "poll #1 unexpected error: {body1}"
            );

            // Second poll arrives within the 5-second interval → slow_down.
            let poll2 = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll2.status_code(),
                400,
                "poll #2 expected 400: {}",
                poll2.text()
            );
            let body2: Value = poll2.json();
            assert_eq!(
                body2["error"], "slow_down",
                "poll #2 unexpected error: {body2}"
            );
        });
    }

    #[test]
    #[ignore]
    fn poll_after_expiry_returns_expired_token() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_id, None).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");

            // Validate the device_code is a UUID before interpolating it into SQL.
            let dc = uuid::Uuid::parse_str(device_code).expect("device_code is a uuid");
            sqlx::query(&format!(
                "UPDATE device_auth_sessions SET expires_at = now() - interval '1 hour' WHERE device_code = '{}'",
                dc
            ))
            .execute(&shared_ctx().pool)
            .await
            .expect("expire session");

            let poll_resp = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll_resp.status_code(),
                400,
                "expected 400 for expired session: {}",
                poll_resp.text()
            );
            let body: Value = poll_resp.json();
            assert_eq!(
                body["error"],
                "expired_token",
                "unexpected error body: {body}"
            );
        });
    }

    #[test]
    #[ignore]
    fn poll_after_deny_returns_access_denied() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_id, None).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");
            let user_code = init_body["user_code"].as_str().expect("user_code");

            let deny_resp = verify(&server, &admin_token, user_code, "deny").await;
            assert_eq!(
                deny_resp.status_code(),
                200,
                "deny failed: {}",
                deny_resp.text()
            );
            let deny_body: Value = deny_resp.json();
            assert_eq!(deny_body["status"], "denied");

            let poll_resp = poll(&server, &client_id, device_code).await;
            assert_eq!(
                poll_resp.status_code(),
                400,
                "expected 400 after deny: {}",
                poll_resp.text()
            );
            let body: Value = poll_resp.json();
            assert_eq!(
                body["error"], "access_denied",
                "unexpected error body: {body}"
            );
        });
    }

    #[test]
    #[ignore]
    fn user_codes_are_unique_across_many_initiates() {
        // The deterministic collision-retry path itself is covered by the unit
        // test `initiate_retries_on_user_code_collision` in
        // `core/src/domain/authentication/device_flow/services.rs`; a true
        // random collision cannot be forced through the HTTP layer, so this
        // asserts the generation+uniqueness machinery end-to-end.
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let client_id = create_device_client(&server, &admin_token).await;

            let mut user_codes = HashSet::new();
            for _ in 0..30 {
                let body = initiate(&server, &client_id, None).await;
                let code = body["user_code"]
                    .as_str()
                    .expect("user_code in response")
                    .to_string();
                user_codes.insert(code);
            }

            assert_eq!(user_codes.len(), 30, "expected 30 unique user codes");
        });
    }

    #[test]
    #[ignore]
    fn poll_with_mismatched_client_returns_invalid_client() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;

            // Client A initiates the flow.
            let client_a = create_device_client(&server, &admin_token).await;
            // Client B is a separate device-grant-enabled client.
            let client_b = create_device_client(&server, &admin_token).await;

            let init_body = initiate(&server, &client_a, None).await;
            let device_code = init_body["device_code"].as_str().expect("device_code");

            // Poll with A's device_code but B's client_id — exercises the
            // device_code ↔ client binding check, not merely "client not found".
            let poll_resp = poll(&server, &client_b, device_code).await;
            assert_eq!(
                poll_resp.status_code(),
                400,
                "expected 400 for mismatched client: {}",
                poll_resp.text()
            );
            let body: Value = poll_resp.json();
            assert_eq!(
                body["error"], "invalid_client",
                "unexpected error body: {body}"
            );
        });
    }
}
