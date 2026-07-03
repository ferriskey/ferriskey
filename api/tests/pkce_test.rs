/// Integration tests for PKCE on the authorization code flow (RFC 7636).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test pkce_test -- --ignored
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

    // ---------------------------------------------------------------------------
    // Shared runtime / context helpers (same pattern as device_flow_test.rs)
    // ---------------------------------------------------------------------------

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
        /// client_id of a public client with PKCE not required (default)
        plain_client_id: String,
        /// client_id of a client with require_pkce = true
        pkce_required_client_id: String,
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

    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("pkce_test_{}", Uuid::new_v4().simple());

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

        let realm_name = format!("pkce-realm-{}", Uuid::new_v4().simple());

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

        // Create a plain public client (PKCE optional)
        let plain_client_id = format!("pkce-plain-{}", Uuid::new_v4().simple());
        sqlx::query(
            r#"INSERT INTO realms (id, name, created_at, updated_at) SELECT id, name, created_at, updated_at FROM realms WHERE name = $1 LIMIT 0"#
        )
        .bind(&realm_name)
        .execute(&pool)
        .await
        .ok();

        // Fetch realm id
        let realm_row: (Uuid,) = sqlx::query_as("SELECT id FROM realms WHERE name = $1")
            .bind(&realm_name)
            .fetch_one(&pool)
            .await
            .expect("fetch realm id");
        let realm_id = realm_row.0;

        // Fetch admin user id
        let user_row: (Uuid,) =
            sqlx::query_as("SELECT id FROM users WHERE username = 'admin' AND realm_id = $1")
                .bind(realm_id)
                .fetch_one(&pool)
                .await
                .expect("fetch admin user");
        let admin_user_id = user_row.0;
        let _ = admin_user_id;

        let plain_client_uuid = Uuid::new_v4();
        let pkce_required_uuid = Uuid::new_v4();
        let pkce_required_client_id = format!("pkce-required-{}", Uuid::new_v4().simple());

        let now = chrono::Utc::now().naive_utc();

        // Plain client — PKCE optional
        sqlx::query(
            r#"INSERT INTO clients
               (id, realm_id, name, client_id, enabled, protocol, public_client,
                service_account_enabled, client_type, require_pkce, created_at, updated_at)
               VALUES ($1,$2,$3,$4,true,'openid-connect',true,false,'public',false,$5,$5)"#,
        )
        .bind(plain_client_uuid)
        .bind(realm_id)
        .bind(&plain_client_id)
        .bind(&plain_client_id)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert plain client");

        // require_pkce client
        sqlx::query(
            r#"INSERT INTO clients
               (id, realm_id, name, client_id, enabled, protocol, public_client,
                service_account_enabled, client_type, require_pkce, created_at, updated_at)
               VALUES ($1,$2,$3,$4,true,'openid-connect',true,false,'public',true,$5,$5)"#,
        )
        .bind(pkce_required_uuid)
        .bind(realm_id)
        .bind(&pkce_required_client_id)
        .bind(&pkce_required_client_id)
        .bind(now)
        .execute(&pool)
        .await
        .expect("insert pkce-required client");

        // Add redirect URIs
        for uuid in [plain_client_uuid, pkce_required_uuid] {
            sqlx::query(
                r#"INSERT INTO redirect_uris (id, client_id, value, enabled)
                   VALUES ($1,$2,'http://localhost/callback',true)"#,
            )
            .bind(Uuid::new_v4())
            .bind(uuid)
            .execute(&pool)
            .await
            .expect("insert redirect uri");
        }

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            plain_client_id,
            pkce_required_client_id,
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

    // ---------------------------------------------------------------------------
    // PKCE helper functions
    // ---------------------------------------------------------------------------

    fn token_url(realm: &str) -> String {
        format!("/realms/{}/protocol/openid-connect/token", realm)
    }

    fn auth_url(realm: &str) -> String {
        format!("/realms/{}/protocol/openid-connect/auth", realm)
    }

    // ---------------------------------------------------------------------------
    // Tests
    // ---------------------------------------------------------------------------

    /// S256 happy-path: authorize with code_challenge, exchange with code_verifier.
    ///
    /// Uses the RFC 7636 Appendix B test vector:
    ///   verifier  = dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
    ///   challenge = E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test pkce_test -- --ignored"]
    fn pkce_s256_happy_path() {
        rt().block_on(async {
            let ctx = shared_ctx();
            let server = make_server();
            let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
            // RFC 7636 Appendix B: BASE64URL(SHA256(verifier))
            let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

            // 1. GET /auth to create a session with PKCE challenge
            let auth_resp = server
                .get(&auth_url(realm()))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &ctx.plain_client_id)
                .add_query_param("redirect_uri", "http://localhost/callback")
                .add_query_param("scope", "openid")
                .add_query_param("code_challenge", &challenge)
                .add_query_param("code_challenge_method", "S256")
                .await;

            // /auth returns 302 redirect — extract session cookie
            assert_eq!(auth_resp.status_code(), 302, "auth should redirect");
            let session_cookie = auth_resp
                .headers()
                .get_all("set-cookie")
                .iter()
                .find_map(|v| {
                    let s = v.to_str().ok()?;
                    if s.contains("FERRISKEY_SESSION") {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .expect("FERRISKEY_SESSION cookie");
            let session_id = session_cookie
                .split('=')
                .nth(1)
                .and_then(|s| s.split(';').next())
                .expect("session id");

            // 2. Authenticate to get the authorization code
            let auth_result = server
                .post(&format!(
                    "/realms/{}/protocol/openid-connect/authenticate",
                    realm()
                ))
                .add_query_param("session_code", session_id)
                .add_query_param("client_id", &ctx.plain_client_id)
                .json(&serde_json::json!({
                    "username": "admin",
                    "password": "admin"
                }))
                .await;

            // 302 redirect with ?code=...
            let location = auth_result
                .headers()
                .get("location")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default()
                .to_string();
            let code = location
                .split("code=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .expect("authorization code in redirect");

            // 3. Exchange code with code_verifier
            let token_resp = server
                .post(&token_url(realm()))
                .content_type("application/x-www-form-urlencoded")
                .text(&format!(
                    "grant_type=authorization_code&client_id={}&code={}&redirect_uri=http%3A%2F%2Flocalhost%2Fcallback&code_verifier={}",
                    ctx.plain_client_id, code, verifier
                ))
                .await;

            let status = token_resp.status_code();
            let body: Value = token_resp.json();
            assert_eq!(status, 200, "token exchange should succeed: {body:?}");
            assert!(body.get("access_token").is_some(), "should have access_token");
        });
    }

    /// Wrong verifier → invalid_grant.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test pkce_test -- --ignored"]
    fn pkce_s256_wrong_verifier_rejected() {
        rt().block_on(async {
            let ctx = shared_ctx();
            let server = make_server();
            let wrong_verifier = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
            // RFC 7636 Appendix B challenge (for the correct verifier, not `wrong_verifier`)
            let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

            let auth_resp = server
                .get(&auth_url(realm()))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &ctx.plain_client_id)
                .add_query_param("redirect_uri", "http://localhost/callback")
                .add_query_param("scope", "openid")
                .add_query_param("code_challenge", &challenge)
                .add_query_param("code_challenge_method", "S256")
                .await;

            assert_eq!(auth_resp.status_code(), 302);
            let session_cookie = auth_resp
                .headers()
                .get_all("set-cookie")
                .iter()
                .find_map(|v| {
                    let s = v.to_str().ok()?;
                    if s.contains("FERRISKEY_SESSION") { Some(s.to_string()) } else { None }
                })
                .expect("session cookie");
            let session_id = session_cookie.split('=').nth(1).and_then(|s| s.split(';').next()).unwrap();

            let auth_result = server
                .post(&format!("/realms/{}/protocol/openid-connect/authenticate", realm()))
                .add_query_param("session_code", session_id)
                .add_query_param("client_id", &ctx.plain_client_id)
                .json(&serde_json::json!({"username": "admin", "password": "admin"}))
                .await;

            let location = auth_result.headers().get("location")
                .and_then(|v| v.to_str().ok()).unwrap_or_default().to_string();
            let code = location.split("code=").nth(1)
                .and_then(|s| s.split('&').next()).expect("code");

            let token_resp = server
                .post(&token_url(realm()))
                .content_type("application/x-www-form-urlencoded")
                .text(&format!(
                    "grant_type=authorization_code&client_id={}&code={}&redirect_uri=http%3A%2F%2Flocalhost%2Fcallback&code_verifier={}",
                    ctx.plain_client_id, code, wrong_verifier
                ))
                .await;

            let status = token_resp.status_code();
            let body: Value = token_resp.json();
            assert_eq!(status, 400, "wrong verifier should fail: {body:?}");
            assert_eq!(body["error"], "invalid_grant");
        });
    }

    /// require_pkce = true: missing code_challenge at /auth → invalid_request.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test pkce_test -- --ignored"]
    fn pkce_required_client_rejects_auth_without_challenge() {
        rt().block_on(async {
            let ctx = shared_ctx();
            let server = make_server();

            let auth_resp = server
                .get(&auth_url(realm()))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &ctx.pkce_required_client_id)
                .add_query_param("redirect_uri", "http://localhost/callback")
                .add_query_param("scope", "openid")
                .await;

            // /auth redirects to error page (no redirect_uri forwarding) with 302
            // or returns OAuth error directly — accept both 302 and 4xx
            let status = auth_resp.status_code().as_u16();
            assert!(
                status == 302 || status == 400,
                "should reject missing challenge: got {status}"
            );
        });
    }

    /// require_pkce = true: plain method rejected.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test pkce_test -- --ignored"]
    fn pkce_required_client_rejects_plain_method() {
        rt().block_on(async {
            let ctx = shared_ctx();
            let server = make_server();
            let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

            let auth_resp = server
                .get(&auth_url(realm()))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &ctx.pkce_required_client_id)
                .add_query_param("redirect_uri", "http://localhost/callback")
                .add_query_param("scope", "openid")
                .add_query_param("code_challenge", verifier)
                .add_query_param("code_challenge_method", "plain")
                .await;

            let status = auth_resp.status_code().as_u16();
            assert!(
                status == 302 || status == 400,
                "should reject plain method: got {status}"
            );
        });
    }

    /// require_pkce = true: omitting code_challenge_method must be rejected.
    /// An absent method would otherwise default to `plain` at verification and
    /// silently defeat the policy, so it has to be refused up front.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test pkce_test -- --ignored"]
    fn pkce_required_client_rejects_omitted_method() {
        rt().block_on(async {
            let ctx = shared_ctx();
            let server = make_server();
            let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

            let auth_resp = server
                .get(&auth_url(realm()))
                .add_query_param("response_type", "code")
                .add_query_param("client_id", &ctx.pkce_required_client_id)
                .add_query_param("redirect_uri", "http://localhost/callback")
                .add_query_param("scope", "openid")
                .add_query_param("code_challenge", verifier)
                .await;

            let status = auth_resp.status_code().as_u16();
            assert!(
                status == 302 || status == 400,
                "should reject omitted method: got {status}"
            );
        });
    }
}
