#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
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
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const CLI_CLIENT_ID: &str = "admin-cli";
    const VICTIM_PASSWORD: &str = "Xq7#vLm2$pRt9Wz!";
    const ROTATED_PASSWORD: &str = "Kd4%bNq8&sTv3Yh!";

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

        let schema = format!("token_revocation_test_{}", Uuid::new_v4().simple());

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

    fn bearer(token: &str) -> HeaderValue {
        format!("Bearer {token}")
            .parse()
            .expect("valid Authorization header")
    }

    fn claim_str(token: &str, name: &str) -> Option<String> {
        let payload = token.split('.').nth(1)?;
        let raw = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let claims: Value = serde_json::from_slice(&raw).ok()?;
        claims.get(name)?.as_str().map(str::to_string)
    }

    fn sid_claim(access_token: &str) -> Option<String> {
        claim_str(access_token, "sid")
    }

    async fn password_grant(server: &TestServer, username: &str, password: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", CLI_CLIENT_ID),
                ("username", username),
                ("password", password),
                ("scope", "openid profile email"),
            ])
            .await
    }

    async fn login(server: &TestServer, username: &str, password: &str) -> Value {
        let response = password_grant(server, username, password).await;
        assert_eq!(
            response.status_code(),
            200,
            "password grant for {username} failed: {}",
            response.text()
        );
        response.json()
    }

    async fn admin_token(server: &TestServer) -> String {
        login(server, "admin", "admin").await["access_token"]
            .as_str()
            .expect("admin access_token")
            .to_string()
    }

    async fn refresh(server: &TestServer, refresh_token: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", CLI_CLIENT_ID),
                ("refresh_token", refresh_token),
            ])
            .await
    }

    async fn userinfo(server: &TestServer, access_token: &str) -> TestResponse {
        server
            .get(&format!(
                "/realms/{}/protocol/openid-connect/userinfo",
                realm()
            ))
            .add_header("Authorization", bearer(access_token))
            .await
    }

    async fn list_sessions(server: &TestServer, token: &str, user_id: &str) -> Vec<Value> {
        let response = server
            .get(&format!("/realms/{}/users/{}/sessions", realm(), user_id))
            .add_header("Authorization", bearer(token))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "listing sessions of {user_id} failed: {}",
            response.text()
        );

        response.json::<Value>()["data"]
            .as_array()
            .expect("sessions array")
            .clone()
    }

    async fn revoke_session(
        server: &TestServer,
        token: &str,
        user_id: &str,
        session_id: &str,
    ) -> TestResponse {
        server
            .delete(&format!(
                "/realms/{}/users/{}/sessions/{}",
                realm(),
                user_id,
                session_id
            ))
            .add_header("Authorization", bearer(token))
            .await
    }

    async fn logout(server: &TestServer, id_token: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/logout",
                realm()
            ))
            .form(&[("id_token_hint", id_token)])
            .await
    }

    async fn create_service_client(server: &TestServer) -> (String, String) {
        let admin = admin_token(server).await;
        let client_id = format!("svc-{}", Uuid::new_v4().simple());

        let created = server
            .post(&format!("/realms/{}/clients", realm()))
            .add_header("Authorization", bearer(&admin))
            .json(&json!({
                "name": client_id,
                "client_id": client_id,
                "client_type": "confidential",
                "service_account_enabled": true,
                "public_client": false,
                "protocol": "openid-connect",
                "enabled": true,
                "direct_access_grants_enabled": false,
                "oauth_device_code_grant_enabled": false,
            }))
            .await;

        assert_eq!(
            created.status_code(),
            201,
            "creating the service client failed: {}",
            created.text()
        );

        let secret = created.json::<Value>()["client_secret"]
            .as_str()
            .expect("a confidential client must be given a secret")
            .to_string();

        (client_id, secret)
    }

    struct Victim {
        id: String,
        username: String,
    }

    async fn create_victim(server: &TestServer, label: &str) -> Victim {
        let admin = admin_token(server).await;
        let username = format!("{label}-{}", Uuid::new_v4().simple());

        let created = server
            .post(&format!("/realms/{}/users", realm()))
            .add_header("Authorization", bearer(&admin))
            .json(&json!({
                "username": username,
                "email": format!("{username}@test.local"),
                "email_verified": true,
                "firstname": "Vic",
                "lastname": "Tim",
            }))
            .await;

        assert_eq!(
            created.status_code(),
            200,
            "creating {username} failed: {}",
            created.text()
        );

        let id = created.json::<Value>()["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        set_password(server, &admin, &id, VICTIM_PASSWORD).await;

        Victim { id, username }
    }

    async fn set_password(server: &TestServer, admin_token: &str, user_id: &str, password: &str) {
        let response = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm(),
                user_id
            ))
            .add_header("Authorization", bearer(admin_token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "setting the password of {user_id} failed: {}",
            response.text()
        );
    }

    async fn assert_token_still_works(server: &TestServer, access_token: &str, victim: &Victim) {
        let response = userinfo(server, access_token).await;
        let body = response.text();

        assert_eq!(
            response.status_code(),
            200,
            "the freshly issued access token was refused before any revocation: {body}"
        );

        let claims: Value = serde_json::from_str(&body).expect("userinfo JSON");
        assert_eq!(
            claims["sub"].as_str(),
            Some(victim.id.as_str()),
            "userinfo answered for the wrong subject: {body}"
        );
        assert_eq!(
            claims["preferred_username"].as_str(),
            Some(victim.username.as_str()),
            "userinfo answered for the wrong user: {body}"
        );
    }

    async fn assert_token_is_dead(server: &TestServer, access_token: &str, victim: &Victim) {
        let response = userinfo(server, access_token).await;
        let body = response.text();

        assert_ne!(
            response.status_code(),
            200,
            "the access token still opens userinfo after revocation: {body}"
        );
        assert!(
            !body.contains(&victim.id) && !body.contains(&victim.username),
            "userinfo still returned the user's identity to a revoked token: {body}"
        );
        assert_eq!(
            response.status_code(),
            401,
            "a revoked token must read as unauthenticated (RFC 6750 invalid_token), got: {body}"
        );
    }

    async fn assert_refresh_is_dead(server: &TestServer, refresh_token: &str) {
        let response = refresh(server, refresh_token).await;
        let body = response.text();

        assert_ne!(
            response.status_code(),
            200,
            "the refresh token still minted a new token pair after revocation: {body}"
        );

        let parsed: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
        assert!(
            parsed.get("access_token").and_then(Value::as_str).is_none(),
            "the refusal still carried an access token: {body}"
        );
        assert!(
            parsed
                .get("refresh_token")
                .and_then(Value::as_str)
                .is_none(),
            "the refusal still carried a refresh token: {body}"
        );
    }

    async fn sole_session_of(server: &TestServer, token: &str, victim: &Victim) -> String {
        let sessions = list_sessions(server, token, &victim.id).await;

        assert_eq!(
            sessions.len(),
            1,
            "expected exactly one session for {}, got: {sessions:?}",
            victim.username
        );

        sessions[0]["id"].as_str().expect("session id").to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn revoking_a_session_kills_the_tokens_it_minted() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "revoke").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            assert_token_still_works(&server, access, &victim).await;

            let session_id = sole_session_of(&server, access, &victim).await;
            assert_eq!(
                sid_claim(access).as_deref(),
                Some(session_id.as_str()),
                "the access token must name the session about to be revoked"
            );

            let revoked = revoke_session(&server, access, &victim.id, &session_id).await;
            assert_eq!(
                revoked.status_code(),
                204,
                "revoking the session failed: {}",
                revoked.text()
            );

            assert_token_is_dead(&server, access, &victim).await;
            assert_refresh_is_dead(&server, refresh_token).await;

            let admin = admin_token(&server).await;
            let remaining = list_sessions(&server, &admin, &victim.id).await;
            assert!(
                remaining.is_empty(),
                "the revoked session is still listed: {remaining:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn disabling_an_account_kills_its_outstanding_tokens() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "disabled").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            assert_token_still_works(&server, access, &victim).await;

            let admin = admin_token(&server).await;
            let disabled = server
                .put(&format!("/realms/{}/users/{}", realm(), victim.id))
                .add_header("Authorization", bearer(&admin))
                .json(&json!({ "enabled": false }))
                .await;

            assert_eq!(
                disabled.status_code(),
                200,
                "disabling {} failed: {}",
                victim.username,
                disabled.text()
            );
            assert_eq!(
                disabled.json::<Value>()["data"]["enabled"].as_bool(),
                Some(false),
                "the account was not actually disabled: {}",
                disabled.text()
            );

            assert_token_is_dead(&server, access, &victim).await;
            assert_refresh_is_dead(&server, refresh_token).await;

            let relogin = password_grant(&server, &victim.username, VICTIM_PASSWORD).await;
            assert_ne!(
                relogin.status_code(),
                200,
                "a disabled account still logged in: {}",
                relogin.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn changing_the_password_kills_sessions_opened_with_the_old_one() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "pwreset").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            assert_token_still_works(&server, access, &victim).await;

            let admin = admin_token(&server).await;
            set_password(&server, &admin, &victim.id, ROTATED_PASSWORD).await;

            assert_token_is_dead(&server, access, &victim).await;
            assert_refresh_is_dead(&server, refresh_token).await;

            let fresh = login(&server, &victim.username, ROTATED_PASSWORD).await;
            let fresh_access = fresh["access_token"].as_str().expect("access_token");
            assert_token_still_works(&server, fresh_access, &victim).await;
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn a_revoked_session_can_no_longer_be_rotated_into_a_fresh_token_pair() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "rotation").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access1 = tokens["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();
            let refresh1 = tokens["refresh_token"]
                .as_str()
                .expect("refresh_token")
                .to_string();

            assert_token_still_works(&server, &access1, &victim).await;

            let rotated = refresh(&server, &refresh1).await;
            assert_eq!(
                rotated.status_code(),
                200,
                "rotation failed while the session was still alive: {}",
                rotated.text()
            );
            let rotated: Value = rotated.json();
            let access2 = rotated["access_token"]
                .as_str()
                .expect("rotated access_token")
                .to_string();
            let refresh2 = rotated["refresh_token"]
                .as_str()
                .expect("rotated refresh_token")
                .to_string();

            assert_ne!(refresh1, refresh2, "rotation must mint a new refresh token");
            assert_token_still_works(&server, &access2, &victim).await;

            let session_id = sole_session_of(&server, &access2, &victim).await;
            assert_eq!(
                sid_claim(&access2).as_deref(),
                Some(session_id.as_str()),
                "the rotated access token lost its session binding"
            );

            let revoked = revoke_session(&server, &access2, &victim.id, &session_id).await;
            assert_eq!(
                revoked.status_code(),
                204,
                "revoking the session failed: {}",
                revoked.text()
            );

            assert_refresh_is_dead(&server, &refresh2).await;
            assert_token_is_dead(&server, &access2, &victim).await;

            assert_refresh_is_dead(&server, &refresh1).await;
            assert_token_is_dead(&server, &access1, &victim).await;
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn an_untouched_session_keeps_working_and_still_refreshes() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "healthy").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            assert!(
                sid_claim(access).is_some(),
                "the password grant must bind its token to a session"
            );
            assert_token_still_works(&server, access, &victim).await;

            assert_token_still_works(&server, access, &victim).await;

            let rotated = refresh(&server, refresh_token).await;
            assert_eq!(
                rotated.status_code(),
                200,
                "a normal refresh was refused: {}",
                rotated.text()
            );
            let rotated: Value = rotated.json();
            let rotated_access = rotated["access_token"]
                .as_str()
                .expect("rotated access_token");

            assert_token_still_works(&server, rotated_access, &victim).await;

            let session_id = sole_session_of(&server, rotated_access, &victim).await;
            assert_eq!(
                sid_claim(access).as_deref(),
                Some(session_id.as_str()),
                "the session backing the original token disappeared without a revocation"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn logging_out_kills_the_tokens_of_the_session_it_ends() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "logout").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");
            let id_token = tokens["id_token"]
                .as_str()
                .expect("the openid scope must yield an id_token to log out with");

            assert_token_still_works(&server, access, &victim).await;

            let session_id = sole_session_of(&server, access, &victim).await;
            assert_eq!(
                claim_str(id_token, "sid").as_deref(),
                Some(session_id.as_str()),
                "the id_token must mirror the sid of the session it can end"
            );

            let logged_out = logout(&server, id_token).await;
            assert_eq!(
                logged_out.status_code(),
                204,
                "logout failed: {}",
                logged_out.text()
            );

            assert_token_is_dead(&server, access, &victim).await;
            assert_refresh_is_dead(&server, refresh_token).await;

            let admin = admin_token(&server).await;
            let remaining = list_sessions(&server, &admin, &victim.id).await;
            assert!(
                remaining.is_empty(),
                "the logged-out session is still listed: {remaining:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn revoking_one_session_leaves_the_users_other_sessions_alone() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "twosess").await;

            let first = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let second = login(&server, &victim.username, VICTIM_PASSWORD).await;

            let access_a = first["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();
            let refresh_a = first["refresh_token"]
                .as_str()
                .expect("refresh_token")
                .to_string();
            let access_b = second["access_token"]
                .as_str()
                .expect("access_token")
                .to_string();
            let refresh_b = second["refresh_token"]
                .as_str()
                .expect("refresh_token")
                .to_string();

            let sid_a = sid_claim(&access_a).expect("session for the first login");
            let sid_b = sid_claim(&access_b).expect("session for the second login");
            assert_ne!(
                sid_a, sid_b,
                "two logins must open two distinct sessions, or this test proves nothing"
            );

            assert_token_still_works(&server, &access_a, &victim).await;
            assert_token_still_works(&server, &access_b, &victim).await;

            let sessions = list_sessions(&server, &access_a, &victim.id).await;
            assert_eq!(
                sessions.len(),
                2,
                "expected both sessions to be listed, got: {sessions:?}"
            );

            let revoked = revoke_session(&server, &access_a, &victim.id, &sid_a).await;
            assert_eq!(
                revoked.status_code(),
                204,
                "revoking the first session failed: {}",
                revoked.text()
            );

            assert_token_is_dead(&server, &access_a, &victim).await;
            assert_refresh_is_dead(&server, &refresh_a).await;

            assert_token_still_works(&server, &access_b, &victim).await;
            let rotated = refresh(&server, &refresh_b).await;
            assert_eq!(
                rotated.status_code(),
                200,
                "the surviving session lost the right to refresh: {}",
                rotated.text()
            );
            let rotated_access = rotated.json::<Value>()["access_token"]
                .as_str()
                .expect("rotated access_token")
                .to_string();
            assert_token_still_works(&server, &rotated_access, &victim).await;

            let remaining = list_sessions(&server, &access_b, &victim.id).await;
            let remaining_ids: Vec<&str> =
                remaining.iter().filter_map(|s| s["id"].as_str()).collect();
            assert_eq!(
                remaining_ids,
                vec![sid_b.as_str()],
                "exactly the revoked session should be gone, got: {remaining:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn client_credentials_tokens_carry_no_session_and_keep_working() {
        rt().block_on(async {
            let server = make_server();
            let (client_id, client_secret) = create_service_client(&server).await;

            let response = server
                .post(&format!(
                    "/realms/{}/protocol/openid-connect/token",
                    realm()
                ))
                .form(&[
                    ("grant_type", "client_credentials"),
                    ("client_id", client_id.as_str()),
                    ("client_secret", client_secret.as_str()),
                    ("scope", "openid profile"),
                ])
                .await;

            assert_eq!(
                response.status_code(),
                200,
                "the client_credentials grant failed: {}",
                response.text()
            );

            let tokens: Value = response.json();
            let access = tokens["access_token"].as_str().expect("access_token");

            assert!(
                sid_claim(access).is_none(),
                "client_credentials must not invent an SSO session for a service account"
            );

            for attempt in 1..=2 {
                let probe = userinfo(&server, access).await;
                assert_eq!(
                    probe.status_code(),
                    200,
                    "attempt {attempt}: a service-account token was refused for having no session: {}",
                    probe.text()
                );
            }

            let service_account_id =
                claim_str(access, "sub").expect("the token must name its service account");
            let admin = admin_token(&server).await;
            let sessions = list_sessions(&server, &admin, &service_account_id).await;
            assert!(
                sessions.is_empty(),
                "client_credentials opened a session nobody asked for: {sessions:?}"
            );
        });
    }
}
