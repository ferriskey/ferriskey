/// Integration test for the realm branding HTTP endpoints.
///
/// This test requires a running PostgreSQL instance.  It is marked `#[ignore]`
/// so it does not block regular `cargo test` runs.  Run it explicitly with:
///
///   cargo test -p ferriskey-api --test realm_branding_test -- --ignored
///
/// All assertions are bundled into a single test because `axum-prometheus`
/// installs a process-global metrics recorder, which prevents building more
/// than one router per test binary.
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

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn realm_branding_endpoints_full_lifecycle() {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("branding_test_{}", Uuid::new_v4().simple());

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

        // --- Admin endpoint requires auth.
        let unauth = server
            .get(&format!("/realms/{}/branding", realm_name))
            .await;
        assert_ne!(
            unauth.status_code(),
            200,
            "admin GET must reject unauthenticated requests"
        );

        // --- Get an admin token.
        let token_response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;
        assert_eq!(token_response.status_code(), 200, "token request failed");
        let token_body: Value = token_response.json();
        let token = token_body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string();

        // --- GET branding returns defaults when nothing has been stored.
        let initial_get = server
            .get(&format!("/realms/{}/branding", realm_name))
            .add_header("Authorization", auth_header(&token))
            .await;
        assert_eq!(initial_get.status_code(), 200);
        let initial_body: Value = initial_get.json();
        assert!(initial_body["data"]["colors"]["primaryButton"].is_string());
        assert!(initial_body["data"]["fonts"]["baseSize"].is_number());
        assert!(initial_body["data"]["widget"]["headerAlignment"].is_string());

        // --- PUT then GET round-trips a partial config.
        let put_response = server
            .put(&format!("/realms/{}/branding", realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "config": {
                    "colors": { "primaryButton": "#ff00aa" },
                    "widget": {
                        "logoUrl": "https://example.test/logo.png",
                        "logoHeight": 80
                    }
                }
            }))
            .await;
        assert_eq!(put_response.status_code(), 200);
        let put_body: Value = put_response.json();
        assert_eq!(
            put_body["data"]["config"]["colors"]["primaryButton"],
            "#ff00aa"
        );

        let after_put_get = server
            .get(&format!("/realms/{}/branding", realm_name))
            .add_header("Authorization", auth_header(&token))
            .await;
        assert_eq!(after_put_get.status_code(), 200);
        let after_put_body: Value = after_put_get.json();
        assert_eq!(after_put_body["data"]["colors"]["primaryButton"], "#ff00aa");
        assert_eq!(
            after_put_body["data"]["widget"]["logoUrl"],
            "https://example.test/logo.png"
        );
        assert_eq!(after_put_body["data"]["widget"]["logoHeight"], 80);

        // --- A second PUT overwrites the existing row (no duplicate inserted).
        let overwrite = server
            .put(&format!("/realms/{}/branding", realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "config": { "colors": { "primaryButton": "#222222" } } }))
            .await;
        assert_eq!(overwrite.status_code(), 200);

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM realm_branding")
            .fetch_one(&pool)
            .await
            .expect("count branding rows");
        assert_eq!(
            count.0, 1,
            "upsert must update in place rather than insert duplicates"
        );

        // --- Public endpoint serves the saved config without authentication.
        let public_response = server
            .get(&format!("/realms/{}/branding/public", realm_name))
            .await;
        assert_eq!(public_response.status_code(), 200);
        let public_body: Value = public_response.json();
        assert_eq!(public_body["data"]["colors"]["primaryButton"], "#222222");

        // --- OpenAPI exposes the new endpoints.
        let openapi_response = server.get("/api-docs/openapi.json").await;
        assert_eq!(openapi_response.status_code(), 200);
        let openapi_body: Value = openapi_response.json();
        let paths = &openapi_body["paths"];
        assert!(paths["/realms/{realm_name}/branding"].is_object());
        assert!(paths["/realms/{realm_name}/branding/public"].is_object());

        // --- Cleanup.
        admin_pool
            .execute(sqlx::query(&format!(
                "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
                schema
            )))
            .await
            .expect("drop schema");
    }
}
