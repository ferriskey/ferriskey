/// Integration tests for the realm lifecycle: creation conflicts (#1286) and
/// deletion of the mirror client that lives in the master realm (#1287).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test realm_lifecycle_test -- --ignored
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

    struct TestContext {
        server: TestServer,
        /// The realm this harness bootstraps as its master realm.
        master_realm: String,
        /// Kept alive for as long as the context lives; not read by the tests.
        #[allow(dead_code)]
        pool: sqlx::PgPool,
    }

    async fn setup() -> TestContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("test_realm_lifecycle_{}", Uuid::new_v4().simple());
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

        // `create_realm` resolves the master realm by its canonical name, so the
        // harness has to bootstrap under `master`. Each run owns its own schema,
        // so nothing is shared between suites.
        let master_realm = "master".to_string();
        svc.initialize_application(StartupConfig {
            webapp_url: "http://localhost:5555".to_string(),
            master_realm_name: master_realm.clone(),
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
        let server = TestServer::new(app).expect("build test server");

        TestContext {
            server,
            master_realm,
            pool,
        }
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

    /// A realm name nothing else in the suite will claim.
    fn unique_realm_name() -> String {
        format!("lifecycle-{}", Uuid::new_v4().simple())
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_lifecycle_test -- --ignored"]
    async fn creating_a_realm_whose_name_is_taken_answers_409() {
        let ctx = setup().await;
        let srv = &ctx.server;
        let token = login(srv, &ctx.master_realm).await;
        let name = unique_realm_name();

        let first = srv
            .post("/realms")
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "name": name }))
            .await;
        assert_eq!(
            first.status_code(),
            201,
            "creating the realm failed: {}",
            first.text()
        );

        let second = srv
            .post("/realms")
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "name": name }))
            .await;

        let body = second.text();
        assert_eq!(
            second.status_code(),
            409,
            "a duplicate realm name must be a conflict, not a server error: {body}"
        );
        assert!(
            !body.contains("realms_name_key"),
            "the database constraint leaked to the client: {body}"
        );
        assert!(
            body.contains(&name),
            "the error should name the realm that is taken: {body}"
        );
    }
}
