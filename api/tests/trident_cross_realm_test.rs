#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
    use base64::prelude::{BASE64_URL_SAFE_NO_PAD, Engine as _};
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::Args,
    };
    use ferriskey_core::{
        application::create_service,
        domain::{
            common::{
                DatabaseConfig, FerriskeyConfig, entities::StartupConfig, ports::CoreService,
            },
            credential::entities::CredentialData,
        },
    };
    use serde_json::{Value, json};
    use sqlx::{Executor, Row};
    use uuid::Uuid;
    use webauthn_rs::prelude::{
        AttestationFormat, COSEAlgorithm, COSEEC2Key, COSEKey, COSEKeyType,
        Credential as WebAuthnCredential, ECDSACurve, ParsedAttestation,
    };

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const RECOVERY_CODE_FORMAT: &str = "b32-split-4";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const FIRST_PASSWORD: &str = "Str0ng!P@ssword#2024";
    const SECOND_PASSWORD: &str = "An0ther!P@ssword#2025";
    const THIRD_PASSWORD: &str = "Refused!P@ssword#2026";
    const ADMIN_PASSKEY_ID: &[u8] = b"admin-passkey-credential-id";
    const ADMIN_PASSKEY_ROW: Uuid = Uuid::from_u128(0x7bd1_4a2c_9f30_4e51_8c62_0d9a_4b7e_1f33);

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: sqlx::PgPool,
        realm_name: String,
        realm_id: Uuid,
        admin_user_id: Uuid,
        neighbour_realm_name: String,
        neighbour_realm_id: Uuid,
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

        let schema = format!("trident_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let realm_id: Uuid = sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(&realm_name)
            .fetch_one(&pool)
            .await
            .expect("fetch master realm")
            .get("id");

        let admin_user_id: Uuid =
            sqlx::query("SELECT id FROM users WHERE realm_id = $1 AND username = $2")
                .bind(realm_id)
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("fetch admin user")
                .get("id");

        let neighbour_realm_id = Uuid::new_v4();
        let neighbour_realm_name = format!("neighbour-{}", Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(neighbour_realm_id)
        .bind(&neighbour_realm_name)
        .execute(&pool)
        .await
        .expect("insert neighbour realm");

        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(Uuid::new_v4())
        .bind(neighbour_realm_id)
        .bind("victim")
        .bind("Victim")
        .bind("User")
        .bind("victim@neighbour.local")
        .execute(&pool)
        .await
        .expect("insert neighbour user");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            realm_name,
            realm_id,
            admin_user_id,
            neighbour_realm_name,
            neighbour_realm_id,
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

    fn realm() -> &'static str {
        ctx().realm_name.as_str()
    }

    fn neighbour_realm() -> &'static str {
        ctx().neighbour_realm_name.as_str()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    async fn get_token(server: &TestServer, username: &str, password: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await
    }

    async fn get_admin_token(server: &TestServer) -> String {
        let response = get_token(server, "admin", "admin").await;

        assert_eq!(response.status_code(), 200, "admin token request failed");
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    async fn start_authorization(server: &TestServer) -> TestResponse {
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

    async fn authenticate(
        server: &TestServer,
        session_code: &str,
        username: &str,
        password: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/login-actions/authenticate", realm()))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_code}")).unwrap(),
            )
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .json(&json!({ "username": username, "password": password }))
            .await
    }

    fn passkey_credential_data() -> Value {
        let credential = WebAuthnCredential {
            cred_id: ADMIN_PASSKEY_ID.to_vec().into(),
            cred: COSEKey {
                type_: COSEAlgorithm::ES256,
                key: COSEKeyType::EC_EC2(COSEEC2Key {
                    curve: ECDSACurve::SECP256R1,
                    x: [1u8; 32].to_vec().into(),
                    y: [2u8; 32].to_vec().into(),
                }),
            },
            counter: 0,
            transports: None,
            user_verified: true,
            backup_eligible: false,
            backup_state: false,
            registration_policy: Default::default(),
            extensions: Default::default(),
            attestation: ParsedAttestation::default(),
            attestation_format: AttestationFormat::None,
        };

        serde_json::to_value(CredentialData::WebAuthn {
            credential: Box::new(credential),
        })
        .expect("the passkey fixture must serialize")
    }

    async fn grant_passkey_to_admin() {
        sqlx::query(
            "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, webauthn_credential_id)
             VALUES ($1, 'webauthn-public-key-credential', $2, '', $3::jsonb, $4)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(ADMIN_PASSKEY_ROW)
        .bind(ctx().admin_user_id)
        .bind(passkey_credential_data())
        .bind(ADMIN_PASSKEY_ID)
        .execute(&ctx().pool)
        .await
        .expect("insert admin passkey");
    }

    async fn count_admin_recovery_codes() -> i64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM credentials WHERE user_id = $1 AND credential_type = 'recovery-code'",
        )
        .bind(ctx().admin_user_id)
        .fetch_one(&ctx().pool)
        .await
        .expect("count admin recovery codes")
    }

    async fn webauthn_request_options(
        server: &TestServer,
        url_realm: &str,
        token: &str,
        session_code: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/webauthn-public-key-request-options",
                url_realm
            ))
            .add_header("Authorization", auth_header(token))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_code}")).unwrap(),
            )
            .await
    }

    async fn generate_recovery_codes(server: &TestServer, token: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/generate-recovery-codes",
                realm()
            ))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "amount": 2, "code_format": RECOVERY_CODE_FORMAT }))
            .await
    }

    async fn burn_recovery_code(
        server: &TestServer,
        url_realm: &str,
        token: &str,
        session_code: &str,
        code: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/burn-recovery-code",
                url_realm
            ))
            .add_header("Authorization", auth_header(token))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_code}")).unwrap(),
            )
            .json(&json!({
                "recovery_code": code,
                "recovery_code_format": RECOVERY_CODE_FORMAT,
            }))
            .await
    }

    async fn update_password(
        server: &TestServer,
        url_realm: &str,
        step_token: &str,
        value: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/update-password",
                url_realm
            ))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_LOGIN_ACTION={step_token}")).unwrap(),
            )
            .json(&json!({ "value": value }))
            .await
    }

    async fn seed_user_owing_a_password_update(server: &TestServer, label: &str) -> String {
        let token = get_admin_token(server).await;
        let username = format!("{label}-{}", Uuid::new_v4().simple());

        let created = server
            .post(&format!("/realms/{}/users", realm()))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "username": username,
                "firstname": "Pass",
                "lastname": "Target",
                "email": format!("{username}@test.local"),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            created.status_code(),
            200,
            "creating the target user failed: {}",
            created.text()
        );

        let user_id: Uuid =
            sqlx::query("SELECT id FROM users WHERE realm_id = $1 AND username = $2")
                .bind(ctx().realm_id)
                .bind(&username)
                .fetch_one(&ctx().pool)
                .await
                .expect("fetch the created user")
                .get("id");

        let reset = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm(),
                user_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "value": FIRST_PASSWORD,
                "temporary": true,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            reset.status_code(),
            200,
            "seeding the temporary password failed: {}",
            reset.text()
        );

        sqlx::query(
            "INSERT INTO user_required_actions (id, user_id, action, created_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("update_password")
        .execute(&ctx().pool)
        .await
        .expect("insert the update_password required action");

        username
    }

    async fn step_token_for(server: &TestServer, username: &str, password: &str) -> String {
        let authorize = start_authorization(server).await;
        let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();
        let login = authenticate(server, &session_code, username, password).await;

        assert_eq!(
            login.status_code(),
            200,
            "the password step must succeed for {username}: {}",
            login.text()
        );

        login
            .maybe_cookie("FERRISKEY_LOGIN_ACTION")
            .map(|cookie| cookie.value().to_string())
            .unwrap_or_else(|| {
                panic!(
                    "a user owing a password update must receive a step token: {}",
                    login.text()
                )
            })
    }

    async fn password_is_accepted(server: &TestServer, username: &str, password: &str) -> bool {
        let authorize = start_authorization(server).await;
        let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();

        authenticate(server, &session_code, username, password)
            .await
            .status_code()
            == 200
    }

    fn unsigned_assertion() -> Value {
        let credential_id = BASE64_URL_SAFE_NO_PAD.encode(ADMIN_PASSKEY_ID);

        json!({
            "id": credential_id,
            "rawId": credential_id,
            "response": {
                "authenticatorData": BASE64_URL_SAFE_NO_PAD.encode([0u8; 37]),
                "clientDataJSON": BASE64_URL_SAFE_NO_PAD.encode(
                    br#"{"type":"webauthn.get","challenge":"","origin":"http://localhost:5555"}"#,
                ),
                "signature": BASE64_URL_SAFE_NO_PAD.encode([0u8; 64]),
                "userHandle": BASE64_URL_SAFE_NO_PAD.encode(ctx().admin_user_id.as_bytes()),
            },
            "type": "public-key",
        })
    }

    async fn stored_challenge(session_id: &str) -> Option<String> {
        let session_id = Uuid::parse_str(session_id).expect("the session cookie must be a uuid");

        sqlx::query("SELECT webauthn_challenge::text AS challenge FROM auth_sessions WHERE id = $1")
            .bind(session_id)
            .fetch_one(&ctx().pool)
            .await
            .expect("fetch the auth session challenge")
            .get("challenge")
    }

    async fn planted_challenge(server: &TestServer, token: &str) -> (String, String) {
        let authorize = start_authorization(server).await;
        let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();

        let options = webauthn_request_options(server, realm(), token, &session_code).await;
        let options_body = options.text();
        assert_eq!(
            options.status_code(),
            200,
            "planting a challenge on the caller's own session must succeed: {} {options_body}",
            options.status_code()
        );
        assert!(
            options_body.contains("allowCredentials"),
            "the planted challenge must list the enrolled passkey: {options_body}"
        );

        let challenge = stored_challenge(&session_code)
            .await
            .expect("the session must carry the challenge the server just issued");
        assert!(
            challenge.contains("Authentication"),
            "the stored challenge must be an authentication challenge: {challenge}"
        );

        (session_code, challenge)
    }

    async fn seed_neighbour_session(challenge: &str) -> Uuid {
        let client_id = Uuid::new_v4();
        let client_name = format!("neighbour-client-{}", Uuid::new_v4().simple());

        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, protocol, public_client, client_type) VALUES ($1, $2, $3, $4, $5, TRUE, $6)",
        )
        .bind(client_id)
        .bind(ctx().neighbour_realm_id)
        .bind(&client_name)
        .bind(&client_name)
        .bind("openid-connect")
        .bind("public")
        .execute(&ctx().pool)
        .await
        .expect("insert neighbour client");

        let session_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO auth_sessions (id, realm_id, client_id, redirect_uri, response_type, scope, state, authenticated, expires_at, protocol, webauthn_challenge, webauthn_challenge_issued_at)
             VALUES ($1, $2, $3, $4, 'code', 'openid', 'st', FALSE, NOW() + INTERVAL '1 hour', 'openid-connect', $5::jsonb, NOW())",
        )
        .bind(session_id)
        .bind(ctx().neighbour_realm_id)
        .bind(client_id)
        .bind(format!(
            "{WEBAPP_URL}/realms/{}/authentication/callback",
            neighbour_realm()
        ))
        .bind(challenge)
        .execute(&ctx().pool)
        .await
        .expect("insert neighbour auth session");

        session_id
    }

    async fn session_is_unspent(session_id: Uuid) -> bool {
        let row = sqlx::query(
            "SELECT code IS NULL AS no_code, user_id IS NULL AS no_user FROM auth_sessions WHERE id = $1",
        )
        .bind(session_id)
        .fetch_optional(&ctx().pool)
        .await
        .expect("query the auth session")
        .expect("the auth session row must still exist");

        row.get::<bool, _>("no_code") && row.get::<bool, _>("no_user")
    }

    async fn webauthn_authenticate(
        server: &TestServer,
        url_realm: &str,
        token: &str,
        session_code: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/webauthn-public-key-authenticate",
                url_realm
            ))
            .add_header("Authorization", auth_header(token))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_code}")).unwrap(),
            )
            .json(&unsigned_assertion())
            .await
    }

    async fn passkey_authenticate(
        server: &TestServer,
        url_realm: &str,
        session_code: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/login-actions/passkey-authenticate",
                url_realm
            ))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_code}")).unwrap(),
            )
            .json(&unsigned_assertion())
            .await
    }

    fn assert_refusal(response: &TestResponse, status: u16, reason: &str, what: &str) {
        let body = response.text();
        assert_eq!(
            response.status_code().as_u16(),
            status,
            "{what}: expected {status}, got {} with body {body}",
            response.status_code()
        );

        let parsed: Value = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("{what}: body is not json: {body}"));
        assert_eq!(
            parsed["reason"],
            json!(reason),
            "{what}: expected the reason {reason:?}, got body {body}"
        );
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

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test trident_cross_realm_test -- --ignored"]
    fn requesting_webauthn_options_through_another_realms_url_is_refused() {
        rt().block_on(async {
            grant_passkey_to_admin().await;

            let server = make_server();
            let token = get_admin_token(&server).await;
            let authorize = start_authorization(&server).await;
            let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();

            let accepted = webauthn_request_options(&server, realm(), &token, &session_code).await;
            let accepted_body = accepted.text();
            assert_eq!(
                accepted.status_code(),
                200,
                "a caller must get a challenge through its own realm's url, otherwise the \
                 refusal below proves nothing: {} {accepted_body}",
                accepted.status_code()
            );
            assert!(
                accepted_body.contains("challenge"),
                "the accepted call returned no challenge, so an empty body would satisfy the \
                 refusal below for the wrong reason: {accepted_body}"
            );
            assert!(
                accepted_body.contains("allowCredentials"),
                "the challenge must list the enrolled passkey: {accepted_body}"
            );

            let refused =
                webauthn_request_options(&server, neighbour_realm(), &token, &session_code).await;
            let refused_body = refused.text();

            assert_not_found(
                &refused,
                "requesting webauthn options through a url realm the caller does not belong to",
            );
            assert!(
                !refused_body.contains("allowCredentials"),
                "the refused call still handed back a challenge: {refused_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test trident_cross_realm_test -- --ignored"]
    fn burning_a_recovery_code_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let generated = generate_recovery_codes(&server, &token).await;
            let generated_body = generated.text();
            assert_eq!(
                generated.status_code(),
                200,
                "the recovery codes must be generated first: {} {generated_body}",
                generated.status_code()
            );

            let generated_json: Value =
                serde_json::from_str(&generated_body).expect("the generated codes must be json");
            let codes: Vec<String> = generated_json["codes"]
                .as_array()
                .expect("codes in response")
                .iter()
                .map(|code| {
                    code.as_str()
                        .expect("every code must be a string")
                        .to_string()
                })
                .collect();
            assert_eq!(codes.len(), 2, "expected two codes: {generated_body}");
            assert_eq!(
                count_admin_recovery_codes().await,
                2,
                "the generated codes must be stored before anything is burnt"
            );

            let authorize = start_authorization(&server).await;
            let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();

            let accepted =
                burn_recovery_code(&server, realm(), &token, &session_code, &codes[0]).await;
            let accepted_body = accepted.text();
            assert_eq!(
                accepted.status_code(),
                200,
                "a code must be burnable through its own realm's url, otherwise the refusal \
                 below proves nothing: {} {accepted_body}",
                accepted.status_code()
            );
            assert!(
                accepted_body.contains("login_url"),
                "the accepted burn returned no login url: {accepted_body}"
            );
            assert_eq!(
                count_admin_recovery_codes().await,
                1,
                "the accepted burn did not reach the database, so the refusal below is not \
                 evidence of scoping"
            );

            let second_authorize = start_authorization(&server).await;
            let second_session = second_authorize
                .cookie("FERRISKEY_SESSION")
                .value()
                .to_string();

            let refused = burn_recovery_code(
                &server,
                neighbour_realm(),
                &token,
                &second_session,
                &codes[1],
            )
            .await;

            assert_not_found(
                &refused,
                "burning a recovery code through a url realm the caller does not belong to",
            );
            assert!(
                !refused.text().contains("login_url"),
                "the refused burn still handed back a login url: {}",
                refused.text()
            );
            assert_eq!(
                count_admin_recovery_codes().await,
                1,
                "the refused burn destroyed the remaining recovery code anyway"
            );

            let third_authorize = start_authorization(&server).await;
            let third_session = third_authorize
                .cookie("FERRISKEY_SESSION")
                .value()
                .to_string();

            let still_valid =
                burn_recovery_code(&server, realm(), &token, &third_session, &codes[1]).await;
            assert_eq!(
                still_valid.status_code(),
                200,
                "the code the refused call named must still be spendable through the right \
                 realm, otherwise the refusal consumed it: {} {}",
                still_valid.status_code(),
                still_valid.text()
            );
            assert_eq!(
                count_admin_recovery_codes().await,
                0,
                "the second burn did not reach the database"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test trident_cross_realm_test -- --ignored"]
    fn authenticating_webauthn_against_another_realms_session_is_refused() {
        rt().block_on(async {
            grant_passkey_to_admin().await;

            let server = make_server();
            let token = get_admin_token(&server).await;
            let (own_session, challenge) = planted_challenge(&server, &token).await;
            let foreign_session = seed_neighbour_session(&challenge).await;

            let witness = webauthn_authenticate(&server, realm(), &token, &own_session).await;
            assert_refusal(
                &witness,
                401,
                "webauthn_challenge_failed",
                "the same request against the caller's own realm session must reach the challenge \
                 stage, otherwise the refusal below proves nothing about the session realm",
            );

            let refused =
                webauthn_authenticate(&server, realm(), &token, &foreign_session.to_string()).await;

            assert_refusal(
                &refused,
                404,
                "session_not_found",
                "completing a webauthn authentication against an authentication session of \
                 another realm",
            );
            assert!(
                !refused.text().contains("login_url"),
                "the refused call handed back a login url: {}",
                refused.text()
            );
            assert!(
                session_is_unspent(foreign_session).await,
                "the refused call stamped an authorization code or a user onto the neighbour \
                 realm's authentication session"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test trident_cross_realm_test -- --ignored"]
    fn authenticating_a_passkey_against_another_realms_session_is_refused() {
        rt().block_on(async {
            grant_passkey_to_admin().await;

            let server = make_server();
            let token = get_admin_token(&server).await;
            let (own_session, challenge) = planted_challenge(&server, &token).await;
            let foreign_session = seed_neighbour_session(&challenge).await;

            let witness = passkey_authenticate(&server, realm(), &own_session).await;
            assert_refusal(
                &witness,
                401,
                "webauthn_challenge_failed",
                "the same tokenless request against a session of the url realm must reach the \
                 challenge stage, otherwise the refusal below proves nothing about the session \
                 realm",
            );

            let refused =
                passkey_authenticate(&server, realm(), &foreign_session.to_string()).await;

            assert_refusal(
                &refused,
                404,
                "session_not_found",
                "completing a passkey authentication against an authentication session of \
                 another realm, with no token at all",
            );
            assert!(
                !refused.text().contains("login_url"),
                "the refused call handed back a login url: {}",
                refused.text()
            );
            assert!(
                session_is_unspent(foreign_session).await,
                "the refused call stamped an authorization code or a user onto the neighbour \
                 realm's authentication session"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test trident_cross_realm_test -- --ignored"]
    fn updating_a_password_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let accepted_user = seed_user_owing_a_password_update(&server, "update-accepted").await;
            let accepted_token = step_token_for(&server, &accepted_user, FIRST_PASSWORD).await;
            let accepted =
                update_password(&server, realm(), &accepted_token, SECOND_PASSWORD).await;
            assert_eq!(
                accepted.status_code(),
                200,
                "a password update through its own realm's url must be accepted, otherwise the \
                 refusal below proves nothing: {} {}",
                accepted.status_code(),
                accepted.text()
            );
            assert!(
                password_is_accepted(&server, &accepted_user, SECOND_PASSWORD).await,
                "the accepted update did not reach the database, so the refusal below is not \
                 evidence of scoping"
            );
            assert!(
                !password_is_accepted(&server, &accepted_user, FIRST_PASSWORD).await,
                "the accepted update left the previous password usable"
            );

            let refused_user = seed_user_owing_a_password_update(&server, "update-refused").await;
            let refused_token = step_token_for(&server, &refused_user, FIRST_PASSWORD).await;
            let refused =
                update_password(&server, neighbour_realm(), &refused_token, THIRD_PASSWORD).await;

            assert_ne!(
                refused.status_code(),
                200,
                "updating a password through a url realm the caller does not belong to was \
                 accepted: {}",
                refused.text()
            );
            assert!(
                !password_is_accepted(&server, &refused_user, THIRD_PASSWORD).await,
                "the refused update installed the new password anyway"
            );
            assert!(
                password_is_accepted(&server, &refused_user, FIRST_PASSWORD).await,
                "the refused update destroyed the existing password"
            );
        });
    }
}
