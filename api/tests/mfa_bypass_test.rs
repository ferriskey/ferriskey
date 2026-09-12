/// Integration test: MFA cannot be bypassed with the password alone (FK-003).
///
/// Two independent bypasses existed, both reachable with nothing but a valid
/// password:
///
/// - **Replay of the step token.** When a user has an OTP credential, the login
///   returns `RequiresOtpChallenge` plus a signed JWT of type `Temporary`, meant to
///   be spent on `/login-actions/challenge-otp`. Replayed as
///   `Authorization: Bearer` on `/login-actions/authenticate`, it was accepted as
///   proof of a completed login and minted an authorization code:
///   `handle_token_refresh` never looked at `claims.typ`.
/// - **Caller-supplied enrolment secret.** `verify_otp` took the TOTP secret from
///   the request body and verified the submitted code against that same secret — a
///   tautology — then deleted every existing OTP credential of the user before
///   storing the caller's. `setup_otp` kept no server-side record to check against.
///
/// These tests drive the real HTTP endpoints, so they cover the guard, the session
/// checks and the enrolment state together.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]`. Run with:
///
///   cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored
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

    use axum::{Router, http::HeaderValue};
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
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    /// Base32, 20 bytes decoded — the shape `TotpSecret::from_base32` expects.
    const VICTIM_OTP_SECRET: &str = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";
    /// RFC 7636 Appendix B test vector. The seeded console client requires PKCE; the
    /// code is never exchanged here, so the matching verifier is irrelevant.
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

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
        pool: PgPool,
    }

    // `router()` installs a process-global Prometheus recorder, so it can only be
    // built once per test binary (#1086). Every test shares this router and runtime.
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
        CTX.get_or_init(|| match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(init_shared_ctx())),
            Err(_) => rt().block_on(init_shared_ctx()),
        })
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("mfa_bypass_test_{}", Uuid::new_v4().simple());

        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = PgPool::connect(&admin_url)
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

        let pool = PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");

        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let service = create_service(FerriskeyConfig {
            webapp_url: WEBAPP_URL.to_string(),
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
                webapp_url: WEBAPP_URL.to_string(),
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
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
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn realm() -> &'static str {
        shared_ctx().realm_name.as_str()
    }

    /// Give the seeded admin an OTP credential, so the login stops at the OTP
    /// challenge and hands back a `Temporary` step token. Inserted directly because
    /// no API enrols a factor on another user's behalf.
    async fn grant_otp_credential_to_admin() {
        let pool = &shared_ctx().pool;

        sqlx::query(
            // The uniqueness rule is a partial index, not a table constraint, so
            // `ON CONFLICT (user_id, credential_type)` cannot infer an arbiter here.
            "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, user_label)
             SELECT $1, 'otp', u.id, $2, '{}'::jsonb, 'test-authenticator'
             FROM users u
             WHERE u.username = 'admin'
               AND NOT EXISTS (
                 SELECT 1 FROM credentials c
                 WHERE c.user_id = u.id AND c.credential_type = 'otp'
               )",
        )
        .bind(Uuid::new_v4())
        .bind(VICTIM_OTP_SECRET)
        .execute(pool)
        .await
        .expect("insert OTP credential");
    }

    async fn count_admin_otp_credentials() -> i64 {
        let pool = &shared_ctx().pool;

        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM credentials c
             JOIN users u ON u.id = c.user_id
             WHERE u.username = 'admin' AND c.credential_type = 'otp'",
        )
        .fetch_one(pool)
        .await
        .expect("count OTP credentials")
    }

    /// Start an authorization request. The caller pulls `FERRISKEY_SESSION` off the
    /// response: naming the cookie type here would pin an `axum_test` re-export.
    async fn start_authorization(server: &TestServer) -> axum_test::TestResponse {
        let response = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm()))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .add_query_param(
                "redirect_uri",
                format!("{WEBAPP_URL}/realms/{}/authentication/callback", realm()).as_str(),
            )
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;

        let status = response.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth, got {status}: {}",
            response.text()
        );

        response
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored"]
    fn a_failed_step_does_not_burn_the_token() {
        rt().block_on(async {
            grant_otp_credential_to_admin().await;
            let server = make_server();
            let authorize = start_authorization(&server).await;

            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();
            let session_id = authorize.cookie("FERRISKEY_SESSION").value().to_string();
            let cookies =
                format!("FERRISKEY_SESSION={session_id}; FERRISKEY_LOGIN_ACTION={step_token}");

            let wrong = server
                .post(&format!("/realms/{}/login-actions/challenge-otp", realm()))
                .add_header("Cookie", HeaderValue::from_str(&cookies).unwrap())
                .json(&json!({ "code": "000000" }))
                .await;

            assert_ne!(
                wrong.status_code(),
                401,
                "the flow cookie must authenticate the call: {}",
                wrong.text()
            );
            assert_ne!(
                wrong.status_code(),
                200,
                "a wrong code must not succeed: {}",
                wrong.text()
            );

            let retry = server
                .post(&format!("/realms/{}/login-actions/challenge-otp", realm()))
                .add_header("Cookie", HeaderValue::from_str(&cookies).unwrap())
                .json(&json!({ "code": "000001" }))
                .await;

            assert_ne!(
                retry.status_code(),
                401,
                "a mistyped code must not cost the user their login flow: {}",
                retry.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored"]
    fn the_step_token_is_confined_to_an_http_only_cookie() {
        rt().block_on(async {
            grant_otp_credential_to_admin().await;
            let server = make_server();
            let authorize = start_authorization(&server).await;

            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            let body: Value = login.json();
            assert!(
                body.get("token").is_none(),
                "the step token must not travel in the response body: {body}"
            );

            let raw = login
                .headers()
                .get_all("set-cookie")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .find(|v| v.starts_with("FERRISKEY_LOGIN_ACTION="))
                .expect("the step token must be handed back as a cookie")
                .to_string();

            assert!(raw.contains("HttpOnly"), "cookie must be HttpOnly: {raw}");
            assert!(
                raw.contains("SameSite=Lax"),
                "cookie must carry the same SameSite policy as FERRISKEY_SESSION: {raw}"
            );
            assert!(
                raw.contains(&format!("Path=/realms/{}/login-actions", realm())),
                "cookie must be scoped to this realm's login actions: {raw}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored"]
    fn a_login_action_no_longer_accepts_a_bearer_header() {
        rt().block_on(async {
            grant_otp_credential_to_admin().await;
            let server = make_server();
            let authorize = start_authorization(&server).await;

            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

            let response = server
                .post(&format!("/realms/{}/login-actions/verify-otp", realm()))
                .add_header(
                    "Authorization",
                    format!("Bearer {step_token}")
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .json(&json!({ "code": "000000" }))
                .await;

            assert_eq!(
                response.status_code(),
                401,
                "a token presented outside the flow cookie must be refused: {}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored"]
    fn temporary_step_token_cannot_be_replayed_to_complete_login() {
        rt().block_on(async {
            grant_otp_credential_to_admin().await;
            let server = make_server();
            let authorize = start_authorization(&server).await;

            // Step 1 — the password alone. This is all the attacker has.
            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            assert_eq!(
                login.status_code(),
                200,
                "password step failed: {}",
                login.text()
            );

            let body: Value = login.json();
            assert_eq!(
                body["status"], "RequiresOtpChallenge",
                "the account must stop at the OTP challenge, got: {body}"
            );

            let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

            // Step 2 — replay that step token as a Bearer credential on the very same
            // endpoint. Before the fix this returned Success with an authorization
            // code, skipping the OTP challenge entirely.
            let replay = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .add_header(
                    "Authorization",
                    format!("Bearer {step_token}")
                        .parse::<HeaderValue>()
                        .unwrap(),
                )
                .json(&json!({}))
                .await;

            let replayed_body = replay.text();
            assert_ne!(
                replay.status_code(),
                200,
                "the replayed step token was accepted: {replayed_body}"
            );
            assert!(
                !replayed_body.contains("\"code="),
                "the replay produced an authorization code: {replayed_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test mfa_bypass_test -- --ignored"]
    fn verify_otp_refuses_to_enrol_without_a_pending_server_side_enrollment() {
        rt().block_on(async {
            grant_otp_credential_to_admin().await;
            let before = count_admin_otp_credentials().await;
            assert_eq!(before, 1, "the victim must start with one OTP credential");

            let server = make_server();
            let authorize = start_authorization(&server).await;

            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

            // The step token is exactly what `auth_login_actions` accepts, so this is
            // the enrolment endpoint an attacker holding only the password can reach.
            // No `setup_otp` was called, so there is no enrolment to claim.
            let enrol = server
                .post(&format!("/realms/{}/login-actions/verify-otp", realm()))
                .add_header(
                    "Cookie",
                    HeaderValue::from_str(&format!("FERRISKEY_LOGIN_ACTION={step_token}")).unwrap(),
                )
                .json(&json!({ "code": "000000" }))
                .await;

            assert_ne!(
                enrol.status_code(),
                401,
                "the flow cookie must authenticate the call, or this test proves nothing: {}",
                enrol.text()
            );

            assert_ne!(
                enrol.status_code(),
                200,
                "enrolment succeeded without a server-side candidate: {}",
                enrol.text()
            );

            assert_eq!(
                count_admin_otp_credentials().await,
                before,
                "the victim's existing OTP credential must survive a refused enrolment"
            );
        });
    }
}
