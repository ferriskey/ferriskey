/// Integration tests for the SeaWatch audit log (#1270).
///
/// These pin two things that were previously broken:
/// - every stored event is actually linked into the tamper-evident hash chain
///   (`event_hash`/`prev_hash` are populated, not left NULL)
/// - `GET .../seawatch/v1/security-events` actually applies the query filter
///   it's given instead of always returning the unfiltered first page
///
/// Require a running PostgreSQL instance. Marked `#[ignore]` so they don't
/// block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test seawatch_test -- --ignored
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

        let schema = format!("test_seawatch_{}", Uuid::new_v4().simple());
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
            admin_password: "admin_pass_1234!".to_string(),
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

    async fn login(server: &TestServer, realm_name: &str) -> String {
        let token_resp = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin_pass_1234!"),
                ("scope", "openid profile"),
            ])
            .await;

        assert_eq!(
            token_resp.status_code(),
            200,
            "password grant failed: {}",
            token_resp.text()
        );

        let body: Value = token_resp.json();
        body["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    async fn create_user(srv: &TestServer, realm: &str, token: &str, username: &str) {
        let resp = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&serde_json::json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "user creation failed: {}",
            resp.text()
        );
    }

    async fn security_events(srv: &TestServer, realm: &str, token: &str, query: &str) -> Value {
        let path = if query.is_empty() {
            format!("/realms/{}/seawatch/v1/security-events", realm)
        } else {
            format!("/realms/{}/seawatch/v1/security-events?{}", realm, query)
        };
        let resp = srv
            .get(&path)
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "fetching security events failed: {}",
            resp.text()
        );
        resp.json()
    }

    async fn create_user_with_password(
        srv: &TestServer,
        realm: &str,
        token: &str,
        username: &str,
        password: &str,
    ) -> String {
        let resp = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&serde_json::json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "user creation failed: {}",
            resp.text()
        );

        let created: Value = resp.json();
        let user_id = created["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let pw_resp = srv
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&serde_json::json!({
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

        user_id
    }

    async fn disable_user(srv: &TestServer, realm: &str, token: &str, user_id: &str) {
        let resp = srv
            .put(&format!("/realms/{}/users/{}", realm, user_id))
            .add_header("Authorization", auth_header(token))
            .json(&serde_json::json!({ "enabled": false }))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "disabling the user failed: {}",
            resp.text()
        );
    }

    async fn password_grant(srv: &TestServer, realm: &str, username: &str, password: &str) -> u16 {
        srv.post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await
            .status_code()
            .as_u16()
    }

    async fn login_failures(srv: &TestServer, realm: &str, token: &str) -> Vec<Value> {
        let body = security_events(srv, realm, token, "event_types=login_failure&limit=1000").await;
        body["data"].as_array().expect("events array").clone()
    }

    fn failure_for(events: &[Value], user_id: &str, reason: &str) -> bool {
        events
            .iter()
            .any(|e| e["actor_id"] == user_id && e["details"]["reason"] == reason)
    }

    async fn interactive_login_attempt(
        srv: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) {
        let authorize = srv
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", "ferriskey-admin")
            .add_query_param(
                "redirect_uri",
                format!(
                    "http://localhost:5555/realms/{}/authentication/callback",
                    realm
                )
                .as_str(),
            )
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param(
                "code_challenge",
                "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
            )
            .add_query_param("code_challenge_method", "S256")
            .await;

        let status = authorize.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth, got {status}: {}",
            authorize.text()
        );

        let login = srv
            .post(&format!("/realms/{}/login-actions/authenticate", realm))
            .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", "ferriskey-admin")
            .json(&serde_json::json!({ "username": username, "password": password }))
            .await;

        assert_ne!(
            login.status_code(),
            200,
            "the login was supposed to fail: {}",
            login.text()
        );
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn every_stored_event_is_linked_into_the_hash_chain() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;

            let body = security_events(&srv, &realm, &token, "").await;
            let events = body["data"].as_array().expect("events array");

            let session_created = events
                .iter()
                .find(|e| e["event_type"] == "session_created")
                .expect("a session_created event was recorded for the login above");

            assert!(
                !session_created["event_hash"].is_null(),
                "session_created event has no event_hash — the chain was not built: {session_created}"
            );
            assert!(
                !session_created["prev_hash"].is_null(),
                "session_created event has no prev_hash — the chain was not built: {session_created}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn event_types_filter_is_actually_applied() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            create_user(&srv, &realm, &token, &format!("filtertest-{}", Uuid::new_v4().simple())).await;

            // Unfiltered: both event types the two actions above produced should be present.
            let all = security_events(&srv, &realm, &token, "").await;
            let all_events = all["data"].as_array().expect("events array");
            assert!(
                all_events.iter().any(|e| e["event_type"] == "session_created"),
                "expected a session_created event in the unfiltered page: {all_events:?}"
            );
            assert!(
                all_events.iter().any(|e| e["event_type"] == "user_created"),
                "expected a user_created event in the unfiltered page: {all_events:?}"
            );

            // Filtered to user_created only: session_created must not leak through.
            let filtered = security_events(&srv, &realm, &token, "event_types=user_created").await;
            let filtered_events = filtered["data"].as_array().expect("events array");
            assert!(
                !filtered_events.is_empty(),
                "expected at least one user_created event: {filtered_events:?}"
            );
            assert!(
                filtered_events.iter().all(|e| e["event_type"] == "user_created"),
                "the event_types filter was not applied — a non-user_created event leaked through: {filtered_events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn limit_is_actually_applied() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            create_user(
                &srv,
                &realm,
                &token,
                &format!("limittest-{}", Uuid::new_v4().simple()),
            )
            .await;

            let limited = security_events(&srv, &realm, &token, "limit=1").await;
            let events = limited["data"].as_array().expect("events array");
            assert_eq!(
                events.len(),
                1,
                "limit=1 should return exactly one event: {events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn a_successful_login_records_login_success_alongside_session_created() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("loginok-{}", Uuid::new_v4().simple());
            let user_id =
                create_user_with_password(&srv, &realm, &token, &username, "S3cret-Password!")
                    .await;

            assert_eq!(
                password_grant(&srv, &realm, &username, "S3cret-Password!").await,
                200
            );

            let body =
                security_events(&srv, &realm, &token, "event_types=login_success&limit=1000").await;
            let events = body["data"].as_array().expect("events array");
            assert!(
                events.iter().any(|e| e["actor_id"] == user_id),
                "a successful login produced no login_success event for {user_id}: {events:?}"
            );

            let sessions = security_events(
                &srv,
                &realm,
                &token,
                "event_types=session_created&limit=1000",
            )
            .await;
            assert!(
                sessions["data"]
                    .as_array()
                    .expect("events array")
                    .iter()
                    .any(|e| e["actor_id"] == user_id),
                "login_success replaced session_created instead of joining it"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn a_wrong_password_is_recorded_as_a_login_failure() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("wrongpw-{}", Uuid::new_v4().simple());
            let user_id =
                create_user_with_password(&srv, &realm, &token, &username, "S3cret-Password!")
                    .await;

            assert_ne!(
                password_grant(&srv, &realm, &username, "not-the-password").await,
                200
            );

            let events = login_failures(&srv, &realm, &token).await;
            assert!(
                failure_for(&events, &user_id, "invalid_credentials"),
                "a wrong password left no login_failure for {user_id}: {events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn an_unknown_username_is_recorded_without_an_actor_and_without_the_identifier() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("ghost-{}", Uuid::new_v4().simple());

            assert_ne!(
                password_grant(&srv, &realm, &username, "whatever").await,
                200
            );

            let events = login_failures(&srv, &realm, &token).await;
            let anonymous: Vec<&Value> = events
                .iter()
                .filter(|e| e["details"]["reason"] == "user_not_found")
                .collect();

            assert!(
                !anonymous.is_empty(),
                "a login for an unknown username left no trace: {events:?}"
            );
            assert!(
                anonymous.iter().all(|e| e["actor_id"].is_null()),
                "an unknown username was attributed to an actor: {anonymous:?}"
            );
            assert!(
                !serde_json::to_string(&anonymous)
                    .expect("serialise events")
                    .contains(&username),
                "the attempted identifier leaked into the audit log: {anonymous:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn a_disabled_account_is_recorded_as_a_login_failure() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("disabled-{}", Uuid::new_v4().simple());
            let user_id =
                create_user_with_password(&srv, &realm, &token, &username, "S3cret-Password!")
                    .await;
            disable_user(&srv, &realm, &token, &user_id).await;

            assert_ne!(
                password_grant(&srv, &realm, &username, "S3cret-Password!").await,
                200
            );

            let events = login_failures(&srv, &realm, &token).await;
            assert!(
                failure_for(&events, &user_id, "user_disabled"),
                "a disabled account left no login_failure for {user_id}: {events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn a_locked_account_is_recorded_as_a_login_failure() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("locked-{}", Uuid::new_v4().simple());
            let user_id =
                create_user_with_password(&srv, &realm, &token, &username, "S3cret-Password!")
                    .await;

            for _ in 0..11 {
                password_grant(&srv, &realm, &username, "not-the-password").await;
            }

            assert_ne!(
                password_grant(&srv, &realm, &username, "S3cret-Password!").await,
                200
            );

            let events = login_failures(&srv, &realm, &token).await;
            assert!(
                failure_for(&events, &user_id, "account_locked"),
                "a locked account left no account_locked login_failure for {user_id}: {events:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test seawatch_test -- --ignored"]
    fn a_failed_interactive_login_is_recorded_as_a_login_failure() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let username = format!("interactive-{}", Uuid::new_v4().simple());
            let user_id =
                create_user_with_password(&srv, &realm, &token, &username, "S3cret-Password!")
                    .await;

            interactive_login_attempt(&srv, &realm, &username, "not-the-password").await;

            let events = login_failures(&srv, &realm, &token).await;
            assert!(
                failure_for(&events, &user_id, "invalid_credentials"),
                "a failed interactive login left no login_failure for {user_id}: {events:?}"
            );
        });
    }
}
