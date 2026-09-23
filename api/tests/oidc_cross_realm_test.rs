#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::Router;
    use axum_test::{TestResponse, TestServer};
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
    const CONSOLE_CLIENT_ID: &str = "security-admin-console";
    const MASTER_REALM: &str = "master";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const S256_VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const ALICE_PASSWORD: &str = "Al1ce-Cross-Realm!";
    const BOB_PASSWORD: &str = "B0b-Cross-Realm!";

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: PgPool,
        tenant_a: String,
        tenant_b: String,
        tenant_a_id: Uuid,
        alice_username: String,
        alice_id: String,
        bob_username: String,
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

        let schema = format!("oidc_cross_realm_{}", Uuid::new_v4().simple());

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

        let server = TestServer::new(app.clone()).expect("create fixture server");
        let admin_token = direct_grant_token(&server, MASTER_REALM, "admin", "admin").await;

        let suffix = Uuid::new_v4().simple().to_string();
        let tenant_a = format!("tenant-a-{}", &suffix[..8]);
        let tenant_b = format!("tenant-b-{}", &suffix[..8]);

        create_realm(&server, &admin_token, &tenant_a).await;
        create_realm(&server, &admin_token, &tenant_b).await;

        let alice_username = format!("alice-{}", &suffix[..8]);
        let alice_id = create_user(&server, &admin_token, &tenant_a, &alice_username).await;
        set_password(&server, &admin_token, &tenant_a, &alice_id, ALICE_PASSWORD).await;

        let bob_username = format!("bob-{}", &suffix[..8]);
        let bob_id = create_user(&server, &admin_token, &tenant_b, &bob_username).await;
        set_password(&server, &admin_token, &tenant_b, &bob_id, BOB_PASSWORD).await;

        let tenant_a_id: Uuid = sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(&tenant_a)
            .fetch_one(&pool)
            .await
            .expect("fetch tenant a realm")
            .get("id");

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            tenant_a,
            tenant_b,
            tenant_a_id,
            alice_username,
            alice_id,
            bob_username,
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

    fn tenant_a() -> &'static str {
        ctx().tenant_a.as_str()
    }

    fn tenant_b() -> &'static str {
        ctx().tenant_b.as_str()
    }

    fn alice_username() -> &'static str {
        ctx().alice_username.as_str()
    }

    fn alice_id() -> &'static str {
        ctx().alice_id.as_str()
    }

    fn bob_username() -> &'static str {
        ctx().bob_username.as_str()
    }

    fn bearer(token: &str) -> String {
        format!("Bearer {}", token)
    }

    async fn direct_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
                ("scope", "openid profile email"),
            ])
            .await
    }

    async fn direct_grant_body(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> Value {
        let response = direct_grant(server, realm, username, password).await;
        let body = response.text();

        assert_eq!(
            response.status_code(),
            200,
            "the direct grant for {username}@{realm} must succeed: {body}"
        );

        serde_json::from_str(&body).expect("the token response must be JSON")
    }

    async fn direct_grant_token(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let body = direct_grant_body(server, realm, username, password).await;
        let token = body["access_token"]
            .as_str()
            .unwrap_or_else(|| {
                panic!("access_token in the response for {username}@{realm}: {body}")
            })
            .to_string();

        assert!(
            !token.is_empty(),
            "the access token handed to {username}@{realm} must not be empty: {body}"
        );

        token
    }

    async fn create_realm(server: &TestServer, token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", bearer(token))
            .json(&json!({ "name": name }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating realm {name} failed: {}",
            response.text()
        );
    }

    async fn create_user(server: &TestServer, token: &str, realm: &str, username: &str) -> String {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", bearer(token))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{}@test.local", username),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "creating user {username} in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("the created user carries an id: {body}"))
            .to_string()
    }

    async fn set_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", bearer(token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "setting the password of {user_id} in {realm} failed: {}",
            response.text()
        );
    }

    async fn admin_token(server: &TestServer) -> String {
        direct_grant_token(server, MASTER_REALM, "admin", "admin").await
    }

    async fn create_service_client(server: &TestServer, realm: &str) -> (String, String) {
        let admin = admin_token(server).await;
        let client_id = format!("svc-{}", Uuid::new_v4().simple());

        let created = server
            .post(&format!("/realms/{}/clients", realm))
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
            "creating the service client in {realm} failed: {}",
            created.text()
        );

        let body: Value = created.json();
        let secret = body["client_secret"]
            .as_str()
            .unwrap_or_else(|| panic!("a confidential client must be given a secret: {body}"))
            .to_string();

        (client_id, secret)
    }

    async fn service_account_of(client_id: &str) -> Uuid {
        sqlx::query(
            "SELECT u.id FROM users u JOIN clients c ON u.client_id = c.id WHERE c.client_id = $1",
        )
        .bind(client_id)
        .fetch_one(&ctx().pool)
        .await
        .expect("the service account of a service-enabled client must exist")
        .get("id")
    }

    async fn realm_of_user(user_id: Uuid) -> Uuid {
        sqlx::query("SELECT realm_id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(&ctx().pool)
            .await
            .expect("the user row must still exist")
            .get("realm_id")
    }

    async fn client_credentials(
        server: &TestServer,
        realm: &str,
        client_id: &str,
        client_secret: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client_id),
                ("client_secret", client_secret),
            ])
            .await
    }

    async fn introspect(
        server: &TestServer,
        realm: &str,
        client_id: &str,
        client_secret: &str,
        token: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token/introspect",
                realm
            ))
            .form(&[
                ("token", token),
                ("client_id", client_id),
                ("client_secret", client_secret),
            ])
            .await
    }

    async fn revoke(
        server: &TestServer,
        realm: &str,
        client_id: &str,
        token: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/revoke", realm))
            .form(&[("token", token), ("client_id", client_id)])
            .await
    }

    async fn userinfo(server: &TestServer, realm: &str, token: &str) -> TestResponse {
        server
            .get(&format!(
                "/realms/{}/protocol/openid-connect/userinfo",
                realm
            ))
            .add_header("Authorization", bearer(token))
            .await
    }

    async fn refresh(server: &TestServer, realm: &str, refresh_token: &str) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", "admin-cli"),
                ("refresh_token", refresh_token),
            ])
            .await
    }

    fn console_callback(realm: &str) -> String {
        format!("{WEBAPP_URL}/realms/{realm}/authentication/callback")
    }

    async fn authorization_code_of(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let authorize = server
            .get(&format!("/realms/{}/protocol/openid-connect/auth", realm))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", CONSOLE_CLIENT_ID)
            .add_query_param("redirect_uri", console_callback(realm).as_str())
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;

        let status = authorize.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth on {realm}, got {status}: {}",
            authorize.text()
        );

        let login = server
            .post(&format!("/realms/{}/login-actions/authenticate", realm))
            .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", CONSOLE_CLIENT_ID)
            .json(&json!({ "username": username, "password": password }))
            .await;

        let body = login.text();
        assert_eq!(
            login.status_code(),
            200,
            "the interactive login on {realm} must succeed: {body}"
        );

        let parsed: Value = serde_json::from_str(&body).expect("the login response must be JSON");
        let url = parsed["url"]
            .as_str()
            .unwrap_or_else(|| panic!("a completed login carries a redirect url: {parsed}"));

        let code = url
            .split("code=")
            .nth(1)
            .and_then(|tail| tail.split('&').next())
            .unwrap_or_else(|| panic!("the redirect must carry an authorization code: {url}"))
            .to_string();

        assert!(
            !code.is_empty(),
            "the authorization code handed back on {realm} must not be empty: {url}"
        );

        code
    }

    async fn exchange_code(
        server: &TestServer,
        realm: &str,
        code: &str,
        redirect_uri: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", CONSOLE_CLIENT_ID),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("code_verifier", S256_VERIFIER),
            ])
            .await
    }

    fn assert_carries_no_token(body: &Value, context: &str) {
        assert!(
            body.get("access_token").and_then(Value::as_str).is_none(),
            "{context}: the refusal still carried an access token: {body}"
        );
        assert!(
            body.get("refresh_token").and_then(Value::as_str).is_none(),
            "{context}: the refusal still carried a refresh token: {body}"
        );
        assert!(
            body.get("id_token").and_then(Value::as_str).is_none(),
            "{context}: the refusal still carried an id token: {body}"
        );
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn an_authorization_code_is_refused_at_another_realms_token_endpoint() {
        rt().block_on(async {
            let server = make_server();
            let code =
                authorization_code_of(&server, tenant_a(), alice_username(), ALICE_PASSWORD).await;
            let redirect_uri = console_callback(tenant_a());

            let refused = exchange_code(&server, tenant_b(), &code, &redirect_uri).await;
            let refused_status = refused.status_code().as_u16();
            let refused_body: Value = serde_json::from_str(&refused.text()).unwrap_or(Value::Null);

            assert_eq!(
                refused_status, 400,
                "a code minted in another realm must not be redeemable: {refused_body}"
            );
            assert_eq!(
                refused_body["error"],
                json!("invalid_grant"),
                "the refusal must speak the OAuth2 grant vocabulary: {refused_body}"
            );
            assert_carries_no_token(&refused_body, "cross-realm code exchange");

            let accepted = exchange_code(&server, tenant_a(), &code, &redirect_uri).await;
            let accepted_body = accepted.text();

            assert_eq!(
                accepted.status_code(),
                200,
                "the same code must still redeem in the realm that minted it: {accepted_body}"
            );

            let accepted_body: Value =
                serde_json::from_str(&accepted_body).expect("the token response must be JSON");
            let access_token = accepted_body["access_token"].as_str().unwrap_or_else(|| {
                panic!("the nominal exchange must mint a token: {accepted_body}")
            });
            assert!(
                !access_token.is_empty(),
                "the nominal exchange must mint a non-empty token: {accepted_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn a_refresh_token_is_refused_at_another_realms_token_endpoint() {
        rt().block_on(async {
            let server = make_server();
            let issued =
                direct_grant_body(&server, tenant_a(), alice_username(), ALICE_PASSWORD).await;
            let refresh_token = issued["refresh_token"]
                .as_str()
                .unwrap_or_else(|| panic!("the direct grant must mint a refresh token: {issued}"))
                .to_string();
            assert!(
                !refresh_token.is_empty(),
                "the refresh token must not be empty: {issued}"
            );

            let renewed = refresh(&server, tenant_a(), &refresh_token).await;
            let renewed_body = renewed.text();
            assert_eq!(
                renewed.status_code(),
                200,
                "the refresh token must renew in its own realm: {renewed_body}"
            );
            let renewed_body: Value =
                serde_json::from_str(&renewed_body).expect("the refresh response must be JSON");
            let rotated = renewed_body["refresh_token"]
                .as_str()
                .unwrap_or_else(|| panic!("rotation must hand back a successor: {renewed_body}"))
                .to_string();
            assert!(
                !rotated.is_empty(),
                "the rotated refresh token must not be empty: {renewed_body}"
            );

            let refused = refresh(&server, tenant_b(), &rotated).await;
            let refused_status = refused.status_code().as_u16();
            let refused_body: Value = serde_json::from_str(&refused.text()).unwrap_or(Value::Null);

            assert_ne!(
                refused_status, 200,
                "a refresh token of another realm must never renew: {refused_body}"
            );
            assert_carries_no_token(&refused_body, "cross-realm refresh");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn client_credentials_of_one_realm_are_refused_on_another() {
        rt().block_on(async {
            let server = make_server();
            let (client_id, client_secret) = create_service_client(&server, tenant_a()).await;

            let accepted =
                client_credentials(&server, tenant_a(), &client_id, &client_secret).await;
            let accepted_body = accepted.text();
            assert_eq!(
                accepted.status_code(),
                200,
                "the service client must obtain a token in its own realm: {accepted_body}"
            );
            let accepted_body: Value =
                serde_json::from_str(&accepted_body).expect("the token response must be JSON");
            let access_token = accepted_body["access_token"]
                .as_str()
                .unwrap_or_else(|| panic!("the nominal grant must mint a token: {accepted_body}"));
            assert!(
                !access_token.is_empty(),
                "the nominal grant must mint a non-empty token: {accepted_body}"
            );

            let refused = client_credentials(&server, tenant_b(), &client_id, &client_secret).await;
            let refused_status = refused.status_code().as_u16();
            let refused_body: Value = serde_json::from_str(&refused.text()).unwrap_or(Value::Null);

            assert_ne!(
                refused_status, 200,
                "a client of another realm must not be honoured: {refused_body}"
            );
            assert_carries_no_token(&refused_body, "cross-realm client credentials");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn a_client_credentials_grant_is_refused_when_its_service_account_sits_in_another_realm() {
        rt().block_on(async {
            let server = make_server();
            let (client_id, client_secret) = create_service_client(&server, tenant_b()).await;

            let accepted =
                client_credentials(&server, tenant_b(), &client_id, &client_secret).await;
            let accepted_body = accepted.text();
            assert_eq!(
                accepted.status_code(),
                200,
                "the service client must obtain a token before the account is moved: {accepted_body}"
            );
            let accepted_body: Value =
                serde_json::from_str(&accepted_body).expect("the token response must be JSON");
            assert!(
                accepted_body["access_token"]
                    .as_str()
                    .is_some_and(|token| !token.is_empty()),
                "the nominal grant must mint a non-empty token: {accepted_body}"
            );

            let service_account = service_account_of(&client_id).await;
            sqlx::query("UPDATE users SET realm_id = $1 WHERE id = $2")
                .bind(ctx().tenant_a_id)
                .bind(service_account)
                .execute(&ctx().pool)
                .await
                .expect("move the service account into the neighbouring realm");

            let refused =
                client_credentials(&server, tenant_b(), &client_id, &client_secret).await;
            let refused_status = refused.status_code().as_u16();
            let refused_body: Value =
                serde_json::from_str(&refused.text()).unwrap_or(Value::Null);

            assert_ne!(
                refused_status, 200,
                "a realm must not sign a token for a service account it does not own: {refused_body}"
            );
            assert_carries_no_token(&refused_body, "foreign service account");

            assert_eq!(
                realm_of_user(service_account).await,
                ctx().tenant_a_id,
                "the refusal must not have touched the account row"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn an_access_token_does_not_introspect_as_active_in_another_realm() {
        rt().block_on(async {
            let server = make_server();
            let alice_token =
                direct_grant_token(&server, tenant_a(), alice_username(), ALICE_PASSWORD).await;

            let (client_a, secret_a) = create_service_client(&server, tenant_a()).await;
            let (client_b, secret_b) = create_service_client(&server, tenant_b()).await;

            let own = introspect(&server, tenant_a(), &client_a, &secret_a, &alice_token).await;
            let own_body = own.text();
            assert_eq!(
                own.status_code(),
                200,
                "introspection in the token's own realm must answer: {own_body}"
            );
            let own_body: Value =
                serde_json::from_str(&own_body).expect("the introspection response must be JSON");
            assert_eq!(
                own_body["active"],
                json!(true),
                "a live token must introspect as active in its own realm: {own_body}"
            );
            assert_eq!(
                own_body["sub"].as_str(),
                Some(alice_id()),
                "the nominal introspection must name the subject: {own_body}"
            );

            let foreign = introspect(&server, tenant_b(), &client_b, &secret_b, &alice_token).await;
            let foreign_body = foreign.text();
            assert_eq!(
                foreign.status_code(),
                200,
                "RFC 7662 asks for an answer, not an error: {foreign_body}"
            );
            let foreign_body: Value = serde_json::from_str(&foreign_body)
                .expect("the introspection response must be JSON");

            assert_eq!(
                foreign_body["active"],
                json!(false),
                "a neighbouring realm must not vouch for a token it did not mint: {foreign_body}"
            );
            assert!(
                foreign_body["sub"]
                    .as_str()
                    .is_none_or(|sub| sub.is_empty()),
                "the refusal leaked the subject of another realm: {foreign_body}"
            );
            assert!(
                foreign_body["username"]
                    .as_str()
                    .is_none_or(|name| name.is_empty()),
                "the refusal leaked the username of another realm: {foreign_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn a_revocation_aimed_from_another_realm_leaves_the_token_alive() {
        rt().block_on(async {
            let server = make_server();
            let alice_token =
                direct_grant_token(&server, tenant_a(), alice_username(), ALICE_PASSWORD).await;

            let foreign = revoke(&server, tenant_b(), "admin-cli", &alice_token).await;
            assert_eq!(
                foreign.status_code(),
                200,
                "RFC 7009 revocation is idempotent and must not leak: {}",
                foreign.text()
            );

            let survived = userinfo(&server, tenant_a(), &alice_token).await;
            let survived_body = survived.text();
            assert_eq!(
                survived.status_code(),
                200,
                "a revocation aimed at another realm must not kill the token: {survived_body}"
            );
            let survived_body: Value =
                serde_json::from_str(&survived_body).expect("the userinfo response must be JSON");
            assert_eq!(
                survived_body["sub"].as_str(),
                Some(alice_id()),
                "the surviving token must still stand for its own subject: {survived_body}"
            );

            let own = revoke(&server, tenant_a(), "admin-cli", &alice_token).await;
            assert_eq!(
                own.status_code(),
                200,
                "revocation in the token's own realm must be accepted: {}",
                own.text()
            );

            let dead = userinfo(&server, tenant_a(), &alice_token).await;
            let dead_body = dead.text();
            assert_ne!(
                dead.status_code(),
                200,
                "a token revoked in its own realm must stop working: {dead_body}"
            );
            assert!(
                !dead_body.contains(alice_id()),
                "a revoked token must not still return its subject: {dead_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn userinfo_refuses_a_token_minted_by_another_realm() {
        rt().block_on(async {
            let server = make_server();
            let alice_token =
                direct_grant_token(&server, tenant_a(), alice_username(), ALICE_PASSWORD).await;

            let own = userinfo(&server, tenant_a(), &alice_token).await;
            let own_body = own.text();
            assert_eq!(
                own.status_code(),
                200,
                "userinfo must answer in the token's own realm: {own_body}"
            );
            let own_body: Value =
                serde_json::from_str(&own_body).expect("the userinfo response must be JSON");
            assert_eq!(
                own_body["sub"].as_str(),
                Some(alice_id()),
                "the nominal userinfo must name the subject: {own_body}"
            );
            assert_eq!(
                own_body["preferred_username"].as_str(),
                Some(alice_username()),
                "the nominal userinfo must carry the profile claims: {own_body}"
            );

            let foreign = userinfo(&server, tenant_b(), &alice_token).await;
            let foreign_body = foreign.text();

            assert_ne!(
                foreign.status_code(),
                200,
                "a neighbouring realm must not describe an account it does not own: {foreign_body}"
            );
            assert!(
                !foreign_body.contains(alice_id()),
                "the refusal leaked the subject of another realm: {foreign_body}"
            );
            assert!(
                !foreign_body.contains(alice_username()),
                "the refusal leaked the username of another realm: {foreign_body}"
            );
            assert_eq!(
                foreign.status_code().as_u16(),
                401,
                "a token minted elsewhere must read as invalid (RFC 6750): {foreign_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test oidc_cross_realm_test -- --ignored"]
    fn an_unknown_realm_never_answers_with_another_realms_account() {
        rt().block_on(async {
            let server = make_server();
            let bob_token =
                direct_grant_token(&server, tenant_b(), bob_username(), BOB_PASSWORD).await;

            let own = userinfo(&server, tenant_b(), &bob_token).await;
            let own_body = own.text();
            assert_eq!(
                own.status_code(),
                200,
                "userinfo must answer in the token's own realm: {own_body}"
            );
            assert!(
                own_body.contains(bob_username()),
                "the nominal userinfo must carry the profile claims: {own_body}"
            );

            let ghost = format!("ghost-{}", Uuid::new_v4().simple());
            let refused = userinfo(&server, &ghost, &bob_token).await;
            let refused_body = refused.text();

            assert_ne!(
                refused.status_code(),
                200,
                "an unknown realm must not answer for an account: {refused_body}"
            );
            assert!(
                !refused_body.contains(bob_username()),
                "the refusal leaked an account through an unknown realm: {refused_body}"
            );
        });
    }
}
