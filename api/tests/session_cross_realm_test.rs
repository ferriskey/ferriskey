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
    use sqlx::{Executor, PgPool, Row};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    const MASTER_REALM: &str = "master";
    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";

    const CONSOLE_CLIENT_ID: &str = "security-admin-console";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const S256_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

    const ALICE: &str = "alice";
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-Adm!";
    const VICTIM: &str = "victim";

    const TENANT_A_FLOW_MARKER: &str = "tenant-a-flow-witness";
    const TENANT_B_FLOW_MARKER: &str = "tenant-b-flow-witness";

    const TENANT_A_SESSION_MARKER: &str = "tenant-a-session-witness";
    const TENANT_B_SESSION_MARKER: &str = "tenant-b-session-witness";

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: PgPool,
        admin_token: String,
        tenant_a_id: Uuid,
        alice_id: Uuid,
        victim_id: Uuid,
        tenant_a_revocable_session: Uuid,
        tenant_a_survivor_session: Uuid,
        tenant_b_listable_session: Uuid,
        tenant_b_survivor_session: Uuid,
        tenant_b_bindable_session: Uuid,
        tenant_a_flow: Uuid,
        tenant_b_flow: Uuid,
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

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
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

        let schema = format!("session_cross_realm_test_{}", Uuid::new_v4().simple());

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

        service
            .initialize_application(StartupConfig {
                webapp_url: WEBAPP_URL.to_string(),
                master_realm_name: MASTER_REALM.to_string(),
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

        let server = TestServer::new(app.clone()).expect("create test server");
        let admin_token = password_grant(&server, MASTER_REALM, "admin", "admin").await;

        create_realm(&server, &admin_token, TENANT_A).await;
        create_realm(&server, &admin_token, TENANT_B).await;

        let tenant_a_id = realm_id(&pool, TENANT_A).await;
        let tenant_b_id = realm_id(&pool, TENANT_B).await;

        let alice_id = create_user(&server, &admin_token, TENANT_A, ALICE).await;
        set_password(&server, &admin_token, TENANT_A, alice_id, ALICE_PASSWORD).await;
        let victim_id = create_user(&server, &admin_token, TENANT_B, VICTIM).await;

        let tenant_a_revocable_session =
            seed_session(&pool, alice_id, tenant_a_id, TENANT_A_SESSION_MARKER).await;
        let tenant_a_survivor_session =
            seed_session(&pool, alice_id, tenant_a_id, TENANT_A_SESSION_MARKER).await;
        let tenant_b_listable_session =
            seed_session(&pool, victim_id, tenant_b_id, TENANT_B_SESSION_MARKER).await;
        let tenant_b_survivor_session =
            seed_session(&pool, victim_id, tenant_b_id, TENANT_B_SESSION_MARKER).await;
        let tenant_b_bindable_session =
            seed_session(&pool, victim_id, tenant_b_id, TENANT_B_SESSION_MARKER).await;

        let tenant_a_flow = seed_flow(&pool, tenant_a_id, TENANT_A_FLOW_MARKER).await;
        let tenant_b_flow = seed_flow(&pool, tenant_b_id, TENANT_B_FLOW_MARKER).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            admin_token,
            tenant_a_id,
            alice_id,
            victim_id,
            tenant_a_revocable_session,
            tenant_a_survivor_session,
            tenant_b_listable_session,
            tenant_b_survivor_session,
            tenant_b_bindable_session,
            tenant_a_flow,
            tenant_b_flow,
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

    fn ctx() -> &'static SharedContext {
        shared_ctx()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("bearer header is valid")
    }

    async fn password_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "token request for {username}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .unwrap_or_else(|| panic!("no access_token for {username}@{realm}: {body}"))
            .to_string()
    }

    async fn create_realm(server: &TestServer, admin_token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "name": name }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating realm {name} failed: {}",
            response.text()
        );
    }

    async fn realm_id(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|e| panic!("fetch realm {name}: {e}"))
            .get("id")
    }

    async fn create_user(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        username: &str,
    ) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": "Seeded",
                "lastname": "User",
                "email": format!("{username}@{realm}.local"),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "creating user {username}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created user {username}: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("user id {raw} is not a uuid: {e}"))
    }

    async fn set_password(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        user_id: Uuid,
        password: &str,
    ) {
        let response = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "setting the password of {user_id}@{realm} failed: {}",
            response.text()
        );
    }

    async fn seed_session(pool: &PgPool, user_id: Uuid, realm: Uuid, marker: &str) -> Uuid {
        let session_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, realm_id, user_agent, ip_address, created_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW() + INTERVAL '1 hour')",
        )
        .bind(session_id)
        .bind(user_id)
        .bind(realm)
        .bind(marker)
        .bind("203.0.113.7")
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("insert session for {user_id}: {e}"));

        session_id
    }

    async fn seed_flow(pool: &PgPool, realm: Uuid, marker: &str) -> Uuid {
        let flow_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO compass_flows (id, realm_id, grant_type, status, user_agent, started_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
        )
        .bind(flow_id)
        .bind(realm)
        .bind("password")
        .bind("pending")
        .bind(marker)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("insert compass flow in {realm}: {e}"));

        flow_id
    }

    async fn start_console_authorization(server: &TestServer, realm: &str) -> TestResponse {
        let redirect_uri = console_callback(realm);

        let response = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", CONSOLE_CLIENT_ID)
            .add_query_param("redirect_uri", redirect_uri.as_str())
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;

        let status = response.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth on {realm}, got {status}: {}",
            response.text()
        );

        response
    }

    fn console_callback(realm: &str) -> String {
        format!("{WEBAPP_URL}/realms/{realm}/authentication/callback")
    }

    async fn authenticate_alice(server: &TestServer) -> (Uuid, String) {
        let authorize = start_console_authorization(server, TENANT_A).await;
        let session_cookie = authorize.cookie("FERRISKEY_SESSION");
        let auth_session_id = Uuid::parse_str(session_cookie.value())
            .expect("the authorization session cookie carries a uuid");

        let login = server
            .post(&format!("/realms/{}/login-actions/authenticate", TENANT_A))
            .add_cookie(session_cookie)
            .add_query_param("client_id", CONSOLE_CLIENT_ID)
            .json(&json!({ "username": ALICE, "password": ALICE_PASSWORD }))
            .await;

        assert_eq!(
            login.status_code(),
            200,
            "alice must be able to log in to tenant-a: {}",
            login.text()
        );

        let body: Value = login.json();
        let callback = body["url"]
            .as_str()
            .unwrap_or_else(|| panic!("a successful login carries a redirect url: {body}"))
            .to_string();

        let code = url::Url::parse(&callback)
            .expect("the callback url is valid")
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.to_string())
            .expect("the callback url carries an authorization code");

        (auth_session_id, code)
    }

    async fn exchange_code(server: &TestServer, code: &str) -> Uuid {
        let redirect_uri = console_callback(TENANT_A);

        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                TENANT_A
            ))
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", CONSOLE_CLIENT_ID),
                ("code", code),
                ("redirect_uri", redirect_uri.as_str()),
                ("code_verifier", S256_VERIFIER),
            ])
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "exchanging a tenant-a authorization code failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let access_token = body["access_token"]
            .as_str()
            .unwrap_or_else(|| panic!("the grant must hand back an access token: {body}"));

        sid_claim(access_token)
    }

    fn sid_claim(access_token: &str) -> Uuid {
        let payload = access_token
            .split('.')
            .nth(1)
            .expect("an access token has three segments");
        let raw = URL_SAFE_NO_PAD
            .decode(payload)
            .expect("the access token payload is base64url");
        let claims: Value = serde_json::from_slice(&raw).expect("the payload is json");
        let sid = claims["sid"]
            .as_str()
            .unwrap_or_else(|| panic!("the grant must bind its token to a session: {claims}"));

        Uuid::parse_str(sid).expect("the sid claim is a uuid")
    }

    async fn bound_user_session(auth_session_id: Uuid) -> Option<Uuid> {
        sqlx::query("SELECT user_session_id FROM auth_sessions WHERE id = $1")
            .bind(auth_session_id)
            .fetch_one(&ctx().pool)
            .await
            .expect("query auth session")
            .get("user_session_id")
    }

    async fn rebind_user_session(auth_session_id: Uuid, session_id: Uuid) {
        sqlx::query("UPDATE auth_sessions SET user_session_id = $1 WHERE id = $2")
            .bind(session_id)
            .bind(auth_session_id)
            .execute(&ctx().pool)
            .await
            .expect("repoint the auth session at a foreign session");
    }

    async fn session_realm(session_id: Uuid) -> Option<Uuid> {
        sqlx::query("SELECT realm_id FROM user_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(&ctx().pool)
            .await
            .expect("query user session realm")
            .map(|row| row.get("realm_id"))
    }

    async fn session_exists(session_id: Uuid) -> bool {
        sqlx::query("SELECT 1 AS present FROM user_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_optional(&ctx().pool)
            .await
            .expect("query user session")
            .is_some()
    }

    async fn list_sessions(server: &TestServer, realm: &str, user_id: Uuid) -> TestResponse {
        server
            .get(&format!("/realms/{}/users/{}/sessions", realm, user_id))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn revoke_session(
        server: &TestServer,
        realm: &str,
        user_id: Uuid,
        session_id: Uuid,
    ) -> TestResponse {
        server
            .delete(&format!(
                "/realms/{}/users/{}/sessions/{}",
                realm, user_id, session_id
            ))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn get_flow(server: &TestServer, realm: &str, flow_id: Uuid) -> TestResponse {
        server
            .get(&format!("/realms/{}/compass/v1/flows/{}", realm, flow_id))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn list_flows(server: &TestServer, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/compass/v1/flows", realm))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    fn assert_ok(response: &TestResponse, what: &str) -> String {
        let body = response.text();

        assert_eq!(
            response.status_code(),
            200,
            "{what}: expected 200, got {} with body {body}",
            response.status_code()
        );

        body
    }

    fn assert_not_found(response: &TestResponse, what: &str) {
        assert_eq!(
            response.status_code(),
            404,
            "{what}: expected 404, got {} with body {}",
            response.status_code(),
            response.text()
        );
    }

    fn assert_body_free_of(response_body: &str, needles: &[&str], what: &str) {
        for needle in needles {
            assert!(
                !response_body.contains(needle),
                "{what}: the response leaked {needle:?}; body was {response_body}"
            );
        }
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn listing_the_sessions_of_a_user_of_another_realm_leaks_nothing() {
        rt().block_on(async {
            let server = make_server();
            let victim_session = ctx().tenant_b_listable_session.to_string();

            let witness = list_sessions(&server, TENANT_B, ctx().victim_id).await;
            let witness_body = assert_ok(
                &witness,
                "listing the sessions of a tenant-b user from the tenant-b url",
            );
            assert!(
                witness_body.contains(&victim_session),
                "the session is not listed from its own realm, so the refusal below would prove \
                 nothing: {witness_body}"
            );
            assert!(
                witness_body.contains(TENANT_B_SESSION_MARKER),
                "the listing does not carry the session metadata, so an empty body would satisfy \
                 the refusal below for the wrong reason: {witness_body}"
            );

            let refused = list_sessions(&server, TENANT_A, ctx().victim_id).await;
            let refused_body = assert_ok(
                &refused,
                "listing the sessions of a tenant-b user from the tenant-a url",
            );

            assert_body_free_of(
                &refused_body,
                &[victim_session.as_str(), TENANT_B_SESSION_MARKER],
                "the listing of a tenant-b user's sessions through the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn revoking_the_session_of_a_user_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let accepted = revoke_session(
                &server,
                TENANT_A,
                ctx().alice_id,
                ctx().tenant_a_revocable_session,
            )
            .await;
            assert_eq!(
                accepted.status_code(),
                204,
                "a tenant-a session must be revocable from the tenant-a url, otherwise the \
                 refusal below proves nothing: {} {}",
                accepted.status_code(),
                accepted.text()
            );
            assert!(
                !session_exists(ctx().tenant_a_revocable_session).await,
                "the accepted revocation did not reach the database, so the refusal below is not \
                 evidence of scoping"
            );

            let refused = revoke_session(
                &server,
                TENANT_A,
                ctx().victim_id,
                ctx().tenant_b_survivor_session,
            )
            .await;

            assert_not_found(
                &refused,
                "revoking a tenant-b session from the tenant-a url",
            );
            assert!(
                session_exists(ctx().tenant_b_survivor_session).await,
                "the tenant-b session was revoked through the tenant-a url"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn revoking_a_session_through_another_realms_url_is_refused_in_both_directions() {
        rt().block_on(async {
            let server = make_server();

            let refused = revoke_session(
                &server,
                TENANT_B,
                ctx().alice_id,
                ctx().tenant_a_survivor_session,
            )
            .await;

            assert_not_found(
                &refused,
                "revoking a tenant-a session from the tenant-b url",
            );
            assert!(
                session_exists(ctx().tenant_a_survivor_session).await,
                "the tenant-a session was revoked through the tenant-b url"
            );

            let still_listed = list_sessions(&server, TENANT_A, ctx().alice_id).await;
            let still_listed_body = assert_ok(
                &still_listed,
                "listing the sessions of a tenant-a user from the tenant-a url",
            );
            assert!(
                still_listed_body.contains(&ctx().tenant_a_survivor_session.to_string()),
                "the tenant-a session is no longer listed in its own realm: {still_listed_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn revoking_a_session_of_a_user_of_another_realm_under_its_own_realm_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let refused = revoke_session(
                &server,
                TENANT_B,
                ctx().alice_id,
                ctx().tenant_b_survivor_session,
            )
            .await;

            assert_not_found(
                &refused,
                "revoking a tenant-b session under a tenant-a user id",
            );
            assert!(
                session_exists(ctx().tenant_b_survivor_session).await,
                "the tenant-b session was revoked under a foreign user id"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn a_grant_never_mints_a_token_on_a_session_of_another_realm() {
        rt().block_on(async {
            let server = make_server();

            let (nominal_auth_session, nominal_code) = authenticate_alice(&server).await;
            let nominal_bound = bound_user_session(nominal_auth_session)
                .await
                .expect("finalizing a login must bind the authorization session to a session");
            let nominal_sid = exchange_code(&server, &nominal_code).await;
            assert_eq!(
                nominal_sid, nominal_bound,
                "the nominal grant does not resume the session it is bound to, so the assertions \
                 below would prove nothing"
            );

            let (tampered_auth_session, tampered_code) = authenticate_alice(&server).await;
            rebind_user_session(tampered_auth_session, ctx().tenant_b_bindable_session).await;
            let tampered_sid = exchange_code(&server, &tampered_code).await;

            assert_ne!(
                tampered_sid,
                ctx().tenant_b_bindable_session,
                "a tenant-a grant minted a token on a tenant-b session"
            );
            assert_eq!(
                session_realm(tampered_sid).await,
                Some(ctx().tenant_a_id),
                "the grant must open a fresh session in its own realm instead of adopting a \
                 foreign one"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn reading_a_compass_flow_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let foreign_flow = ctx().tenant_b_flow.to_string();

            let witness = get_flow(&server, TENANT_B, ctx().tenant_b_flow).await;
            let witness_body = assert_ok(
                &witness,
                "reading a tenant-b compass flow from the tenant-b url",
            );
            assert!(
                witness_body.contains(TENANT_B_FLOW_MARKER),
                "the flow is not readable from its own realm, so the refusal below would prove \
                 nothing: {witness_body}"
            );

            let refused = get_flow(&server, TENANT_A, ctx().tenant_b_flow).await;
            let refused_body = refused.text();

            assert_not_found(
                &refused,
                "reading a tenant-b compass flow from the tenant-a url",
            );
            assert_body_free_of(
                &refused_body,
                &[foreign_flow.as_str(), TENANT_B_FLOW_MARKER],
                "the read of a tenant-b compass flow through the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test session_cross_realm_test -- --ignored"]
    fn listing_compass_flows_does_not_leak_another_realms_flow() {
        rt().block_on(async {
            let server = make_server();

            let witness = list_flows(&server, TENANT_B).await;
            let witness_body = assert_ok(&witness, "listing the compass flows of tenant-b");
            assert!(
                witness_body.contains(TENANT_B_FLOW_MARKER),
                "tenant-b does not list its own flow, so the assertions below would prove \
                 nothing: {witness_body}"
            );

            let listed = list_flows(&server, TENANT_A).await;
            let listed_body = assert_ok(&listed, "listing the compass flows of tenant-a");
            assert!(
                listed_body.contains(TENANT_A_FLOW_MARKER)
                    && listed_body.contains(&ctx().tenant_a_flow.to_string()),
                "tenant-a does not list its own flow, so an empty listing would satisfy the \
                 assertion below for the wrong reason: {listed_body}"
            );
            assert_body_free_of(
                &listed_body,
                &[
                    ctx().tenant_b_flow.to_string().as_str(),
                    TENANT_B_FLOW_MARKER,
                ],
                "the compass flow listing of tenant-a",
            );
        });
    }
}
