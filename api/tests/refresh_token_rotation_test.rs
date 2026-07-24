/// Integration tests for refresh token rotation (RFC 6749 + rotation security).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test refresh_token_rotation_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
///
/// See `device_flow_test.rs` for the shared-runtime rationale.
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

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

    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("rt_rotation_test_{}", Uuid::new_v4().simple());

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
            encryption: ferriskey_core::domain::common::EncryptionConfig::default(),
        })
        .await
        .expect("create service");

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());

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

    fn make_server() -> TestServer {
        let ctx = shared_ctx();
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
    }

    fn realm() -> &'static str {
        shared_ctx().realm_name.as_str()
    }

    async fn get_password_token(server: &TestServer, client_id: &str) -> Value {
        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", client_id),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "password token request failed: {}",
            response.text()
        );
        response.json::<Value>()
    }

    async fn do_refresh(server: &TestServer, client_id: &str, refresh_token: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", client_id),
                ("refresh_token", refresh_token),
            ])
            .await
    }

    /// Happy path: each refresh returns a NEW refresh token; the old one is invalidated.
    #[test]
    #[ignore]
    fn test_refresh_token_rotates_on_each_use() {
        rt().block_on(async {
            let server = make_server();

            // Initial token set via password grant
            let tokens = get_password_token(&server, "admin-cli").await;
            let rt1 = tokens["refresh_token"]
                .as_str()
                .expect("refresh_token")
                .to_string();

            // First refresh → get rt2
            let resp2 = do_refresh(&server, "admin-cli", &rt1).await;
            assert_eq!(
                resp2.status_code(),
                200,
                "first refresh failed: {}",
                resp2.text()
            );
            let tokens2: Value = resp2.json();
            let rt2 = tokens2["refresh_token"].as_str().expect("rt2").to_string();

            assert_ne!(rt1, rt2, "refresh tokens must differ after rotation");

            // rt1 is now rotated — replaying it must fail
            let replay = do_refresh(&server, "admin-cli", &rt1).await;
            assert_eq!(
                replay.status_code(),
                400,
                "replaying rotated token should fail, got: {}",
                replay.text()
            );
            let replay_body: Value = replay.json();
            assert_eq!(replay_body["error"].as_str(), Some("invalid_grant"));

            // rt2 should also now be revoked (family revocation triggered by rt1 replay)
            let rt2_try = do_refresh(&server, "admin-cli", &rt2).await;
            assert_eq!(
                rt2_try.status_code(),
                400,
                "rt2 should be revoked after family revocation, got: {}",
                rt2_try.text()
            );
        });
    }

    /// Second refresh in the same lineage: only the latest token is valid.
    #[test]
    #[ignore]
    fn test_chained_refresh_rotation() {
        rt().block_on(async {
            let server = make_server();

            let tokens = get_password_token(&server, "admin-cli").await;
            let rt1 = tokens["refresh_token"].as_str().expect("rt1").to_string();

            // rt1 → rt2
            let resp2 = do_refresh(&server, "admin-cli", &rt1).await;
            assert_eq!(resp2.status_code(), 200);
            let rt2 = resp2.json::<Value>()["refresh_token"]
                .as_str()
                .expect("rt2")
                .to_string();

            // rt2 → rt3
            let resp3 = do_refresh(&server, "admin-cli", &rt2).await;
            assert_eq!(resp3.status_code(), 200);
            let rt3 = resp3.json::<Value>()["refresh_token"]
                .as_str()
                .expect("rt3")
                .to_string();

            assert_ne!(rt2, rt3);

            // Old token rt2 is now rotated — must be rejected
            let old_replay = do_refresh(&server, "admin-cli", &rt2).await;
            assert_eq!(old_replay.status_code(), 400);

            // rt3 (the current one) should now be revoked too (family revocation)
            let rt3_try = do_refresh(&server, "admin-cli", &rt3).await;
            assert_eq!(rt3_try.status_code(), 400);
        });
    }
}
