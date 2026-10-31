/// Integration tests for the self-service session API (#1885).
///
/// These tests require a running PostgreSQL instance. They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test own_sessions_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::PasswordHashingCost;
    use std::{env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
    use axum_test::TestServer;
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
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

    const ADMIN_PASSWORD: &str = "admin_pass_1234!";
    const USER_PASSWORD: &str = "Own-Sessions-Pw-1234!";

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
        /// Kept alive for the process lifetime; not read directly by these tests.
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

        let schema = format!("test_own_sessions_{}", Uuid::new_v4().simple());
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
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: false,
            config_cache_ttl_seconds: 0,
            password_hashing: PasswordHashingCost::default(),
            refresh_token_format: Default::default(),
            refresh_token_commit: Default::default(),
            webapp_url: "http://localhost:5555".to_string(),
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema: schema.clone(),
                max_connections: 100,
                min_connections: 5,
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

    fn sid_claim(access_token: &str) -> String {
        let payload = access_token.split('.').nth(1).expect("jwt payload");
        let raw = URL_SAFE_NO_PAD.decode(payload).expect("base64 payload");
        let claims: Value = serde_json::from_slice(&raw).expect("json claims");
        claims["sid"]
            .as_str()
            .expect("the password grant binds its token to a session")
            .to_string()
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

    /// A dedicated user, so tests running in parallel against the one shared
    /// realm never see each other's sessions.
    async fn create_user(srv: &TestServer, realm: &str) -> String {
        let admin = login(srv, realm, "admin", ADMIN_PASSWORD).await;
        let username = format!("own-sessions-{}", Uuid::new_v4().simple());

        let created = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(&admin))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(created.status_code(), 200, "{}", created.text());
        let user_id = created.json::<Value>()["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let password = srv
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(&admin))
            .json(&json!({
                "value": USER_PASSWORD,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(password.status_code(), 200, "{}", password.text());

        username
    }

    async fn listed(srv: &TestServer, realm: &str, token: &str) -> Vec<Value> {
        let resp = srv
            .get(&format!("/realms/{}/users/me/sessions", realm))
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(resp.status_code(), 200, "{}", resp.text());
        resp.json::<Value>()["data"]
            .as_array()
            .expect("sessions array")
            .clone()
    }

    async fn token_still_works(srv: &TestServer, realm: &str, token: &str) -> bool {
        srv.get(&format!(
            "/realms/{}/protocol/openid-connect/userinfo",
            realm
        ))
        .add_header("Authorization", auth_header(token))
        .await
        .status_code()
            == 200
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test own_sessions_test -- --ignored"]
    fn a_user_lists_only_its_own_sessions_and_sees_which_one_is_current() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let alice = create_user(&srv, &realm).await;
            let bob = create_user(&srv, &realm).await;

            let first = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let second = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let bobs = login(&srv, &realm, &bob, USER_PASSWORD).await;

            let sessions = listed(&srv, &realm, &second).await;

            let ids: Vec<&str> = sessions.iter().filter_map(|s| s["id"].as_str()).collect();
            assert_eq!(ids.len(), 2, "{sessions:?}");
            assert!(ids.contains(&sid_claim(&first).as_str()), "{sessions:?}");
            assert!(ids.contains(&sid_claim(&second).as_str()), "{sessions:?}");
            assert!(!ids.contains(&sid_claim(&bobs).as_str()), "{sessions:?}");

            let current: Vec<&str> = sessions
                .iter()
                .filter(|s| s["current"] == true)
                .filter_map(|s| s["id"].as_str())
                .collect();
            assert_eq!(current, [sid_claim(&second).as_str()]);

            for session in &sessions {
                assert!(session.get("user_id").is_none(), "{session}");
                assert!(session.get("sso_token_hash").is_none(), "{session}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test own_sessions_test -- --ignored"]
    fn a_user_revokes_one_of_its_sessions_and_that_session_dies() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let alice = create_user(&srv, &realm).await;
            let keep = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let doomed = login(&srv, &realm, &alice, USER_PASSWORD).await;

            let revoked = srv
                .delete(&format!(
                    "/realms/{}/users/me/sessions/{}",
                    realm,
                    sid_claim(&doomed)
                ))
                .add_header("Authorization", auth_header(&keep))
                .await;
            assert_eq!(revoked.status_code(), 204, "{}", revoked.text());

            assert!(!token_still_works(&srv, &realm, &doomed).await);
            assert!(token_still_works(&srv, &realm, &keep).await);

            let remaining = listed(&srv, &realm, &keep).await;
            assert_eq!(remaining.len(), 1, "{remaining:?}");
            assert_eq!(remaining[0]["id"], sid_claim(&keep));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test own_sessions_test -- --ignored"]
    fn a_user_cannot_revoke_a_session_of_another_user() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let alice = create_user(&srv, &realm).await;
            let bob = create_user(&srv, &realm).await;
            let alices = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let bobs = login(&srv, &realm, &bob, USER_PASSWORD).await;

            let attempt = srv
                .delete(&format!(
                    "/realms/{}/users/me/sessions/{}",
                    realm,
                    sid_claim(&bobs)
                ))
                .add_header("Authorization", auth_header(&alices))
                .await;
            assert_eq!(attempt.status_code(), 404, "{}", attempt.text());

            assert!(token_still_works(&srv, &realm, &bobs).await);
            assert_eq!(listed(&srv, &realm, &bobs).await.len(), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test own_sessions_test -- --ignored"]
    fn revoking_the_others_keeps_the_current_session_and_spares_other_users() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let alice = create_user(&srv, &realm).await;
            let bob = create_user(&srv, &realm).await;
            let current = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let phone = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let laptop = login(&srv, &realm, &alice, USER_PASSWORD).await;
            let bobs = login(&srv, &realm, &bob, USER_PASSWORD).await;

            let resp = srv
                .post(&format!(
                    "/realms/{}/users/me/sessions/revoke-others",
                    realm
                ))
                .add_header("Authorization", auth_header(&current))
                .await;
            assert_eq!(resp.status_code(), 200, "{}", resp.text());
            assert_eq!(resp.json::<Value>()["revoked"], 2);

            assert!(token_still_works(&srv, &realm, &current).await);
            assert!(!token_still_works(&srv, &realm, &phone).await);
            assert!(!token_still_works(&srv, &realm, &laptop).await);
            assert!(token_still_works(&srv, &realm, &bobs).await);

            let remaining = listed(&srv, &realm, &current).await;
            assert_eq!(remaining.len(), 1, "{remaining:?}");
            assert_eq!(remaining[0]["current"], true);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test own_sessions_test -- --ignored"]
    fn the_own_session_endpoints_require_authentication() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let list = srv
                .get(&format!("/realms/{}/users/me/sessions", realm))
                .await;
            let revoke = srv
                .delete(&format!(
                    "/realms/{}/users/me/sessions/{}",
                    realm,
                    Uuid::new_v4()
                ))
                .await;
            let others = srv
                .post(&format!(
                    "/realms/{}/users/me/sessions/revoke-others",
                    realm
                ))
                .await;

            assert_eq!(list.status_code(), 401);
            assert_eq!(revoke.status_code(), 401);
            assert_eq!(others.status_code(), 401);
        });
    }
}
