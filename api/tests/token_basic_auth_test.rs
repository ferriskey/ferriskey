/// Integration tests for token endpoint Basic Auth (RFC 6749 §2.3.1 `client_secret_basic`)
/// and the `token_endpoint_auth_methods_supported` field in the OIDC discovery document.
///
/// Run with:
///   cargo test -p ferriskey-api -- --ignored
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::http::HeaderValue;
    use axum_test::TestServer;
    use base64::{Engine, engine::general_purpose};
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

        let schema = format!("token_auth_test_{}", Uuid::new_v4().simple());

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

        service
            .initialize_application(StartupConfig {
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "admin-cli".to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::new(app).expect("create test server");

        TestContext { server, realm_name }
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

        assert_eq!(response.status_code(), 200, "token request failed");
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    async fn create_confidential_client(ctx: &TestContext, token: &str) -> (String, String) {
        let client_id = format!("conf-client-{}", Uuid::new_v4().simple());

        let response = ctx
            .server
            .post(&format!("/realms/{}/clients", ctx.realm_name))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "client_id": client_id,
                "name": "Confidential Test Client",
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": true,
                "direct_access_grants_enabled": false,
                "enabled": true,
            }))
            .await;

        assert_eq!(response.status_code(), 201, "client creation failed");
        let body: Value = response.json();
        let secret = body["secret"]
            .as_str()
            .expect("confidential client has a secret")
            .to_string();
        (client_id, secret)
    }

    fn basic_auth_value(client_id: &str, client_secret: &str) -> HeaderValue {
        let encoded = general_purpose::STANDARD.encode(format!("{client_id}:{client_secret}"));
        format!("Basic {encoded}").parse().unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn token_endpoint_accepts_client_secret_basic() {
        let ctx = setup().await;
        let admin_token = get_admin_token(&ctx).await;
        let (client_id, secret) = create_confidential_client(&ctx, &admin_token).await;

        let response = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
            ))
            .add_header("Authorization", basic_auth_value(&client_id, &secret))
            .form(&[("grant_type", "client_credentials")])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "client_secret_basic should succeed, got body: {}",
            response.text()
        );
        let body: Value = response.json();
        assert!(body["access_token"].is_string());
    }

    #[tokio::test]
    #[ignore]
    async fn token_endpoint_still_accepts_client_secret_post() {
        let ctx = setup().await;
        let admin_token = get_admin_token(&ctx).await;
        let (client_id, secret) = create_confidential_client(&ctx, &admin_token).await;

        let response = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
            ))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client_id.as_str()),
                ("client_secret", secret.as_str()),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "client_secret_post should still work, got body: {}",
            response.text()
        );
        let body: Value = response.json();
        assert!(body["access_token"].is_string());
    }

    #[tokio::test]
    #[ignore]
    async fn discovery_advertises_token_endpoint_auth_methods() {
        let ctx = setup().await;

        let response = ctx
            .server
            .get(&format!(
                "/realms/{}/.well-known/openid-configuration",
                ctx.realm_name
            ))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        let methods = body["token_endpoint_auth_methods_supported"]
            .as_array()
            .expect("token_endpoint_auth_methods_supported is an array");
        let methods: Vec<&str> = methods.iter().filter_map(|v| v.as_str()).collect();
        assert!(methods.contains(&"client_secret_basic"));
        assert!(methods.contains(&"client_secret_post"));
    }
}
