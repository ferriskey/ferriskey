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
    use serde_json::json;
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const SURVEY_CLIENT_ID: &str = "survey-app";
    const SURVEY_REDIRECT_URI: &str = "https://survey.example.com/oidc/callback";
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

        let schema = format!("sso_session_test_{}", Uuid::new_v4().simple());

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
            webhook_allow_private_endpoints: false,
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

        seed_survey_client(&pool, &realm_name).await;

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            pool,
        }
    }

    async fn seed_survey_client(pool: &PgPool, realm_name: &str) {
        let client_uuid = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, secret, enabled, protocol, \
             public_client, service_account_enabled, client_type, require_pkce) \
             SELECT $1, r.id, 'Survey', $2, 'survey-secret', TRUE, 'openid-connect', \
             FALSE, FALSE, 'confidential', FALSE \
             FROM realms r WHERE r.name = $3",
        )
        .bind(client_uuid)
        .bind(SURVEY_CLIENT_ID)
        .bind(realm_name)
        .execute(pool)
        .await
        .expect("insert survey client");

        sqlx::query(
            "INSERT INTO redirect_uris (id, client_id, value, enabled) VALUES ($1, $2, $3, TRUE)",
        )
        .bind(Uuid::new_v4())
        .bind(client_uuid)
        .bind(SURVEY_REDIRECT_URI)
        .execute(pool)
        .await
        .expect("insert survey redirect uri");
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

    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn serial() -> std::sync::MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    async fn set_admin_enabled(enabled: bool) {
        sqlx::query("UPDATE users SET enabled = $1 WHERE username = 'admin'")
            .bind(enabled)
            .execute(&shared_ctx().pool)
            .await
            .expect("update admin enabled");
    }

    async fn count_sso_login_events() -> i64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM security_events \
             WHERE event_type = 'login_success' AND details->>'method' = 'sso_session'",
        )
        .fetch_one(&shared_ctx().pool)
        .await
        .expect("count sso login events")
    }

    async fn count_admin_sessions() -> i64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_sessions s \
             JOIN users u ON u.id = s.user_id WHERE u.username = 'admin'",
        )
        .fetch_one(&shared_ctx().pool)
        .await
        .expect("count admin sessions")
    }

    async fn start_console_authorization(server: &TestServer) -> axum_test::TestResponse {
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

    async fn start_survey_authorization(
        server: &TestServer,
        sso_cookie: Option<&str>,
    ) -> axum_test::TestResponse {
        let mut request = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm()))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", SURVEY_CLIENT_ID)
            .add_query_param("redirect_uri", SURVEY_REDIRECT_URI)
            .add_query_param("scope", "openid")
            .add_query_param("state", "survey-state");

        if let Some(value) = sso_cookie {
            request = request.add_header(
                "Cookie",
                axum::http::HeaderValue::from_str(&format!("FERRISKEY_SSO={value}"))
                    .expect("cookie header"),
            );
        }

        request.await
    }

    fn location_of(response: &axum_test::TestResponse) -> String {
        response
            .headers()
            .get("location")
            .expect("a redirect must carry a Location header")
            .to_str()
            .expect("Location must be readable")
            .to_string()
    }

    async fn sign_in(server: &TestServer) -> String {
        let authorize = start_console_authorization(server).await;

        let login = server
            .post(&format!("/realms/{}/login-actions/authenticate", realm()))
            .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .json(&json!({ "username": "admin", "password": "admin" }))
            .await;

        assert_eq!(
            login.status_code(),
            200,
            "the login must succeed: {}",
            login.text()
        );

        login.cookie("FERRISKEY_SSO").value().to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn a_second_application_signs_in_from_the_session_cookie() {
        let _serial = serial();
        rt().block_on(async {
            set_admin_enabled(true).await;
            let server = make_server();
            let sso = sign_in(&server).await;

            let survey = start_survey_authorization(&server, Some(&sso)).await;
            let location = location_of(&survey);

            assert!(
                location.starts_with(SURVEY_REDIRECT_URI),
                "SSO must send the browser back to the application, not to the login page: {location}"
            );
            assert!(
                location.contains("code="),
                "the redirect must carry an authorization code: {location}"
            );
            assert!(
                location.contains("state=survey-state"),
                "the state must survive the round trip: {location}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn the_login_hands_the_browser_an_opaque_session_cookie() {
        let _serial = serial();
        rt().block_on(async {
            set_admin_enabled(true).await;
            let server = make_server();
            let authorize = start_console_authorization(&server).await;

            let login = server
                .post(&format!("/realms/{}/login-actions/authenticate", realm()))
                .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
                .add_query_param("client_id", SEEDED_CLIENT_ID)
                .json(&json!({ "username": "admin", "password": "admin" }))
                .await;

            let raw = login
                .headers()
                .get_all("set-cookie")
                .iter()
                .filter_map(|v| v.to_str().ok())
                .find(|v| v.starts_with("FERRISKEY_SSO="))
                .expect("the login must hand back an SSO cookie")
                .to_string();

            assert!(
                raw.contains("HttpOnly"),
                "the cookie must be HttpOnly: {raw}"
            );
            assert!(
                raw.contains("Max-Age"),
                "the cookie must outlive the browser window: {raw}"
            );

            let value = login.cookie("FERRISKEY_SSO").value().to_string();
            assert_eq!(
                value.split('.').count(),
                1,
                "the cookie must not be a JWT: {value}"
            );
            assert!(
                value.len() >= 32,
                "the cookie must be hard to guess: {value}"
            );

            let session_ids = sqlx::query_scalar::<_, Uuid>(
                "SELECT s.id FROM user_sessions s JOIN users u ON u.id = s.user_id \
                 WHERE u.username = 'admin'",
            )
            .fetch_all(&shared_ctx().pool)
            .await
            .expect("read admin sessions");

            assert!(
                !session_ids.iter().any(|id| id.to_string() == value),
                "the cookie must not be the session id: {value}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn the_second_application_joins_the_session_instead_of_opening_another() {
        let _serial = serial();
        rt().block_on(async {
            set_admin_enabled(true).await;
            let server = make_server();
            let sso = sign_in(&server).await;

            let before = count_admin_sessions().await;

            let survey = start_survey_authorization(&server, Some(&sso)).await;
            assert!(
                location_of(&survey).starts_with(SURVEY_REDIRECT_URI),
                "pre-condition: SSO must succeed"
            );

            assert_eq!(
                count_admin_sessions().await,
                before,
                "resuming a session must not open a second one for the same sign-in"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn resuming_a_session_is_recorded_as_a_login() {
        let _serial = serial();
        rt().block_on(async {
            set_admin_enabled(true).await;
            let server = make_server();
            let sso = sign_in(&server).await;

            let before = count_sso_login_events().await;

            let survey = start_survey_authorization(&server, Some(&sso)).await;
            assert!(
                location_of(&survey).starts_with(SURVEY_REDIRECT_URI),
                "pre-condition: SSO must succeed"
            );

            assert_eq!(
                count_sso_login_events().await,
                before + 1,
                "an SSO login must leave a trace in the audit log"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn a_disabled_user_cannot_resume_a_session() {
        let _serial = serial();
        rt().block_on(async {
            set_admin_enabled(true).await;
            let server = make_server();
            let sso = sign_in(&server).await;

            set_admin_enabled(false).await;

            let survey = start_survey_authorization(&server, Some(&sso)).await;
            set_admin_enabled(true).await;

            let location = location_of(&survey);

            assert!(
                !location.starts_with(SURVEY_REDIRECT_URI),
                "a disabled account must not walk away with an authorization code: {location}"
            );
            assert!(
                location.starts_with(WEBAPP_URL),
                "the browser must land on the login page instead: {location}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn an_unknown_session_falls_back_to_the_login_page() {
        let _serial = serial();
        rt().block_on(async {
            let server = make_server();

            let survey =
                start_survey_authorization(&server, Some(&Uuid::new_v4().to_string())).await;
            let location = location_of(&survey);

            assert!(
                location.starts_with(WEBAPP_URL),
                "an unknown session must land on the login page: {location}"
            );
            assert!(
                location.contains("session_expired=1"),
                "the login page should say why the user is seeing it again: {location}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test sso_session_test -- --ignored"]
    fn no_cookie_means_the_ordinary_login_page() {
        let _serial = serial();
        rt().block_on(async {
            let server = make_server();

            let survey = start_survey_authorization(&server, None).await;
            let location = location_of(&survey);

            assert!(
                location.starts_with(WEBAPP_URL),
                "the browser must land on the login page: {location}"
            );
            assert!(
                !location.contains("session_expired=1"),
                "a first visit is not an expired session: {location}"
            );
        });
    }
}
