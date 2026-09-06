/// Integration tests for the self-service account profile (#995, first
/// iteration): `GET/PUT /realms/{realm}/me`.
///
/// Each test acts through its own dedicated user (created via the admin API,
/// never the shared `admin` account) so the tests stay parallel-safe against
/// the one realm the shared harness provisions.
///
/// Require a running PostgreSQL instance. Marked `#[ignore]` so they don't
/// block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test me_profile_test -- --ignored
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

    const ADMIN_PASSWORD: &str = "admin_pass_1234!";
    const TEST_USER_PASSWORD: &str = "S3cret-Password!";

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

        let schema = format!("test_me_profile_{}", Uuid::new_v4().simple());
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
            admin_password: ADMIN_PASSWORD.to_string(),
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

    async fn login(srv: &TestServer, realm: &str, username: &str, password: &str) -> String {
        let resp = srv
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
                ("scope", "openid profile"),
            ])
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "password grant failed for {username}: {}",
            resp.text()
        );

        resp.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    async fn admin_token(srv: &TestServer, realm: &str) -> String {
        login(srv, realm, "admin", ADMIN_PASSWORD).await
    }

    /// Creates a dedicated, isolated user (own username/email) and returns
    /// its access token — never the shared `admin` account, so tests mutating
    /// their own profile can run in parallel against the one shared realm.
    async fn create_test_user_and_login(
        srv: &TestServer,
        realm: &str,
        admin_token: &str,
    ) -> String {
        let username = format!("me-test-{}", Uuid::new_v4().simple());

        let create_resp = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&serde_json::json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            create_resp.status_code(),
            200,
            "user creation failed: {}",
            create_resp.text()
        );
        let user_id = create_resp.json::<Value>()["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let pw_resp = srv
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&serde_json::json!({
                "value": TEST_USER_PASSWORD,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(
            pw_resp.status_code(),
            200,
            "reset password failed: {}",
            pw_resp.text()
        );

        login(srv, realm, &username, TEST_USER_PASSWORD).await
    }

    async fn set_edit_username_enabled(
        srv: &TestServer,
        realm: &str,
        admin_token: &str,
        enabled: bool,
    ) {
        let resp = srv
            .put(&format!("/realms/{}/settings", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&serde_json::json!({ "edit_username_enabled": enabled }))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "updating realm settings failed: {}",
            resp.text()
        );
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test me_profile_test -- --ignored"]
    fn get_me_returns_the_caller_own_profile() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let token = create_test_user_and_login(&srv, &realm, &admin_token).await;

            let resp = srv
                .get(&format!("/realms/{}/users/me", realm))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(resp.status_code(), 200, "GET /me failed: {}", resp.text());
            let body: Value = resp.json();
            assert!(
                body["data"]["username"]
                    .as_str()
                    .expect("username")
                    .starts_with("me-test-"),
                "unexpected profile returned: {body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test me_profile_test -- --ignored"]
    fn put_me_updates_firstname_lastname_and_email() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let token = create_test_user_and_login(&srv, &realm, &admin_token).await;
            let new_email = format!("updated-{}@ferriskey.test", Uuid::new_v4().simple());

            let update = srv
                .put(&format!("/realms/{}/users/me", realm))
                .add_header("Authorization", auth_header(&token))
                .json(&serde_json::json!({
                    "firstname": "Updated",
                    "lastname": "Name",
                    "email": new_email,
                }))
                .await;
            assert_eq!(
                update.status_code(),
                200,
                "PUT /me failed: {}",
                update.text()
            );
            let updated: Value = update.json();
            assert_eq!(updated["data"]["firstname"], "Updated");
            assert_eq!(updated["data"]["lastname"], "Name");
            assert_eq!(updated["data"]["email"], new_email);
            assert_eq!(
                updated["data"]["email_verified"], false,
                "a changed email must not inherit the previous address' verified status"
            );

            let fetched = srv
                .get(&format!("/realms/{}/users/me", realm))
                .add_header("Authorization", auth_header(&token))
                .await;
            let fetched: Value = fetched.json();
            assert_eq!(fetched["data"]["email"], new_email);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test me_profile_test -- --ignored"]
    fn username_edit_is_refused_unless_the_realm_allows_it() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let token = create_test_user_and_login(&srv, &realm, &admin_token).await;
            let new_username = format!("renamed-{}", Uuid::new_v4().simple());

            // Default: username editing is disabled.
            let refused = srv
                .put(&format!("/realms/{}/users/me", realm))
                .add_header("Authorization", auth_header(&token))
                .json(&serde_json::json!({ "username": new_username }))
                .await;
            assert_eq!(
                refused.status_code(),
                403,
                "username editing must be refused by default: {}",
                refused.text()
            );

            set_edit_username_enabled(&srv, &realm, &admin_token, true).await;

            let allowed = srv
                .put(&format!("/realms/{}/users/me", realm))
                .add_header("Authorization", auth_header(&token))
                .json(&serde_json::json!({ "username": new_username }))
                .await;
            assert_eq!(
                allowed.status_code(),
                200,
                "username editing must succeed once the realm allows it: {}",
                allowed.text()
            );
            let body: Value = allowed.json();
            assert_eq!(body["data"]["username"], new_username);

            // Restore the realm-wide setting for any test still running against it.
            set_edit_username_enabled(&srv, &realm, &admin_token, false).await;
        });
    }
}
