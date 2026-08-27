/// Integration tests for the active session management API (#1093).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test session_management_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
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
    use serde_json::Value;
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

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        realm_name: String,
        /// Kept alive for the process lifetime; not read directly by these tests.
        #[allow(dead_code)]
        pool: sqlx::PgPool,
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

    fn ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(async { setup().await }))
    }

    async fn setup() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("test_session_mgmt_{}", Uuid::new_v4().simple());
        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema).as_str())
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

        let svc = create_service(FerriskeyConfig {
            webapp_url: "http://localhost:5555".to_string(),
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

        let realm_name = format!("test-realm-{}", Uuid::new_v4().simple());
        svc.initialize_application(StartupConfig {
            webapp_url: "http://localhost:5555".to_string(),
            master_realm_name: realm_name.clone(),
            admin_username: "admin".to_string(),
            admin_email: "admin@ferriskey.test".to_string(),
            admin_password: "admin_pass_1234!".to_string(),
            default_client_id: "ferriskey-admin".to_string(),
        })
        .await
        .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, svc);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            pool,
        }
    }

    fn server() -> TestServer {
        let app = ctx().app.lock().expect("lock app mutex").clone();
        TestServer::new(app).expect("build test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    fn sid_claim(access_token: &str) -> Option<String> {
        let payload = access_token.split('.').nth(1)?;
        let raw = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let claims: Value = serde_json::from_slice(&raw).ok()?;
        claims.get("sid")?.as_str().map(str::to_string)
    }

    async fn login(server: &TestServer, realm_name: &str) -> String {
        login_tokens(server, realm_name).await["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    async fn login_tokens(server: &TestServer, realm_name: &str) -> Value {
        let token_resp = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin_pass_1234!"),
                ("scope", "openid profile"),
            ])
            .await;

        assert_eq!(
            token_resp.status_code(),
            200,
            "password grant failed: {}",
            token_resp.text()
        );

        token_resp.json()
    }

    async fn admin_user_id(srv: &TestServer, realm: &str, token: &str) -> String {
        let me_resp = srv
            .get(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .await;
        let me_body: Value = me_resp.json();
        let users = me_body["data"].as_array().expect("users array");
        users
            .iter()
            .find(|u| u["username"] == "admin")
            .and_then(|u| u["id"].as_str())
            .expect("admin user id")
            .to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn admin_can_list_own_sessions() {
        // Build the server + resolve the realm synchronously (initialising the
        // shared context here, before entering `block_on`, avoids a nested
        // runtime panic since `ctx()` itself calls `rt().block_on()`).
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let admin_id = admin_user_id(&srv, &realm, &token).await;

            // List sessions
            let sessions_resp = srv
                .get(&format!("/realms/{}/users/{}/sessions", realm, admin_id))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(sessions_resp.status_code(), 200);
            let body: Value = sessions_resp.json();
            let sessions = body["data"].as_array().expect("sessions array");

            let session_id =
                sid_claim(&token).expect("the password grant must bind its token to a session");
            assert!(
                sessions
                    .iter()
                    .any(|s| s["id"].as_str() == Some(session_id.as_str())),
                "the session backing the current token is not listed: {sessions:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn admin_can_revoke_own_session() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let tokens = login_tokens(&srv, &realm).await;
            let token = tokens["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();
            let refresh_token = tokens["refresh_token"]
                .as_str()
                .expect("refresh_token")
                .to_string();

            let admin_id = admin_user_id(&srv, &realm, &token).await;

            let session_id =
                sid_claim(&token).expect("the password grant must bind its token to a session");

            let sessions_resp = srv
                .get(&format!("/realms/{}/users/{}/sessions", realm, admin_id))
                .add_header("Authorization", auth_header(&token))
                .await;
            assert_eq!(sessions_resp.status_code(), 200);
            let sessions_body: Value = sessions_resp.json();
            let sessions = sessions_body["data"].as_array().expect("sessions array");
            assert!(
                sessions
                    .iter()
                    .any(|s| s["id"].as_str() == Some(session_id.as_str())),
                "the session backing the current token is not listed: {sessions:?}"
            );

            let revoke_resp = srv
                .delete(&format!(
                    "/realms/{}/users/{}/sessions/{}",
                    realm, admin_id, session_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                revoke_resp.status_code(),
                204,
                "revoking own session failed: {}",
                revoke_resp.text()
            );

            let after = srv
                .get(&format!(
                    "/realms/{}/protocol/openid-connect/userinfo",
                    realm
                ))
                .add_header("Authorization", auth_header(&token))
                .await;
            assert_eq!(
                after.status_code(),
                401,
                "the revoked session's access token still works: {}",
                after.text()
            );

            let refreshed = srv
                .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("client_id", "admin-cli"),
                    ("refresh_token", refresh_token.as_str()),
                ])
                .await;
            let refreshed_body = refreshed.text();
            assert_ne!(
                refreshed.status_code(),
                200,
                "the revoked session still rotated into a new token pair: {refreshed_body}"
            );
            assert!(
                !refreshed_body.contains("access_token"),
                "the refusal still handed back a token: {refreshed_body}"
            );

            let fresh = login(&srv, &realm).await;
            let sessions_after = srv
                .get(&format!("/realms/{}/users/{}/sessions", realm, admin_id))
                .add_header("Authorization", auth_header(&fresh))
                .await;
            assert_eq!(sessions_after.status_code(), 200);
            let remaining: Value = sessions_after.json();
            assert!(
                !remaining["data"]
                    .as_array()
                    .expect("sessions array")
                    .iter()
                    .any(|s| s["id"].as_str() == Some(session_id.as_str())),
                "the revoked session is still listed: {remaining}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn unauthenticated_request_returns_401() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let resp = srv
                .get(&format!(
                    "/realms/{}/users/{}/sessions",
                    realm,
                    Uuid::new_v4()
                ))
                .await;

            assert_eq!(resp.status_code(), 401);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn revoke_nonexistent_session_returns_404() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let admin_id = admin_user_id(&srv, &realm, &token).await;

            let fake_session_id = Uuid::new_v4();
            let revoke_resp = srv
                .delete(&format!(
                    "/realms/{}/users/{}/sessions/{}",
                    realm, admin_id, fake_session_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(revoke_resp.status_code(), 404);
        });
    }
}
