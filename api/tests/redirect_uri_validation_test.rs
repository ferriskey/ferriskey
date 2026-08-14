/// Integration test: `redirect_uri` validation on the authorization endpoint (FK-002).
///
/// Registered redirect URIs used to be compiled as regexes and matched with
/// `Regex::is_match` — a substring search — and every realm's admin console was
/// seeded with the pattern `^/*`, which matches every string. Any host an attacker
/// named was therefore an accepted redirect target on a default installation, and
/// the authorization code was delivered straight to it.
///
/// These tests drive the real `/protocol/openid-connect/auth` endpoint against a
/// normally-seeded realm, so they cover the seeding and the matching together.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]` so it does not run in
/// regular `cargo test`. Run explicitly with:
///
///   cargo test -p ferriskey-api --test redirect_uri_validation_test -- --ignored
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
    use sqlx::Executor;
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";

    /// The client `initialize_application` seeds. Named to avoid the `admin-cli`
    /// short-circuit described in #1086, exactly as the other integration tests do.
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    /// RFC 7636 Appendix B test vector, so the test needs no SHA-256 dependency.
    /// These tests never exchange the code, so the matching verifier is irrelevant —
    /// the challenge only has to satisfy `require_pkce`.
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
    }

    // `router()` installs a *global* Prometheus recorder, so it can only be built
    // once per test binary (#1086). Every test shares this one router and runtime.
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

        let schema = format!("redirect_uri_validation_test_{}", Uuid::new_v4().simple());

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

        assert_eq!(response.status_code(), 200, "admin token request failed");
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    /// Drive `/protocol/openid-connect/auth` and return the `Location` header.
    async fn authorize(server: &TestServer, client_id: &str, redirect_uri: &str) -> String {
        let response = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm()))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", client_id)
            .add_query_param("redirect_uri", redirect_uri)
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;

        assert_eq!(
            response.status_code(),
            302,
            "expected a redirect from /auth, body: {}",
            response.text()
        );

        response
            .headers()
            .get("location")
            .expect("Location header")
            .to_str()
            .expect("Location is valid UTF-8")
            .to_string()
    }

    /// The rejection path redirects to the login page carrying `login_error`; the
    /// accepted path redirects to the login page without it.
    fn assert_rejected(location: &str, redirect_uri: &str) {
        assert!(
            location.contains("login_error"),
            "redirect_uri {redirect_uri} was accepted; /auth redirected to {location}"
        );
        assert!(
            !location.starts_with(redirect_uri),
            "authorization endpoint redirected to the supplied redirect_uri: {location}"
        );
    }

    fn assert_accepted(location: &str, redirect_uri: &str) {
        assert!(
            !location.contains("login_error"),
            "redirect_uri {redirect_uri} was rejected; /auth redirected to {location}"
        );
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test redirect_uri_validation_test -- --ignored"]
    fn seeded_console_rejects_an_attacker_supplied_redirect_uri() {
        // FK-002: this is the whole exploit. A default installation seeded `^/*` on
        // the console client, so this request used to be accepted and the resulting
        // authorization code delivered to attacker.example.
        rt().block_on(async {
            let server = make_server();
            let hostile = "https://attacker.example/steal";

            let location = authorize(&server, SEEDED_CLIENT_ID, hostile).await;

            assert_rejected(&location, hostile);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test redirect_uri_validation_test -- --ignored"]
    fn seeded_console_accepts_its_own_callback() {
        // The other half of the fix: removing the catch-all must not lock the
        // administrator out of the console.
        rt().block_on(async {
            let server = make_server();
            let callback = format!("{WEBAPP_URL}/realms/{}/authentication/callback", realm());

            let location = authorize(&server, SEEDED_CLIENT_ID, &callback).await;

            assert_accepted(&location, &callback);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test redirect_uri_validation_test -- --ignored"]
    fn registered_uri_does_not_accept_a_uri_that_merely_contains_it() {
        // The substring defect applied to every client, independently of the seed:
        // `is_match` is a search, not a full match.
        rt().block_on(async {
            let server = make_server();
            let admin_token = get_admin_token(&server).await;

            let client_id = format!("redirect-test-client-{}", Uuid::new_v4().simple());

            let create_resp = server
                .post(&format!("/realms/{}/clients", realm()))
                .add_header("Authorization", auth_header(&admin_token))
                .json(&json!({
                    "client_id": client_id,
                    "name": "Redirect URI Test Client",
                    "client_type": "public",
                    "protocol": "openid-connect",
                    "public_client": true,
                    "service_account_enabled": false,
                    "direct_access_grants_enabled": false,
                    "enabled": true,
                    "oauth_device_code_grant_enabled": false
                }))
                .await;

            assert_eq!(
                create_resp.status_code(),
                201,
                "client creation failed: {}",
                create_resp.text()
            );
            let client_body: Value = create_resp.json();
            let client_uuid = client_body["id"].as_str().expect("client id in response");

            let registered = "https://app.example.com/callback";

            let redirect_resp = server
                .post(&format!(
                    "/realms/{}/clients/{}/redirects",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&admin_token))
                .json(&json!({ "value": registered, "enabled": true }))
                .await;

            assert_eq!(
                redirect_resp.status_code(),
                201,
                "redirect URI registration failed: {}",
                redirect_resp.text()
            );

            let hostile = format!("https://attacker.example/?next={registered}");
            let location = authorize(&server, &client_id, &hostile).await;
            assert_rejected(&location, &hostile);

            // …and the registered URI itself still works.
            let location = authorize(&server, &client_id, registered).await;
            assert_accepted(&location, registered);
        });
    }
}
