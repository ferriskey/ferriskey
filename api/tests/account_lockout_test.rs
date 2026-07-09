/// Integration tests for account lockout / anti-brute-force (issue #1095).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test account_lockout_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
///
/// These tests verify:
///   1. N-1 failed attempts do NOT lock the account.
///   2. Nth failed attempt locks the account (subsequent login returns 401 with
///      the "locked" message even with correct credentials).
///   3. Admin unlock endpoint clears the lockout → login succeeds.
///   4. Auto-recovery: locked_until in the past allows login again.
///
/// The shared-runtime / single-`router()` harness mirrors `device_flow_test.rs`
/// (see #1086 for why a process-wide runtime and one `router()` call are used).
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

    /// Lockout threshold configured on the shared test realm. Kept low so a
    /// handful of failed attempts trips the lock.
    const LOCKOUT_THRESHOLD: i32 = 3;
    const LOCKOUT_DURATION_SECONDS: i32 = 900;

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
        /// `!Sync` router guarded by a `Mutex` so it can live in a `OnceLock`.
        app: std::sync::Mutex<Router>,
        realm_name: String,
        /// Schema-scoped pool for direct SQL (setting lockout config, forcing
        /// auto-recovery by rewinding `locked_until`).
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

    /// Returns the shared context, initialising it on first call.
    ///
    /// Must be called from a synchronous context (not inside `rt().block_on()`)
    /// because init itself calls `rt().block_on(init_shared_ctx())`.
    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("account_lockout_test_{}", Uuid::new_v4().simple());

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

        // Non-"admin-cli" default_client_id so the dedicated admin-cli seeding
        // path is not short-circuited (see #1086 and device_flow_test.rs).
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

        // Lower the lockout threshold for the whole realm so a few failed
        // attempts trip the lock. Constants (not user input) → safe to inline.
        pool.execute(sqlx::query(&format!(
            "UPDATE realm_settings SET lockout_threshold = {}, lockout_duration_seconds = {}",
            LOCKOUT_THRESHOLD, LOCKOUT_DURATION_SECONDS
        )))
        .await
        .expect("configure lockout threshold");

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

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    async fn get_admin_token(server: &TestServer) -> String {
        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
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

    /// Create a fresh victim user with a known password. Returns `(user_id,
    /// username, password)`. Each test uses its own user so the per-user
    /// `failed_login_attempts` counter never bleeds across parallel tests.
    async fn create_victim(server: &TestServer, admin_token: &str) -> (String, String, String) {
        let suffix = Uuid::new_v4().simple().to_string();
        let username = format!("victim-{}", suffix);
        let password = "S3cret-Password!".to_string();

        let create_resp = server
            .post(&format!("/realms/{}/users", realm()))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": "Victim",
                "lastname": "User",
                "email": format!("{}@test.local", username),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            create_resp.status_code(),
            200,
            "create user failed: {}",
            create_resp.text()
        );
        let created: Value = create_resp.json();
        let user_id = created["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let pw_resp = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm(),
                user_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "value": password,
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

        (user_id, username, password)
    }

    /// Attempt a direct-grant login for `username`/`password`. Returns the raw
    /// response so callers can assert status and body.
    async fn login_attempt(server: &TestServer, username: &str, password: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await
    }

    fn is_locked_response(resp: &TestResponse) -> bool {
        if resp.status_code() != 401 {
            return false;
        }
        let body: Value = resp.json();
        body["message"]
            .as_str()
            .map(|m| m.to_lowercase().contains("locked"))
            .unwrap_or(false)
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    /// N-1 wrong-password attempts do not lock the account: the correct
    /// password still authenticates.
    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    fn below_threshold_does_not_lock() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_id, username, password) = create_victim(&server, &admin_token).await;

            // threshold - 1 failures must not lock the account.
            for i in 0..(LOCKOUT_THRESHOLD - 1) {
                let resp = login_attempt(&server, &username, "wrong-password").await;
                assert!(
                    !is_locked_response(&resp),
                    "attempt {} should not report a lock yet: {}",
                    i + 1,
                    resp.text()
                );
            }

            let ok = login_attempt(&server, &username, &password).await;
            assert_eq!(
                ok.status_code(),
                200,
                "correct password below threshold must succeed: {}",
                ok.text()
            );
            let body: Value = ok.json();
            assert!(
                body["access_token"].as_str().is_some(),
                "expected an access_token in the success response: {body}"
            );
        });
    }

    /// The Nth wrong-password attempt locks the account: the correct password
    /// then returns 401 with the lockout message.
    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    fn at_threshold_locks_account() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_id, username, password) = create_victim(&server, &admin_token).await;

            for _ in 0..LOCKOUT_THRESHOLD {
                let _ = login_attempt(&server, &username, "wrong-password").await;
            }

            let locked = login_attempt(&server, &username, &password).await;
            assert!(
                is_locked_response(&locked),
                "correct password after {} failures must be rejected as locked, got {}: {}",
                LOCKOUT_THRESHOLD,
                locked.status_code(),
                locked.text()
            );
        });
    }

    /// The admin unlock endpoint clears the lock and restores access.
    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    fn admin_unlock_restores_access() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (user_id, username, password) = create_victim(&server, &admin_token).await;

            for _ in 0..LOCKOUT_THRESHOLD {
                let _ = login_attempt(&server, &username, "wrong-password").await;
            }
            assert!(
                is_locked_response(&login_attempt(&server, &username, &password).await),
                "account should be locked before unlock"
            );

            let unlock = server
                .post(&format!("/realms/{}/users/{}/unlock", realm(), user_id))
                .add_header("Authorization", auth_header(&admin_token))
                .await;
            assert_eq!(
                unlock.status_code(),
                204,
                "unlock should return 204: {}",
                unlock.text()
            );

            let after = login_attempt(&server, &username, &password).await;
            assert_eq!(
                after.status_code(),
                200,
                "login after admin unlock must succeed: {}",
                after.text()
            );
        });
    }

    /// Once `locked_until` lapses, the account auto-recovers without an admin
    /// action. We simulate the elapsed window by rewinding `locked_until`.
    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_lockout_test -- --ignored"]
    fn auto_recovery_after_window_elapses() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (user_id, username, password) = create_victim(&server, &admin_token).await;

            for _ in 0..LOCKOUT_THRESHOLD {
                let _ = login_attempt(&server, &username, "wrong-password").await;
            }
            assert!(
                is_locked_response(&login_attempt(&server, &username, &password).await),
                "account should be locked before the window elapses"
            );

            // Rewind the lock into the past to simulate an elapsed window.
            let uid = Uuid::parse_str(&user_id).expect("victim id is a uuid");
            sqlx::query(&format!(
                "UPDATE users SET locked_until = now() - interval '1 minute' WHERE id = '{}'",
                uid
            ))
            .execute(&shared_ctx().pool)
            .await
            .expect("rewind locked_until");

            let after = login_attempt(&server, &username, &password).await;
            assert_eq!(
                after.status_code(),
                200,
                "login after the lockout window elapses must succeed: {}",
                after.text()
            );
        });
    }
}
