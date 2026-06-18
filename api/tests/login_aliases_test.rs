/// Integration tests for login_aliases configuration via the realm settings API.
///
/// Requires a running Postgres instance. Run with:
///   cargo test -p ferriskey-api --test login_aliases_test -- --ignored
///
/// Environment variables (all optional, defaults shown):
///   DATABASE_HOST=localhost  DATABASE_PORT=5432
///   DATABASE_NAME=ferriskey  DATABASE_USER=ferriskey  DATABASE_PASSWORD=ferriskey
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
        realm_name: String,
    }

    async fn setup() -> TestContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("login_aliases_test_{}", Uuid::new_v4().simple());

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
                // Use "ferriskey-admin" (not "admin-cli") to avoid the known
                // harness bug (#1086) that breaks the admin password grant.
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::new(app).expect("create test server");

        TestContext { server, realm_name }
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    async fn get_admin_token(ctx: &TestContext) -> String {
        let response = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
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

    /// PUT /realms/{name}/settings with login_aliases=["email","username"] succeeds (200),
    /// and GET /realms/{name}/login-settings returns login_aliases == ["email","username"].
    #[tokio::test]
    #[ignore = "requires it-harness fix, issue #1086"]
    async fn put_settings_accepts_email_alias_and_get_login_settings_returns_it() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let put_response = ctx
            .server
            .put(&format!("/realms/{}/settings", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "login_aliases": ["email", "username"] }))
            .await;

        assert_eq!(
            put_response.status_code(),
            200,
            "PUT /settings failed: {}",
            put_response.text()
        );

        let get_response = ctx
            .server
            .get(&format!("/realms/{}/login-settings", ctx.realm_name))
            .await;

        assert_eq!(
            get_response.status_code(),
            200,
            "GET /login-settings failed: {}",
            get_response.text()
        );

        let body: Value = get_response.json();
        let aliases = body["login_aliases"]
            .as_array()
            .expect("login_aliases should be an array");
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0].as_str(), Some("email"));
        assert_eq!(aliases[1].as_str(), Some("username"));
    }

    /// PUT /realms/{name}/settings with login_aliases=[] is rejected with 400.
    #[tokio::test]
    #[ignore = "requires it-harness fix, issue #1086"]
    async fn put_settings_rejects_empty_login_aliases() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let response = ctx
            .server
            .put(&format!("/realms/{}/settings", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "login_aliases": [] }))
            .await;

        assert_eq!(
            response.status_code(),
            400,
            "expected 400 for empty login_aliases, got: {}",
            response.text()
        );
    }

    /// PUT /realms/{name}/settings with login_aliases=["phone"] is rejected with 400.
    #[tokio::test]
    #[ignore = "requires it-harness fix, issue #1086"]
    async fn put_settings_rejects_unknown_login_alias() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let response = ctx
            .server
            .put(&format!("/realms/{}/settings", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "login_aliases": ["phone"] }))
            .await;

        assert_eq!(
            response.status_code(),
            400,
            "expected 400 for unknown login alias 'phone', got: {}",
            response.text()
        );
    }
}
