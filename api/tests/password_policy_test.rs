/// Integration tests for per-realm password policy enforcement.
///
/// Requires a running Postgres instance. Run with:
///   cargo test -p ferriskey-api --test password_policy_test -- --ignored
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

        let schema = format!("pw_policy_test_{}", Uuid::new_v4().simple());

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

    /// Verifies:
    ///  1. Admin can configure a strict password policy.
    ///  2. The public endpoint returns the policy without auth.
    ///  3. Resetting a user password with a weak value is rejected (422).
    ///  4. Resetting with a strong value succeeds (200).
    ///
    /// Note: Registration enforcement shares the same `validate_password_policy`
    /// seam as the other flows; it is not separately exercised here to avoid
    /// the additional realm-setting setup required to enable self-registration.
    #[tokio::test]
    #[ignore]
    async fn password_policy_is_enforced_and_public() {
        let ctx = setup().await;
        let admin_token = get_admin_token(&ctx).await;

        // --- 1. Set a strict policy ---
        let policy_response = ctx
            .server
            .put(&format!("/realms/{}/password-policy", ctx.realm_name))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({
                "min_length": 10,
                "require_uppercase": true,
                "require_lowercase": true,
                "require_number": true,
                "require_special": true,
                // This test exercises character-class + length enforcement only.
                // Disable the entropy/common-password rules (which default to the
                // strict CNIL values) so the configured policy matches what the
                // assertions below expect.
                "min_entropy_bits": 0,
                "forbid_common": false,
                "check_breached": false,
            }))
            .await;

        assert_eq!(
            policy_response.status_code(),
            200,
            "PUT password-policy failed: {}",
            policy_response.text()
        );

        // --- 2. Public endpoint returns the policy without auth ---
        let public_response = ctx
            .server
            .get(&format!(
                "/realms/{}/password-policy/public",
                ctx.realm_name
            ))
            .await;

        assert_eq!(
            public_response.status_code(),
            200,
            "GET password-policy/public failed: {}",
            public_response.text()
        );
        let public_body: Value = public_response.json();
        assert_eq!(
            public_body["min_length"].as_i64(),
            Some(10),
            "expected min_length == 10"
        );
        assert_eq!(
            public_body["require_uppercase"].as_bool(),
            Some(true),
            "expected require_uppercase == true"
        );

        // --- 3. Create a test user so we can reset their password ---
        let user_response = ctx
            .server
            .post(&format!("/realms/{}/users", ctx.realm_name))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({
                "username": format!("testuser-{}", Uuid::new_v4().simple()),
                "email": format!("test-{}@example.com", Uuid::new_v4().simple()),
            }))
            .await;

        let user_status = user_response.status_code();
        assert!(
            user_status == 200 || user_status == 201,
            "user creation failed ({}): {}",
            user_status,
            user_response.text()
        );
        // The user-creation endpoint wraps its payload in a `data` envelope.
        let user_body: Value = user_response.json();
        let user_id = user_body["data"]["id"]
            .as_str()
            .expect("user id in response")
            .to_string();

        // --- 4. Weak password is rejected ---
        let weak_reset = ctx
            .server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                ctx.realm_name, user_id
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({ "value": "weak", "temporary": false }))
            .await;

        assert_eq!(
            weak_reset.status_code(),
            422,
            "weak password should be rejected (422), got: {}",
            weak_reset.text()
        );

        // --- 5. Strong password succeeds ---
        let strong_reset = ctx
            .server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                ctx.realm_name, user_id
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({ "value": "StrongPass1!", "temporary": false }))
            .await;

        assert_eq!(
            strong_reset.status_code(),
            200,
            "strong password should succeed, got: {}",
            strong_reset.text()
        );
    }
}
