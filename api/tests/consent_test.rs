#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
    use axum_extra::extract::cookie::Cookie;
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

        let schema = format!("test_consent_{}", Uuid::new_v4().simple());
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

        let db = DatabaseConfig {
            host: db_host,
            port: db_port,
            username: db_user,
            password: db_password,
            name: db_name,
            schema: schema.clone(),
        };

        let svc = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: false,
            webapp_url: "http://localhost:5555".to_string(),
            database: db.clone(),
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

    fn query_param(url: &str, key: &str) -> Option<String> {
        let (_, query) = url.split_once('?')?;
        query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == key).then(|| urlencoding::decode(v).ok().map(|s| s.to_string()))?
        })
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

    struct TestClient {
        id: String,
        client_id: String,
    }

    async fn create_client(
        server: &TestServer,
        realm: &str,
        token: &str,
        prefix: &str,
    ) -> TestClient {
        let client_id = format!("{prefix}-{}", Uuid::new_v4().simple());

        let response = server
            .post(&format!("/realms/{realm}/clients"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "client_id": client_id,
                "name": prefix,
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": false,
                "direct_access_grants_enabled": true,
                "enabled": true,
                "oauth_device_code_grant_enabled": false,
                "token_exchange_enabled": false,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let body: Value = response.json();
        let id = body["id"].as_str().expect("client id").to_string();

        let redirect = server
            .post(&format!("/realms/{realm}/clients/{id}/redirects"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "value": format!("http://localhost:5555/realms/{realm}/authentication/callback"),
                "enabled": true,
            }))
            .await;
        assert_eq!(redirect.status_code(), 201, "{}", redirect.text());

        TestClient { id, client_id }
    }

    async fn create_optional_scope(
        server: &TestServer,
        realm: &str,
        token: &str,
        client: &TestClient,
        name: &str,
    ) {
        let response = server
            .post(&format!("/realms/{realm}/client-scopes"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "name": name,
                "description": format!("{name} description"),
                "protocol": "openid-connect",
                "is_default": false,
            }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let body: Value = response.json();
        let scope_id = body["id"].as_str().expect("scope id");

        let assign = server
            .put(&format!(
                "/realms/{realm}/clients/{}/optional-client-scopes/{scope_id}",
                client.id
            ))
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(assign.status_code(), 200, "{}", assign.text());
    }

    async fn set_consent_required(
        server: &TestServer,
        realm: &str,
        token: &str,
        client: &TestClient,
    ) {
        let response = server
            .patch(&format!("/realms/{realm}/clients/{}", client.id))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "consent_required": true }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());

        let body: Value = response.json();
        assert_eq!(
            body["data"]["consent_required"], true,
            "the update response did not echo consent_required back: {body}"
        );
    }

    async fn authorize_and_authenticate(
        server: &TestServer,
        realm: &str,
        client: &TestClient,
        scope: &str,
    ) -> (Value, Cookie<'static>) {
        let authorize = server
            .get(&format!("/realms/{realm}/protocol/openid-connect/auth"))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", &client.client_id)
            .add_query_param(
                "redirect_uri",
                format!("http://localhost:5555/realms/{realm}/authentication/callback"),
            )
            .add_query_param("scope", scope)
            .add_query_param("state", "st")
            .await;

        let status = authorize.status_code().as_u16();
        let location = authorize
            .headers()
            .get("location")
            .map(|value| value.to_str().unwrap_or("<non-utf8>").to_string())
            .unwrap_or_default();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth, got {status}: {}",
            authorize.text()
        );
        assert!(
            authorize.maybe_cookie("FERRISKEY_SESSION").is_some(),
            "/auth redirected to {location} without opening an auth session"
        );

        let login = server
            .post(&format!("/realms/{realm}/login-actions/authenticate"))
            .add_cookie(authorize.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", &client.client_id)
            .json(&json!({ "username": "admin", "password": "admin_pass_1234!" }))
            .await;

        assert_eq!(login.status_code(), 200, "{}", login.text());
        (login.json(), authorize.cookie("FERRISKEY_SESSION"))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_client_with_consent_required_false_never_reaches_the_consent_screen() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "noconsent").await;

            let (result, _session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid").await;

            let url = result["url"].as_str().unwrap_or_default();
            assert!(
                !url.contains("authentication/consent"),
                "a client with consent_required=false was sent to the consent screen: {result}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_client_requiring_consent_redirects_to_the_consent_screen_and_the_token_round_trips() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "consent").await;
            create_optional_scope(&srv, &realm, &token, &client, "contacts").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid contacts").await;
            let url = result["url"].as_str().expect("redirect url");
            assert!(
                url.contains("authentication/consent"),
                "a client requiring consent was not sent to the consent screen: {result}"
            );

            let consent_token = query_param(url, "consent_token").expect("consent_token in url");

            let view = srv
                .get(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .add_query_param("consent_token", &consent_token)
                .await;
            assert_eq!(view.status_code(), 200, "{}", view.text());
            let view: Value = view.json();
            assert!(
                view["awaiting_decision"]
                    .as_array()
                    .expect("awaiting_decision array")
                    .iter()
                    .any(|s| s["name"] == "contacts"),
                "contacts was not offered as an awaiting-decision scope: {view}"
            );

            let decision = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": ["contacts"],
                }))
                .await;
            assert_eq!(decision.status_code(), 200, "{}", decision.text());
            let decision: Value = decision.json();
            let redirect_url = decision["redirect_url"].as_str().expect("redirect_url");
            assert!(
                redirect_url.contains("code="),
                "approving consent did not issue an authorization code: {decision}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn denying_every_optional_scope_redirects_with_access_denied() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "denyall").await;
            create_optional_scope(&srv, &realm, &token, &client, "calendar").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid calendar").await;
            let url = result["url"].as_str().expect("redirect url");
            let consent_token = query_param(url, "consent_token").expect("consent_token in url");

            let decision = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": [],
                }))
                .await;
            assert_eq!(decision.status_code(), 200, "{}", decision.text());
            let decision: Value = decision.json();
            let redirect_url = decision["redirect_url"].as_str().expect("redirect_url");
            assert!(
                redirect_url.contains("error=access_denied"),
                "denying every optional scope did not yield access_denied: {decision}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn an_unknown_consent_token_is_rejected_without_revealing_why() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let response = srv
                .get(&format!("/realms/{realm}/auth/consent"))
                .add_query_param("consent_token", "this-token-never-existed")
                .await;

            assert_eq!(response.status_code(), 404, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_client_requiring_consent_with_no_optional_scopes_requested_skips_the_screen() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "noscopes").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, _session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid").await;

            let url = result["url"].as_str().unwrap_or_default();
            assert!(
                !url.contains("authentication/consent"),
                "a client with nothing optional requested was sent to the consent screen: {result}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_consent_token_lives_in_the_database_and_not_in_process_memory() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "crosspod").await;
            create_optional_scope(&srv, &realm, &token, &client, "notes").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid notes").await;
            let url = result["url"].as_str().expect("redirect url");
            let consent_token = query_param(url, "consent_token").expect("consent_token in url");

            let session_id =
                Uuid::parse_str(session.value()).expect("the session cookie is a uuid");

            let stored: Option<String> =
                sqlx::query_scalar("SELECT consent_token_hash FROM auth_sessions WHERE id = $1")
                    .bind(session_id)
                    .fetch_one(&ctx().pool)
                    .await
                    .expect("the pending consent must be readable from the database alone");

            let stored = stored.expect("a pending consent must carry its token hash");
            assert_ne!(
                stored, consent_token,
                "the raw consent token must never be stored"
            );
            assert_eq!(
                stored.len(),
                64,
                "the stored value must be a sha-256 hex digest, got {stored}"
            );

            let view = srv
                .get(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .add_query_param("consent_token", &consent_token)
                .await;
            assert_eq!(view.status_code(), 200, "{}", view.text());

            let decision = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": ["notes"],
                }))
                .await;
            assert_eq!(decision.status_code(), 200, "{}", decision.text());
            let decision: Value = decision.json();
            let redirect_url = decision["redirect_url"].as_str().expect("redirect_url");
            assert!(
                redirect_url.contains("code="),
                "the decision did not complete the login: {decision}"
            );

            let remaining: Option<String> =
                sqlx::query_scalar("SELECT consent_token_hash FROM auth_sessions WHERE id = $1")
                    .bind(session_id)
                    .fetch_one(&ctx().pool)
                    .await
                    .expect("read the consent token hash back");
            assert_eq!(remaining, None, "a consent token must be single use");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_consent_token_is_useless_without_the_browser_session() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "leaked").await;
            create_optional_scope(&srv, &realm, &token, &client, "leaked-notes").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, _session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid leaked-notes").await;
            let url = result["url"].as_str().expect("redirect url");
            let consent_token = query_param(url, "consent_token").expect("consent_token in url");

            let view = srv
                .get(&format!("/realms/{realm}/auth/consent"))
                .add_query_param("consent_token", &consent_token)
                .await;
            assert_eq!(
                view.status_code(),
                404,
                "a token lifted from a log or a referer must not open the screen: {}",
                view.text()
            );

            let decision = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": ["leaked-notes"],
                }))
                .await;
            assert_eq!(
                decision.status_code(),
                404,
                "a token alone must not be able to approve scopes: {}",
                decision.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test consent_test -- --ignored"]
    fn a_spent_consent_token_cannot_be_replayed() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let token = login(&srv, &realm).await;
            let client = create_client(&srv, &realm, &token, "replay").await;
            create_optional_scope(&srv, &realm, &token, &client, "replay-notes").await;
            set_consent_required(&srv, &realm, &token, &client).await;

            let (result, session) =
                authorize_and_authenticate(&srv, &realm, &client, "openid replay-notes").await;
            let url = result["url"].as_str().expect("redirect url");
            let consent_token = query_param(url, "consent_token").expect("consent_token in url");

            let first = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": ["replay-notes"],
                }))
                .await;
            assert_eq!(first.status_code(), 200, "{}", first.text());

            let replay = srv
                .post(&format!("/realms/{realm}/auth/consent"))
                .add_cookie(session.clone())
                .json(&json!({
                    "consent_token": consent_token,
                    "approved_scopes": ["replay-notes"],
                }))
                .await;
            assert_eq!(
                replay.status_code(),
                404,
                "a spent consent token must be refused on replay: {}",
                replay.text()
            );
        });
    }
}
