/// Integration tests for rate limiting on authentication endpoints.
///
/// These tests require a running PostgreSQL instance.  They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs.  Run them explicitly with:
///
///   cargo test -p ferriskey-api -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
#[cfg(test)]
mod tests {
    use std::{env, net::SocketAddr, sync::Arc};

    use axum_test::TestServer;
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::{Args, RateLimitArgs},
    };
    use ferriskey_core::{
        application::create_service,
        domain::common::{
            DatabaseConfig, FerriskeyConfig, entities::StartupConfig, ports::CoreService,
        },
    };
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

    /// Build a test server with a very low rate limit so we can trigger 429
    /// responses quickly. The `auth_rate_limit_per_minute` is set to 2 and
    /// `auth_rate_limit_burst` to 1, meaning the second rapid-fire request
    /// to a rate-limited endpoint should already be rejected.
    async fn setup_with_rate_limit() -> TestContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("rl_test_{}", Uuid::new_v4().simple());

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

        let args = Arc::new(Args {
            rate_limit: RateLimitArgs {
                auth_rate_limit_per_minute: 2,
                auth_rate_limit_burst: 1,
            },
            ..Args::default()
        });

        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::builder()
            .http_transport()
            .build(app.into_make_service_with_connect_info::<SocketAddr>())
            .expect("create test server");

        TestContext { server, realm_name }
    }

    // ── Rate-limited endpoint: POST /realms/{realm}/protocol/openid-connect/token ──

    #[tokio::test]
    #[ignore]
    async fn token_endpoint_returns_429_when_rate_limit_exceeded() {
        let ctx = setup_with_rate_limit().await;

        let token_url = format!(
            "/realms/{}/protocol/openid-connect/token",
            ctx.realm_name
        );

        // The first request should succeed (or fail with 400/401 due to missing
        // credentials — the important thing is it is NOT 429).
        let first = ctx
            .server
            .post(&token_url)
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_ne!(
            first.status_code().as_u16(),
            429,
            "first request should not be rate-limited"
        );

        // Fire additional requests to exceed the burst limit. With burst=1 and
        // rate=2/min the second or third request must be rejected.
        let mut saw_429 = false;
        for _ in 0..5 {
            let resp = ctx
                .server
                .post(&token_url)
                .form(&[
                    ("grant_type", "password"),
                    ("client_id", "admin-cli"),
                    ("username", "admin"),
                    ("password", "admin"),
                ])
                .await;

            if resp.status_code().as_u16() == 429 {
                saw_429 = true;
                break;
            }
        }

        assert!(
            saw_429,
            "expected at least one 429 response after exceeding rate limit"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn token_endpoint_429_includes_retry_after_header() {
        let ctx = setup_with_rate_limit().await;

        let token_url = format!(
            "/realms/{}/protocol/openid-connect/token",
            ctx.realm_name
        );

        // Exhaust the rate limit.
        for _ in 0..5 {
            ctx.server
                .post(&token_url)
                .form(&[
                    ("grant_type", "password"),
                    ("client_id", "admin-cli"),
                    ("username", "admin"),
                    ("password", "admin"),
                ])
                .await;
        }

        // Send one more request that should definitely be rate-limited.
        let resp = ctx
            .server
            .post(&token_url)
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_eq!(
            resp.status_code().as_u16(),
            429,
            "expected 429 after exhausting rate limit"
        );

        let retry_after = resp
            .headers()
            .get("retry-after")
            .expect("429 response must include a Retry-After header");

        let retry_secs: u64 = retry_after
            .to_str()
            .expect("Retry-After should be valid UTF-8")
            .parse()
            .expect("Retry-After should be a numeric value in seconds");

        assert!(
            retry_secs > 0,
            "Retry-After must be a positive number of seconds, got {}",
            retry_secs
        );
    }

    // ── Non-rate-limited endpoints ────────────────────────────────────────────────

    #[tokio::test]
    #[ignore]
    async fn certs_endpoint_is_not_rate_limited() {
        let ctx = setup_with_rate_limit().await;

        let certs_url = format!(
            "/realms/{}/protocol/openid-connect/certs",
            ctx.realm_name
        );

        // Fire many requests — they should ALL succeed (200). None should be 429.
        for i in 0..20 {
            let resp = ctx.server.get(&certs_url).await;
            assert_eq!(
                resp.status_code().as_u16(),
                200,
                "certs request #{} was unexpectedly rate-limited or failed with status {}",
                i + 1,
                resp.status_code()
            );
        }
    }

    #[tokio::test]
    #[ignore]
    async fn well_known_openid_configuration_is_not_rate_limited() {
        let ctx = setup_with_rate_limit().await;

        let well_known_url = format!(
            "/realms/{}/.well-known/openid-configuration",
            ctx.realm_name
        );

        // Fire many requests — they should ALL succeed (200). None should be 429.
        for i in 0..20 {
            let resp = ctx.server.get(&well_known_url).await;
            assert_eq!(
                resp.status_code().as_u16(),
                200,
                "well-known request #{} was unexpectedly rate-limited or failed with status {}",
                i + 1,
                resp.status_code()
            );
        }
    }
}
