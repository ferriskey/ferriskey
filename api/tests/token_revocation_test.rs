/// Integration tests: revocation actually cuts off already-issued tokens (FK-007).
///
/// An operator who detected a compromise and ran the standard playbook — revoke the
/// sessions, log out, disable the account, change the password — interrupted the
/// attacker in *none* of those cases. The `sid` claim was written into every token
/// and never read back; `revoke_session` deleted a row without touching a single
/// token; no lifecycle path revoked anything at all. The OpenAPI description of
/// `DELETE /users/{id}/sessions/{sid}` nevertheless promised "The session's tokens
/// are immediately invalidated".
///
/// Every test here follows the same shape, because the bug was never about status
/// codes: **get a token, prove it works, apply the remediation, prove it no longer
/// works.** The "prove it works" step is what makes the failure meaningful — without
/// it, a token rejected after revocation could just as well have been rejected all
/// along.
///
/// `GET /protocol/openid-connect/userinfo` is the probe for "does this access token
/// still open a protected resource": it goes through the bearer auth middleware, and
/// its body carries `sub` and `preferred_username`, so a leak is visible as data and
/// not merely as a 200.
///
/// Each test provisions its *own* user. Disabling an account and changing a password
/// both cut every session the user holds, so sharing the seeded admin between tests
/// running in parallel would have them revoke each other.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]`. Run with:
///
///   cargo test -p ferriskey-api --test token_revocation_test -- --ignored
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
    /// The direct-access-grant client every test logs in through.
    const CLI_CLIENT_ID: &str = "admin-cli";
    /// 16 chars, all four classes: satisfies the CNIL-compliant default policy
    /// (min 12, upper/lower/digit/special, >= 80 bits estimated entropy).
    const VICTIM_PASSWORD: &str = "Xq7#vLm2$pRt9Wz!";
    /// The replacement used by the password-change scenario.
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
        /// Kept alive for the process lifetime so the schema outlives the tests.
        #[allow(dead_code)]
        pool: PgPool,
    }

    // `router()` installs a process-global Prometheus recorder, so it can only be
    // built once per test binary (#1086). Every test shares this router and runtime,
    // which is why these are `#[test]` + `rt().block_on(..)` and not `#[tokio::test]`.
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

    /// Read a string claim straight off a JWT, without verifying the signature.
    fn claim_str(token: &str, name: &str) -> Option<String> {
        let payload = token.split('.').nth(1)?;
        let raw = URL_SAFE_NO_PAD.decode(payload).ok()?;
        let claims: Value = serde_json::from_slice(&raw).ok()?;
        claims.get(name)?.as_str().map(str::to_string)
    }

    /// The `sid` claim. The tests use it to prove the token they are about to kill
    /// really names the session they are about to revoke — otherwise "the token
    /// stopped working" would not be evidence that *this* revocation is what
    /// stopped it.
    fn sid_claim(access_token: &str) -> Option<String> {
        claim_str(access_token, "sid")
    }

    // ---- HTTP helpers ------------------------------------------------------

    /// `scope` is explicit because `userinfo` refuses a token without `openid`, and
    /// `profile` is what puts `preferred_username` in its answer — the two claims the
    /// assertions below read to show *whose* identity a token still buys.
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

    /// Log in and return the whole token response, asserting the grant succeeded.
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

    /// RP-initiated logout with nothing but the ID token, which is all a relying
    /// party holds. No `post_logout_redirect_uri`, so the endpoint answers 204
    /// instead of redirecting.
    async fn logout(server: &TestServer, id_token: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/logout",
                realm()
            ))
            .form(&[("id_token_hint", id_token)])
            .await
    }

    /// A confidential client with a service account, i.e. the `client_credentials`
    /// setup. Returns `(client_id, client_secret)`.
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

        let secret = created.json::<Value>()["secret"]
            .as_str()
            .expect("a confidential client must be given a secret")
            .to_string();

        (client_id, secret)
    }

    /// A user with a usable password, created through the public admin API so the
    /// test exercises the same provisioning path an operator would.
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

    // ---- assertions on effect, not on status codes -------------------------

    /// The token opens a protected resource and hands back this user's identity.
    /// This is the "before" half of every scenario: without it, a rejection after
    /// revocation proves nothing.
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

    /// The token no longer opens the protected resource *and* leaks no identity.
    /// The status code is checked too, but the load-bearing assertion is that the
    /// body no longer carries the user — a 200 with `sub` in it is the bug.
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

    /// The refresh token can no longer be exchanged for anything. Checked on the
    /// body, not the status: the failure that mattered was rotation quietly handing
    /// back a *working* new pair.
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

    /// The single session backing `access_token`, as the operator sees it on
    /// `GET /users/{id}/sessions`. Asserting there is exactly one keeps the
    /// experiment single-variable: the session revoked below is the one that minted
    /// this very token, which the `sid` cross-check then confirms.
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

    // ---- scenarios ---------------------------------------------------------

    /// Revoking a session must cut both halves of the grant it produced.
    ///
    /// Before the fix `revoke_session` deleted the `user_sessions` row and stopped
    /// there: the access token kept opening userinfo and the refresh token kept
    /// minting fresh pairs, for the full natural lifetime of the tokens. The
    /// endpoint returned 204 the whole time.
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

            // The operator's view must agree with reality: nothing left to revoke.
            let admin = admin_token(&server).await;
            let remaining = list_sessions(&server, &admin, &victim.id).await;
            assert!(
                remaining.is_empty(),
                "the revoked session is still listed: {remaining:?}"
            );
        });
    }

    /// Disabling an account must reach the grants already handed out.
    ///
    /// Before the fix `enabled = false` was one column write. The password grant
    /// refused new logins, and that was the whole of it — every access token already
    /// out there kept working, and refresh kept renewing them.
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

            // A disabled account cannot log back in either — so the revocation above
            // is the only thing standing between the attacker and the account.
            let relogin = password_grant(&server, &victim.username, VICTIM_PASSWORD).await;
            assert_ne!(
                relogin.status_code(),
                200,
                "a disabled account still logged in: {}",
                relogin.text()
            );
        });
    }

    /// Changing a password must invalidate the sessions opened with the old one.
    ///
    /// Before the fix, resetting a compromised password stored a new hash and left
    /// every session opened with the old one renewing itself indefinitely — the
    /// single most common remediation gesture, with no effect on the attacker.
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

            // The account is locked out of its old grants, not bricked: the new
            // password still logs in and the new token works.
            let fresh = login(&server, &victim.username, ROTATED_PASSWORD).await;
            let fresh_access = fresh["access_token"].as_str().expect("access_token");
            assert_token_still_works(&server, fresh_access, &victim).await;
        });
    }

    /// The one that matters most: rotation from a revoked session.
    ///
    /// Refresh tokens rotate, and the successor inherits the session binding. Before
    /// the fix nothing checked that binding, so a revoked session could launder
    /// itself into a brand-new, fully valid token pair — and then do it again, and
    /// again. Revocation was not merely late, it was unreachable.
    ///
    /// The refresh performed *before* the revocation is the control: it proves the
    /// refusal afterwards comes from the revocation and not from a broken grant.
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

            // Control: while the session lives, rotation works and yields a usable pair.
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

            // Rotation must not detach the pair from its session, or revocation would
            // have nothing left to grab.
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

            // The successor refresh token — the one rotation just handed out — must
            // now be worthless.
            assert_refresh_is_dead(&server, &refresh2).await;
            assert_token_is_dead(&server, &access2, &victim).await;

            // And the ancestor must not be a way back in either.
            assert_refresh_is_dead(&server, &refresh1).await;
            assert_token_is_dead(&server, &access1, &victim).await;
        });
    }

    /// Non-regression: a session nobody touched keeps working.
    ///
    /// Reading the `sid` claim back on every token validation is a new hard failure
    /// on the hottest path in the system. Without this test, a fix that rejected
    /// *every* token would look exactly like a fix that rejected the right ones.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn an_untouched_session_keeps_working_and_still_refreshes() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "healthy").await;

            let tokens = login(&server, &victim.username, VICTIM_PASSWORD).await;
            let access = tokens["access_token"].as_str().expect("access_token");
            let refresh_token = tokens["refresh_token"].as_str().expect("refresh_token");

            // The token is bound to a session, and that binding is not in itself a
            // reason to reject it.
            assert!(
                sid_claim(access).is_some(),
                "the password grant must bind its token to a session"
            );
            assert_token_still_works(&server, access, &victim).await;

            // Repeated use is fine: the session lookup happens on every validation.
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

            // Nothing was revoked as a side effect: the session is still there, and
            // still the same one.
            let session_id = sole_session_of(&server, rotated_access, &victim).await;
            assert_eq!(
                sid_claim(access).as_deref(),
                Some(session_id.as_str()),
                "the session backing the original token disappeared without a revocation"
            );
        });
    }

    /// RP-initiated logout must end the session, not just clear cookies.
    ///
    /// Before the fix `end_session` validated the `id_token_hint`, assembled a
    /// redirect and returned — the `sid` the ID token carries was read nowhere, so
    /// every token that session had minted survived the logout untouched. The user
    /// saw "you are logged out" while their tokens stayed live for hours.
    ///
    /// The status code is deliberately *not* the load-bearing assertion: a logout
    /// without `post_logout_redirect_uri` swallows an unusable hint and answers 204
    /// regardless (`core/src/domain/authentication/services.rs:3740`). Only the dead
    /// tokens distinguish a real logout from a cookie wipe.
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

            // The hint is only useful if it names the session — that mirroring is
            // what makes logout able to find anything to revoke.
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

    /// Revocation must be surgical: one session, not the account.
    ///
    /// `DELETE /users/{id}/sessions/{sid}` promises "Other sessions are unaffected".
    /// Every other test here would still pass if the cascade cut *every* token the
    /// user holds — an over-broad fix logs out the victim's phone, laptop and CI job
    /// because one of them was revoked, which is its own outage. This is the test
    /// that pins the blast radius.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_revocation_test -- --ignored"]
    fn revoking_one_session_leaves_the_users_other_sessions_alone() {
        rt().block_on(async {
            let server = make_server();
            let victim = create_victim(&server, "twosess").await;

            // Two independent logins — two devices, as far as the server can tell.
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

            // The revoked one is gone...
            assert_token_is_dead(&server, &access_a, &victim).await;
            assert_refresh_is_dead(&server, &refresh_a).await;

            // ...and the other one is untouched, including its ability to rotate.
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

            // The operator's view agrees: exactly the revoked one disappeared.
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

    /// Non-regression for machine-to-machine: `client_credentials` mints tokens that
    /// carry no `sid`, and those must keep working.
    ///
    /// `validate_session_binding` accepts a token with no `sid` on purpose — a
    /// service account has no SSO session to end, and every token issued before the
    /// migration carries no claim either, so enforcing a binding that was never
    /// recorded would be an outage rather than a hardening. That exemption is the
    /// one place where the new check deliberately does *not* fire, so it needs a
    /// test: without it, tightening the rule to "every token must name a session"
    /// would silently break every service account and every token in flight during a
    /// deploy.
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

            // Unbound, and still accepted — twice, so the session lookup cannot be
            // failing open only on a first, uncached validation.
            for attempt in 1..=2 {
                let probe = userinfo(&server, access).await;
                assert_eq!(
                    probe.status_code(),
                    200,
                    "attempt {attempt}: a service-account token was refused for having no session: {}",
                    probe.text()
                );
            }

            // And the service account really holds no session an operator could
            // revoke — the row was not created behind the scenes.
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
