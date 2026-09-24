/// Integration tests for the self-service account security surface (#1482):
/// `POST /me/reauthenticate`, `PUT /me/password`, `GET /me/credentials`,
/// `/me/mfa/otp` and `/me/passkeys`.
///
/// What they hold the surface to: no sensitive operation goes through without a
/// live elevation, an elevation only ever unlocks the account and the session it
/// was minted for, a refused re-authentication mints nothing, a password change
/// rotates the credential while sparing the caller's own session, and the
/// credential listing never hands back secret material.
///
/// Each test acts through its own dedicated user (created via the admin API,
/// never the shared `admin` account) so the tests stay parallel-safe against
/// the one realm the shared harness provisions.
///
/// Require a running PostgreSQL instance. Marked `#[ignore]` so they don't
/// block regular `cargo test` runs. Run them explicitly with:
///
///   cargo test -p ferriskey-api --test account_security_test -- --ignored
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
    use axum_test::{TestResponse, TestServer};
    use base64::{Engine, engine::general_purpose};
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

    const ADMIN_PASSWORD: &str = "admin_pass_1234!";
    const INITIAL_PASSWORD: &str = "S3cret-Password!";
    const ROTATED_PASSWORD: &str = "R0tated-Passw0rd!";

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

        let schema = format!("test_account_security_{}", Uuid::new_v4().simple());
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

    fn realm() -> String {
        ctx().realm_name.clone()
    }

    fn pool() -> &'static sqlx::PgPool {
        &ctx().pool
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    async fn request_token(
        srv: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> TestResponse {
        srv.post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
                ("scope", "openid profile"),
            ])
            .await
    }

    async fn login(srv: &TestServer, realm: &str, username: &str, password: &str) -> String {
        let resp = request_token(srv, realm, username, password).await;

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

    async fn admin_token(srv: &TestServer, realm: &str) -> String {
        login(srv, realm, "admin", ADMIN_PASSWORD).await
    }

    struct TestUser {
        id: String,
        username: String,
        token: String,
    }

    async fn create_test_user_and_login(
        srv: &TestServer,
        realm: &str,
        admin_token: &str,
    ) -> TestUser {
        let username = format!("account-security-{}", Uuid::new_v4().simple());

        let create_resp = srv
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            create_resp.status_code(),
            200,
            "user creation failed: {}",
            create_resp.text()
        );
        let id = create_resp.json::<Value>()["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let pw_resp = srv
            .put(&format!("/realms/{}/users/{}/reset-password", realm, id))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "value": INITIAL_PASSWORD,
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

        let token = login(srv, realm, &username, INITIAL_PASSWORD).await;

        TestUser {
            id,
            username,
            token,
        }
    }

    async fn reauthenticate(
        srv: &TestServer,
        realm: &str,
        token: &str,
        body: Value,
    ) -> TestResponse {
        srv.post(&format!("/realms/{}/users/me/reauthenticate", realm))
            .add_header("Authorization", auth_header(token))
            .json(&body)
            .await
    }

    async fn elevate(srv: &TestServer, realm: &str, token: &str, password: &str) -> String {
        let resp = reauthenticate(srv, realm, token, json!({ "password": password })).await;
        assert_eq!(
            resp.status_code(),
            200,
            "re-authentication failed: {}",
            resp.text()
        );

        resp.json::<Value>()["elevation_id"]
            .as_str()
            .expect("elevation_id")
            .to_string()
    }

    async fn change_password(
        srv: &TestServer,
        realm: &str,
        token: &str,
        elevation_id: &str,
        current: &str,
        new: &str,
    ) -> TestResponse {
        srv.put(&format!("/realms/{}/users/me/password", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "elevation_id": elevation_id,
                "current_password": current,
                "new_password": new,
            }))
            .await
    }

    async fn list_credentials(srv: &TestServer, realm: &str, token: &str) -> TestResponse {
        srv.get(&format!("/realms/{}/users/me/credentials", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn service_account_token(srv: &TestServer, realm: &str, admin_token: &str) -> String {
        let client_id = format!("account-security-client-{}", Uuid::new_v4().simple());

        let created = srv
            .post(&format!("/realms/{}/clients", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "client_id": client_id,
                "name": "Account Security Machine Client",
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": true,
                "direct_access_grants_enabled": false,
                "enabled": true,
            }))
            .await;
        assert_eq!(
            created.status_code(),
            201,
            "confidential client creation failed: {}",
            created.text()
        );
        let secret = created.json::<Value>()["client_secret"]
            .as_str()
            .expect("confidential client secret")
            .to_string();

        let encoded = general_purpose::STANDARD.encode(format!("{client_id}:{secret}"));
        let resp = srv
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .add_header(
                "Authorization",
                format!("Basic {encoded}")
                    .parse::<HeaderValue>()
                    .expect("valid header value"),
            )
            .form(&[("grant_type", "client_credentials")])
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "client_credentials grant failed: {}",
            resp.text()
        );

        resp.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    fn totp_code_for(secret_base32: &str) -> String {
        let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, secret_base32)
            .expect("the enrolled secret decodes as base32");

        let counter = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after the unix epoch")
            .as_secs()
            / 30;

        let mut mac = Hmac::<Sha1>::new_from_slice(&secret).expect("hmac accepts any key length");
        mac.update(&counter.to_be_bytes());
        let digest = mac.finalize().into_bytes();

        let offset = (digest[19] & 0x0f) as usize;
        let truncated = ((digest[offset] as u32 & 0x7f) << 24)
            | ((digest[offset + 1] as u32) << 16)
            | ((digest[offset + 2] as u32) << 8)
            | (digest[offset + 3] as u32);

        format!("{:06}", truncated % 1_000_000)
    }

    fn registration_payload() -> Value {
        json!({
            "id": "AAAA",
            "rawId": "AAAA",
            "response": { "attestationObject": "AAAA", "clientDataJSON": "AAAA" },
            "type": "public-key",
            "extensions": {},
        })
    }

    async fn elevation_count(user_id: &str) -> i64 {
        let user_id = Uuid::parse_str(user_id).expect("user id");
        sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM account_elevations WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool())
            .await
            .expect("count elevations")
            .0
    }

    async fn stored_secret_of(user_id: &str, credential_type: &str) -> String {
        let user_id = Uuid::parse_str(user_id).expect("user id");
        sqlx::query_as::<_, (String,)>(
            "SELECT secret_data FROM credentials WHERE user_id = $1 AND credential_type = $2",
        )
        .bind(user_id)
        .bind(credential_type)
        .fetch_one(pool())
        .await
        .expect("credential row")
        .0
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn changing_the_password_without_an_elevation_is_refused() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let refused = change_password(
                &srv,
                &realm,
                &user.token,
                &Uuid::new_v4().to_string(),
                INITIAL_PASSWORD,
                ROTATED_PASSWORD,
            )
            .await;

            assert_eq!(
                refused.status_code(),
                403,
                "a password change with no elevation must be refused: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(body["reason"], json!("elevation_required"), "{body}");

            let unchanged = request_token(&srv, &realm, &user.username, INITIAL_PASSWORD).await;
            assert_eq!(
                unchanged.status_code(),
                200,
                "the refused change must have left the password in place: {}",
                unchanged.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn an_elevation_minted_for_one_account_does_not_unlock_another() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let owner = create_test_user_and_login(&srv, &realm, &admin).await;
            let neighbour = create_test_user_and_login(&srv, &realm, &admin).await;

            let elevation_id = elevate(&srv, &realm, &owner.token, INITIAL_PASSWORD).await;

            let refused = change_password(
                &srv,
                &realm,
                &neighbour.token,
                &elevation_id,
                INITIAL_PASSWORD,
                ROTATED_PASSWORD,
            )
            .await;

            assert_eq!(
                refused.status_code(),
                403,
                "another account's elevation must never authorise this caller: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(body["reason"], json!("elevation_required"), "{body}");

            let unchanged =
                request_token(&srv, &realm, &neighbour.username, INITIAL_PASSWORD).await;
            assert_eq!(
                unchanged.status_code(),
                200,
                "the refused change must have left the password in place: {}",
                unchanged.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn an_elevation_minted_in_one_session_does_not_unlock_another_session_of_the_same_user() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let second = request_token(&srv, &realm, &user.username, INITIAL_PASSWORD).await;
            assert_eq!(
                second.status_code(),
                200,
                "the second sign-in must succeed: {}",
                second.text()
            );
            let second: Value = second.json();
            let second_token = second["access_token"]
                .as_str()
                .expect("the second session must carry an access token")
                .to_string();

            let elevation_id = elevate(&srv, &realm, &user.token, INITIAL_PASSWORD).await;

            let refused = change_password(
                &srv,
                &realm,
                &second_token,
                &elevation_id,
                INITIAL_PASSWORD,
                ROTATED_PASSWORD,
            )
            .await;

            assert_eq!(
                refused.status_code(),
                403,
                "an elevation bound to another session must not authorise this one: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(body["reason"], json!("elevation_required"), "{body}");

            let unchanged = request_token(&srv, &realm, &user.username, INITIAL_PASSWORD).await;
            assert_eq!(
                unchanged.status_code(),
                200,
                "the refused change must have left the password in place: {}",
                unchanged.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn a_reauthentication_with_the_wrong_password_mints_no_elevation() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let refused = reauthenticate(
                &srv,
                &realm,
                &user.token,
                json!({ "password": "not-the-password" }),
            )
            .await;

            assert_eq!(
                refused.status_code(),
                401,
                "a wrong password must not re-authenticate: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(body["reason"], json!("invalid_password"), "{body}");
            assert!(
                body["elevation_id"].is_null(),
                "a refusal must not hand back an elevation: {body}"
            );
            assert_eq!(
                elevation_count(&user.id).await,
                0,
                "a refused re-authentication must leave no elevation behind"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn an_elevation_request_must_carry_exactly_one_proof() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let both = reauthenticate(
                &srv,
                &realm,
                &user.token,
                json!({ "password": INITIAL_PASSWORD, "otp_code": "123456" }),
            )
            .await;
            assert_eq!(
                both.status_code(),
                400,
                "two proofs at once must be refused: {}",
                both.text()
            );
            let body: Value = both.json();
            assert_eq!(body["reason"], json!("invalid_elevation_proof"), "{body}");

            let neither = reauthenticate(&srv, &realm, &user.token, json!({})).await;
            assert_eq!(
                neither.status_code(),
                400,
                "a request carrying no proof must be refused: {}",
                neither.text()
            );

            assert_eq!(
                elevation_count(&user.id).await,
                0,
                "a malformed request must leave no elevation behind"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn a_reauthentication_then_a_password_change_rotates_the_credential() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let elevation_id = elevate(&srv, &realm, &user.token, INITIAL_PASSWORD).await;
            assert_eq!(
                elevation_count(&user.id).await,
                1,
                "an accepted password must mint exactly one elevation"
            );

            let changed = change_password(
                &srv,
                &realm,
                &user.token,
                &elevation_id,
                INITIAL_PASSWORD,
                ROTATED_PASSWORD,
            )
            .await;
            assert_eq!(
                changed.status_code(),
                200,
                "an elevated password change must go through: {}",
                changed.text()
            );
            assert_eq!(
                elevation_count(&user.id).await,
                0,
                "a password change must burn the elevation that authorised it"
            );

            let with_new = request_token(&srv, &realm, &user.username, ROTATED_PASSWORD).await;
            assert_eq!(
                with_new.status_code(),
                200,
                "the new password must sign the account in: {}",
                with_new.text()
            );
            assert!(with_new.json::<Value>()["access_token"].is_string());

            let with_old = request_token(&srv, &realm, &user.username, INITIAL_PASSWORD).await;
            assert_ne!(
                with_old.status_code(),
                200,
                "the replaced password must no longer sign the account in: {}",
                with_old.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn a_password_change_leaves_the_caller_own_session_signed_in() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;

            let elevation_id = elevate(&srv, &realm, &user.token, INITIAL_PASSWORD).await;

            let changed = change_password(
                &srv,
                &realm,
                &user.token,
                &elevation_id,
                INITIAL_PASSWORD,
                ROTATED_PASSWORD,
            )
            .await;
            assert_eq!(
                changed.status_code(),
                200,
                "an elevated password change must go through: {}",
                changed.text()
            );

            let still_signed_in = list_credentials(&srv, &realm, &user.token).await;
            assert_eq!(
                still_signed_in.status_code(),
                200,
                "changing one's own password must not sign the caller out: {}",
                still_signed_in.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn listing_own_credentials_never_returns_secret_material() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;
            let elevation_id = elevate(&srv, &realm, &user.token, INITIAL_PASSWORD).await;

            let started = srv
                .post(&format!("/realms/{}/users/me/mfa/otp", realm))
                .add_header("Authorization", auth_header(&user.token))
                .json(&json!({ "elevation_id": elevation_id }))
                .await;
            assert_eq!(
                started.status_code(),
                200,
                "starting an OTP enrolment failed: {}",
                started.text()
            );
            let otp_secret = started.json::<Value>()["secret"]
                .as_str()
                .expect("enrolment secret")
                .to_string();

            let confirmed = srv
                .put(&format!("/realms/{}/users/me/mfa/otp", realm))
                .add_header("Authorization", auth_header(&user.token))
                .json(&json!({
                    "elevation_id": elevation_id,
                    "code": totp_code_for(&otp_secret),
                    "label": "Test authenticator",
                }))
                .await;
            assert_eq!(
                confirmed.status_code(),
                200,
                "confirming the OTP enrolment failed: {}",
                confirmed.text()
            );

            let password_hash = stored_secret_of(&user.id, "password").await;
            assert_eq!(
                stored_secret_of(&user.id, "otp").await,
                otp_secret,
                "the enrolled secret must be the one the account now holds"
            );

            let listed = list_credentials(&srv, &realm, &user.token).await;
            assert_eq!(
                listed.status_code(),
                200,
                "listing own credentials failed: {}",
                listed.text()
            );

            let raw = listed.text();
            assert!(
                !raw.contains(&password_hash),
                "the credential listing leaked the password hash: {raw}"
            );
            assert!(
                !raw.contains(&otp_secret),
                "the credential listing leaked the authenticator secret: {raw}"
            );

            let body: Value = listed.json();
            let entries = body["data"].as_array().expect("a credential array");
            let mut kinds = entries
                .iter()
                .map(|entry| {
                    entry["credential_type"]
                        .as_str()
                        .expect("credential_type")
                        .to_string()
                })
                .collect::<Vec<String>>();
            kinds.sort();
            assert_eq!(kinds, vec!["otp".to_string(), "password".to_string()]);

            for entry in entries {
                let mut keys = entry
                    .as_object()
                    .expect("a credential object")
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<&str>>();
                keys.sort_unstable();
                assert_eq!(
                    keys,
                    ["created_at", "credential_type", "id", "label"],
                    "a credential entry carries a field beyond the safe projection: {entry}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn an_access_token_carrying_no_session_cannot_reach_the_account_surface() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let machine_token = service_account_token(&srv, &realm, &admin).await;

            let refused = reauthenticate(
                &srv,
                &realm,
                &machine_token,
                json!({ "password": INITIAL_PASSWORD }),
            )
            .await;

            assert_eq!(
                refused.status_code(),
                401,
                "a token that names no session must not reach the account surface: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(
                body["reason"],
                json!("session_bound_token_required"),
                "{body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn every_credential_management_route_refuses_a_request_without_an_elevation() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;
            let stale = Uuid::new_v4().to_string();
            let unknown_passkey = Uuid::new_v4();

            let refusals = vec![
                (
                    "start an OTP enrolment",
                    srv.post(&format!("/realms/{}/users/me/mfa/otp", realm))
                        .add_header("Authorization", auth_header(&user.token))
                        .json(&json!({ "elevation_id": stale }))
                        .await,
                ),
                (
                    "confirm an OTP enrolment",
                    srv.put(&format!("/realms/{}/users/me/mfa/otp", realm))
                        .add_header("Authorization", auth_header(&user.token))
                        .json(&json!({ "elevation_id": stale, "code": "123456" }))
                        .await,
                ),
                (
                    "disable the authenticator",
                    srv.delete(&format!("/realms/{}/users/me/mfa/otp", realm))
                        .add_header("Authorization", auth_header(&user.token))
                        .json(&json!({ "elevation_id": stale }))
                        .await,
                ),
                (
                    "start a passkey registration",
                    srv.post(&format!("/realms/{}/users/me/passkeys/options", realm))
                        .add_header("Authorization", auth_header(&user.token))
                        .json(&json!({ "elevation_id": stale }))
                        .await,
                ),
                (
                    "confirm a passkey registration",
                    srv.post(&format!("/realms/{}/users/me/passkeys", realm))
                        .add_header("Authorization", auth_header(&user.token))
                        .json(&json!({
                            "elevation_id": stale,
                            "credential": registration_payload(),
                        }))
                        .await,
                ),
                (
                    "delete a passkey",
                    srv.delete(&format!(
                        "/realms/{}/users/me/passkeys/{}",
                        realm, unknown_passkey
                    ))
                    .add_header("Authorization", auth_header(&user.token))
                    .json(&json!({ "elevation_id": stale }))
                    .await,
                ),
            ];

            for (operation, response) in refusals {
                assert_eq!(
                    response.status_code(),
                    403,
                    "{operation} must be refused without an elevation: {}",
                    response.text()
                );
                let body: Value = response.json();
                assert_eq!(
                    body["reason"],
                    json!("elevation_required"),
                    "{operation}: {body}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test account_security_test -- --ignored"]
    fn confirming_a_passkey_without_a_live_challenge_is_refused() {
        let srv = server();
        let realm = realm();
        rt().block_on(async {
            let admin = admin_token(&srv, &realm).await;
            let user = create_test_user_and_login(&srv, &realm, &admin).await;
            let elevation_id = elevate(&srv, &realm, &user.token, INITIAL_PASSWORD).await;

            let refused = srv
                .post(&format!("/realms/{}/users/me/passkeys", realm))
                .add_header("Authorization", auth_header(&user.token))
                .json(&json!({
                    "elevation_id": elevation_id,
                    "credential": registration_payload(),
                }))
                .await;

            assert_eq!(
                refused.status_code(),
                400,
                "an answer to a challenge that was never issued must be refused: {}",
                refused.text()
            );
            let body: Value = refused.json();
            assert_eq!(
                body["reason"],
                json!("webauthn_missing_challenge"),
                "{body}"
            );

            let listed = list_credentials(&srv, &realm, &user.token).await;
            let body: Value = listed.json();
            let entries = body["data"].as_array().expect("a credential array");
            assert!(
                entries.iter().all(
                    |entry| entry["credential_type"] != json!("webauthn-public-key-credential")
                ),
                "the refused confirmation must not have registered a passkey: {body}"
            );
        });
    }
}
