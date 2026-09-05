/// Integration tests for the SeaWatch audit log (#1270).
///
/// These pin two things that were previously broken:
/// - every stored event is actually linked into the tamper-evident hash chain
///   (`event_hash`/`prev_hash` are populated, not left NULL)
/// - `GET .../seawatch/v1/security-events` actually applies the query filter
///   it's given instead of always returning the unfiltered first page
///
/// Require a running PostgreSQL instance. Marked `#[ignore]` so they don't
/// block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test seawatch_test -- --ignored
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

        let schema = format!("test_seawatch_{}", Uuid::new_v4().simple());
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

    async fn login(server: &TestServer, realm_name: &str) -> String {
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

        let body: Value = token_resp.json();
        body["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    async fn create_user(srv: &TestServer, realm: &str, token: &str, username: &str) {
        let resp = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&serde_json::json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "user creation failed: {}",
            resp.text()
        );
    }

    async fn security_events(srv: &TestServer, realm: &str, token: &str, query: &str) -> Value {
        let path = if query.is_empty() {
            format!("/realms/{}/seawatch/v1/security-events", realm)
        } else {
            format!("/realms/{}/seawatch/v1/security-events?{}", realm, query)
        };
        let resp = srv
            .get(&path)
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "fetching security events failed: {}",
            resp.text()
        );
        resp.json()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn every_stored_event_is_linked_into_the_hash_chain() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;

            let body = security_events(&srv, &realm, &token, "").await;
            let events = body["data"].as_array().expect("events array");

            let session_created = events
                .iter()
                .find(|e| e["event_type"] == "session_created")
                .expect("a session_created event was recorded for the login above");

            assert!(
                !session_created["event_hash"].is_null(),
                "session_created event has no event_hash — the chain was not built: {session_created}"
            );
            assert!(
                !session_created["prev_hash"].is_null(),
                "session_created event has no prev_hash — the chain was not built: {session_created}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn event_types_filter_is_actually_applied() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            create_user(&srv, &realm, &token, &format!("filtertest-{}", Uuid::new_v4().simple())).await;

            // Unfiltered: both event types the two actions above produced should be present.
            let all = security_events(&srv, &realm, &token, "").await;
            let all_events = all["data"].as_array().expect("events array");
            assert!(
                all_events.iter().any(|e| e["event_type"] == "session_created"),
                "expected a session_created event in the unfiltered page: {all_events:?}"
            );
            assert!(
                all_events.iter().any(|e| e["event_type"] == "user_created"),
                "expected a user_created event in the unfiltered page: {all_events:?}"
            );

            // Filtered to user_created only: session_created must not leak through.
            let filtered = security_events(&srv, &realm, &token, "event_types=user_created").await;
            let filtered_events = filtered["data"].as_array().expect("events array");
            assert!(
                !filtered_events.is_empty(),
                "expected at least one user_created event: {filtered_events:?}"
            );
            assert!(
                filtered_events.iter().all(|e| e["event_type"] == "user_created"),
                "the event_types filter was not applied — a non-user_created event leaked through: {filtered_events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn limit_is_actually_applied() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            create_user(
                &srv,
                &realm,
                &token,
                &format!("limittest-{}", Uuid::new_v4().simple()),
            )
            .await;

            let limited = security_events(&srv, &realm, &token, "limit=1").await;
            let events = limited["data"].as_array().expect("events array");
            assert_eq!(
                events.len(),
                1,
                "limit=1 should return exactly one event: {events:?}"
            );
        });
    }
}
