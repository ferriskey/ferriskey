/// Integration tests for the Bearer-authenticated `/me/*` self-service endpoints
/// and the recovery-code password reset.
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test me_self_service_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
///
/// ### Why a shared runtime and not `#[tokio::test]`?
///
/// `ferriskey_api::application::http::server::http_server::router` installs a
/// process-wide metrics recorder that panics on a second call (see
/// `device_flow_test.rs`). A single long-lived `tokio::runtime::Runtime`
/// ensures `router()` is called exactly once.
#[cfg(test)]
mod tests {
    use std::env;

    use axum::http::HeaderValue;
    use axum_test::TestServer;
    use base32::Alphabet;
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
    use hmac::{Hmac, Mac};
    use serde_json::{Value, json};
    use sha1::Sha1;
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

    /// Best-effort schema teardown on normal process exit.
    ///
    /// The shared `OnceLock` context below is never dropped, so the one-off
    /// UUID schema would otherwise leak into the development database. An
    /// `atexit` hook runs after all tests complete and before the process
    /// exits, giving the DROP a chance to execute.
    struct SchemaCleanup {
        admin_url: String,
        schema: String,
    }

    static CLEANUP: std::sync::OnceLock<SchemaCleanup> = std::sync::OnceLock::new();

    extern "C" fn drop_test_schema() {
        let Some(cleanup) = CLEANUP.get() else {
            return;
        };
        // atexit runs after the tokio runtime has shut down, so spin up a
        // short-lived one to issue the DROP. All failures are ignored because
        // this is best-effort teardown.
        let Ok(rt) = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        else {
            return;
        };
        let _ = rt.block_on(async {
            let Ok(pool) = sqlx::PgPool::connect(&cleanup.admin_url).await else {
                return;
            };
            let _ = sqlx::Executor::execute(
                &pool,
                sqlx::query(&format!(
                    "DROP SCHEMA IF EXISTS \"{}\" CASCADE",
                    cleanup.schema
                )),
            )
            .await;
        });
    }

    /// Shared context built exactly once per process.
    struct SharedContext {
        app: std::sync::Mutex<axum::Router>,
        realm_name: String,
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
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                env::var("RUST_LOG").unwrap_or_else(|_| "warn,ferriskey_core=debug".to_string()),
            )
            .with_test_writer()
            .try_init();

        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("me_self_service_test_{}", Uuid::new_v4().simple());

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

        // Register teardown so the one-off schema is dropped when the process
        // exits normally (see `drop_test_schema`).
        let _ = CLEANUP.set(SchemaCleanup {
            admin_url: admin_url.clone(),
            schema: schema.clone(),
        });
        // SAFETY: `drop_test_schema` is a `#[no_mangle]`-free plain C function
        // with no arguments; registering it with atexit is valid on Unix.
        unsafe {
            libc::atexit(drop_test_schema);
        }

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
                // Non-"admin-cli" default_client_id to avoid the admin-cli seeding
                // short-circuit harness bug (#1086).
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let args = std::sync::Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
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
        format!("Bearer {token}")
            .parse()
            .expect("Bearer token must produce a valid header value")
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

    /// Create a regular user with a known password and return (user_id, username, password).
    async fn create_user(server: &TestServer, admin_token: &str) -> (String, String, String) {
        let username = format!("user-{}", Uuid::new_v4().simple());
        let password = "S3cret-Password!".to_string();

        let create_resp = server
            .post(&format!("/realms/{}/users", realm()))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": "Self",
                "lastname": "Service",
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

    /// Direct-grant login for a regular user. Returns the raw response.
    async fn login(server: &TestServer, username: &str, password: &str) -> axum_test::TestResponse {
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

    /// Compute a TOTP code for a base32 secret (SHA1, 6 digits, 30s period),
    /// mirroring `generate_totp_code` in `core/src/domain/trident/services.rs`.
    fn totp_code(secret_base32: &str, counter: u64) -> String {
        let secret = base32::decode(Alphabet::Rfc4648 { padding: false }, secret_base32)
            .expect("valid base32 secret");
        type HmacSha1 = Hmac<Sha1>;
        let mut mac = HmacSha1::new_from_slice(&secret).expect("hmac key");
        mac.update(&counter.to_be_bytes());
        let result = mac.finalize().into_bytes();
        let offset = (result[19] & 0xf) as usize;
        let code = ((u32::from(result[offset]) & 0x7f) << 24
            | (u32::from(result[offset + 1]) & 0xff) << 16
            | (u32::from(result[offset + 2]) & 0xff) << 8
            | (u32::from(result[offset + 3]) & 0xff))
            % 1_000_000;
        format!("{code:06}")
    }

    // -------------------------------------------------------------------------
    // Tests
    // -------------------------------------------------------------------------

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test me_self_service_test -- --ignored"]
    fn list_credentials_requires_auth_and_returns_overview() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_user_id, username, password) = create_user(&server, &admin_token).await;
            let user_token =
                login(&server, &username, &password).await.json::<Value>()["access_token"]
                    .as_str()
                    .expect("user access_token")
                    .to_string();

            // No token → 401.
            let unauthorized = server
                .get(&format!("/realms/{}/me/credentials", realm()))
                .await;
            assert_eq!(unauthorized.status_code(), 401);

            // With token → 200 and at least the password credential is present.
            let resp = server
                .get(&format!("/realms/{}/me/credentials", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .await;
            assert_eq!(
                resp.status_code(),
                200,
                "list credentials failed: {}",
                resp.text()
            );
            let body: Value = resp.json();
            let data = body["data"].as_array().expect("data array");
            assert!(
                data.iter().any(|c| c["credential_type"] == "password"),
                "expected a password credential in overview: {data:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test me_self_service_test -- --ignored"]
    fn reauthenticate_accepts_correct_password_and_rejects_wrong() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_user_id, username, password) = create_user(&server, &admin_token).await;
            let user_token =
                login(&server, &username, &password).await.json::<Value>()["access_token"]
                    .as_str()
                    .expect("user access_token")
                    .to_string();

            let ok = server
                .post(&format!("/realms/{}/me/reauthenticate", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .json(&json!({ "password": password }))
                .await;
            assert_eq!(ok.status_code(), 200, "reauth failed: {}", ok.text());

            let bad = server
                .post(&format!("/realms/{}/me/reauthenticate", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .json(&json!({ "password": "wrong-password" }))
                .await;
            assert_eq!(bad.status_code(), 401);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test me_self_service_test -- --ignored"]
    fn totp_setup_then_verify_stores_credential() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_user_id, username, password) = create_user(&server, &admin_token).await;
            let user_token =
                login(&server, &username, &password).await.json::<Value>()["access_token"]
                    .as_str()
                    .expect("user access_token")
                    .to_string();

            // Setup → 200 with secret + otpauth_uri.
            let setup = server
                .post(&format!("/realms/{}/me/totp/setup", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .await;
            assert_eq!(
                setup.status_code(),
                200,
                "totp setup failed: {}",
                setup.text()
            );
            let setup_body: Value = setup.json();
            let secret = setup_body["secret"]
                .as_str()
                .expect("totp secret")
                .to_string();
            assert!(
                setup_body["otpauth_url"]
                    .as_str()
                    .expect("otpauth_url in TOTP setup response")
                    .starts_with("otpauth://totp/")
            );

            // Verify now requires a step-up token minted by /me/reauthenticate
            // and only the code (the secret is persisted server-side at setup).
            let reauth = server
                .post(&format!("/realms/{}/me/reauthenticate", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .json(&json!({ "password": password }))
                .await;
            assert_eq!(
                reauth.status_code(),
                200,
                "reauthenticate failed: {}",
                reauth.text()
            );
            let reauth_body: Value = reauth.json();
            let step_up_token = reauth_body["step_up_token"]
                .as_str()
                .expect("step_up_token in reauthenticate response")
                .to_string();

            // Verify with a freshly computed valid code → 200.
            let counter = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock must be after the Unix epoch")
                .as_secs()
                / 30;
            let code = totp_code(&secret, counter);
            let verify = server
                .post(&format!("/realms/{}/me/totp/verify", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .json(&json!({ "code": code, "step_up_token": step_up_token }))
                .await;
            assert_eq!(
                verify.status_code(),
                200,
                "totp verify failed: {}",
                verify.text()
            );

            // A second setup now shows the stored OTP credential.
            let list = server
                .get(&format!("/realms/{}/me/credentials", realm()))
                .add_header("Authorization", auth_header(&user_token))
                .await;
            let list_body: Value = list.json();
            let data = list_body["data"]
                .as_array()
                .expect("credentials response data must be an array");
            assert!(
                data.iter().any(|c| c["credential_type"] == "otp"),
                "expected an otp credential after verify: {data:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test me_self_service_test -- --ignored"]
    fn passkey_registration_options_returns_creation_options() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_user_id, username, password) = create_user(&server, &admin_token).await;
            let user_token =
                login(&server, &username, &password).await.json::<Value>()["access_token"]
                    .as_str()
                    .expect("user access_token")
                    .to_string();

            let resp = server
                .post(&format!(
                    "/realms/{}/me/passkey/registration-options",
                    realm()
                ))
                .add_header("Authorization", auth_header(&user_token))
                .await;
            assert_eq!(
                resp.status_code(),
                200,
                "passkey options failed: {}",
                resp.text()
            );
            let body: Value = resp.json();
            // The response reuses the public-key creation options envelope.
            let public_key = body
                .get("publicKey")
                .or_else(|| body.get("data"))
                .expect("passkey registration options missing publicKey/data");
            // A usable challenge must be present so the client can sign it.
            let challenge = public_key
                .get("challenge")
                .and_then(|c| c.as_str())
                .unwrap_or_default();
            assert!(
                !challenge.is_empty(),
                "passkey registration options missing a usable challenge"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test me_self_service_test -- --ignored"]
    fn recovery_code_reset_rejects_invalid_code() {
        let server = make_server();
        rt().block_on(async {
            let admin_token = get_admin_token(&server).await;
            let (_user_id, username, _password) = create_user(&server, &admin_token).await;

            let resp = server
                .post(&format!(
                    "/realms/{}/login-actions/reset-password-with-recovery-code",
                    realm()
                ))
                .json(&json!({
                    "email": format!("{}@test.local", username),
                    // Valid b32-split-4 formatting but not a code this user owns,
                    // so the request reaches recovery-code comparison and is
                    // rejected as an unmatched code (HTTP 400).
                    "recovery_code": "aaaa-bbbb-cccc-dddd",
                    "recovery_code_format": "b32-split-4",
                    "new_password": "New-Str0ng!P@ssword#2024",
                }))
                .await;
            // No matching recovery code → 400 (BadRequest via RecoveryCodeBurnError).
            let status = resp.status_code().as_u16();
            assert_eq!(
                status,
                400,
                "expected 400 for unmatched recovery code, got {}: {}",
                status,
                resp.text()
            );
        });
    }
}
