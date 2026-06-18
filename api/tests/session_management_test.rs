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
            admin_username: "admin".to_string(),
            admin_email: "admin@ferriskey.test".to_string(),
            admin_password: "admin_pass_1234!".to_string(),
            admin_firstname: None,
            admin_lastname: None,
            realm_name: realm_name.clone(),
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

    async fn login(server: &TestServer, realm_name: &str) -> String {
        let token_resp = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", &format!("{}-realm", realm_name)),
                ("username", "admin"),
                ("password", "admin_pass_1234!"),
            ])
            .await;
        let body: Value = token_resp.json();
        body["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn admin_can_list_own_sessions() {
        rt().block_on(async {
            let srv = server();
            let realm = &ctx().realm_name;
            let token = login(&srv, realm).await;

            // Get admin user id
            let me_resp = srv
                .get(&format!("/realms/{}/users", realm))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;
            let me_body: Value = me_resp.json();
            let users = me_body.as_array().expect("users array");
            let admin_id = users
                .iter()
                .find(|u| u["username"] == "admin")
                .and_then(|u| u["id"].as_str())
                .expect("admin user id");

            // List sessions
            let sessions_resp = srv
                .get(&format!("/realms/{}/users/{}/sessions", realm, admin_id))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;

            assert_eq!(sessions_resp.status_code(), 200);
            let body: Value = sessions_resp.json();
            assert!(body["data"].is_array());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn admin_can_revoke_own_session() {
        rt().block_on(async {
            let srv = server();
            let realm = &ctx().realm_name;
            let token = login(&srv, realm).await;

            // Get admin user id
            let me_resp = srv
                .get(&format!("/realms/{}/users", realm))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;
            let me_body: Value = me_resp.json();
            let users = me_body.as_array().expect("users array");
            let admin_id = users
                .iter()
                .find(|u| u["username"] == "admin")
                .and_then(|u| u["id"].as_str())
                .expect("admin user id");

            // List sessions
            let sessions_resp = srv
                .get(&format!("/realms/{}/users/{}/sessions", realm, admin_id))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;
            assert_eq!(sessions_resp.status_code(), 200);
            let sessions_body: Value = sessions_resp.json();
            let sessions = sessions_body["data"].as_array().expect("sessions array");

            if let Some(session) = sessions.first() {
                let session_id = session["id"].as_str().expect("session id");

                let revoke_resp = srv
                    .delete(&format!(
                        "/realms/{}/users/{}/sessions/{}",
                        realm, admin_id, session_id
                    ))
                    .add_header(
                        "Authorization",
                        format!("Bearer {}", token).parse().unwrap(),
                    )
                    .await;

                // Revoke is 204 No Content
                assert_eq!(revoke_resp.status_code(), 204);
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_management_test -- --ignored"]
    fn unauthenticated_request_returns_401() {
        rt().block_on(async {
            let srv = server();
            let realm = &ctx().realm_name;

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
        rt().block_on(async {
            let srv = server();
            let realm = &ctx().realm_name;
            let token = login(&srv, realm).await;

            // Get admin user id
            let me_resp = srv
                .get(&format!("/realms/{}/users", realm))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;
            let me_body: Value = me_resp.json();
            let users = me_body.as_array().expect("users array");
            let admin_id = users
                .iter()
                .find(|u| u["username"] == "admin")
                .and_then(|u| u["id"].as_str())
                .expect("admin user id");

            let fake_session_id = Uuid::new_v4();
            let revoke_resp = srv
                .delete(&format!(
                    "/realms/{}/users/{}/sessions/{}",
                    realm, admin_id, fake_session_id
                ))
                .add_header(
                    "Authorization",
                    format!("Bearer {}", token).parse().unwrap(),
                )
                .await;

            assert_eq!(revoke_resp.status_code(), 404);
        });
    }
}
