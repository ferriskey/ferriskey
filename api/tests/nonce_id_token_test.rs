/// Integration test: OIDC nonce claim round-trip through the Authorization Code Flow.
///
/// Verifies that when a nonce is supplied on the authorization request, it is
/// present as the `nonce` claim in the issued `id_token` (OIDC Core 1.0 §3.1.3.7).
///
/// Requires a running PostgreSQL instance. The test is marked `#[ignore]` so it
/// does not run in regular `cargo test` (no local Postgres). Run explicitly with:
///
///   cargo test -p ferriskey-api --test nonce_id_token_test -- --ignored
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

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    struct TestContext {
        server: TestServer,
        realm_name: String,
    }

    async fn setup() -> TestContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("nonce_id_token_test_{}", Uuid::new_v4().simple());

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
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema: schema.clone(),
            },
            encryption: ferriskey_core::domain::common::EncryptionConfig::default(),
        })
        .await
        .expect("create service");

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());

        // Use a non-"admin-cli" default_client_id so the dedicated admin-cli
        // seeding (public/System, direct_access_grants_enabled=true) is not
        // short-circuited by the generic confidential default-client creation,
        // which would otherwise leave admin-cli unable to do the password grant
        // (see #1086).
        service
            .initialize_application(StartupConfig {
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::new(app).expect("create test server");

        TestContext { server, realm_name }
    }

    async fn get_admin_token(ctx: &TestContext) -> String {
        let response = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_eq!(response.status_code(), 200, "admin token request failed");
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    #[tokio::test]
    #[ignore]
    async fn nonce_is_present_in_id_token_after_authorization_code_flow() {
        let ctx = setup().await;
        let admin_token = get_admin_token(&ctx).await;

        // 1. Create a public client for the OIDC flow
        let client_id = format!("nonce-test-client-{}", Uuid::new_v4().simple());

        let create_resp = ctx
            .server
            .post(&format!("/realms/{}/clients", ctx.realm_name))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({
                "client_id": client_id,
                "name": "Nonce Test Client",
                "client_type": "public",
                "protocol": "openid-connect",
                "public_client": true,
                "service_account_enabled": false,
                "direct_access_grants_enabled": false,
                "enabled": true,
                "oauth_device_code_grant_enabled": false
            }))
            .await;

        assert_eq!(
            create_resp.status_code(),
            201,
            "client creation failed: {}",
            create_resp.text()
        );
        let client_body: Value = create_resp.json();
        let client_uuid = client_body["id"]
            .as_str()
            .expect("client id in response")
            .to_string();

        // 2. Register a redirect URI for the client
        let redirect_uri = "https://app.example.com/callback";

        let redirect_resp = ctx
            .server
            .post(&format!(
                "/realms/{}/clients/{}/redirects",
                ctx.realm_name, client_uuid
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({
                "value": redirect_uri,
                "enabled": true
            }))
            .await;

        assert_eq!(
            redirect_resp.status_code(),
            201,
            "redirect URI registration failed: {}",
            redirect_resp.text()
        );

        // 3. Start the authorization flow — sets the FERRISKEY_SESSION cookie
        let nonce = "test-nonce-abc123";

        let auth_resp = ctx
            .server
            .get(&format!(
                "/realms/{}/protocol/openid-connect/auth",
                ctx.realm_name
            ))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", client_id.as_str())
            .add_query_param("redirect_uri", redirect_uri)
            .add_query_param("scope", "openid")
            .add_query_param("nonce", nonce)
            .await;

        // The authorize endpoint always redirects (302) and sets the
        // FERRISKEY_SESSION cookie on that response.
        let auth_status = auth_resp.status_code().as_u16();
        assert!(
            (300..=399).contains(&auth_status),
            "expected a 3xx redirect from /auth, got {}",
            auth_status
        );

        // Forward the session cookie explicitly. The cookie is `SameSite=Lax`, so
        // the automatic cookie jar withholds it on the cross-site POST below;
        // attaching it by hand (as device_flow_test does) bypasses that.
        let session_cookie = auth_resp.cookie("FERRISKEY_SESSION");

        // 4. Authenticate: POST /login-actions/authenticate?client_id=...
        let login_resp = ctx
            .server
            .post(&format!(
                "/realms/{}/login-actions/authenticate",
                ctx.realm_name
            ))
            .add_cookie(session_cookie)
            .add_query_param("client_id", client_id.as_str())
            .json(&json!({
                "username": "admin",
                "password": "admin"
            }))
            .await;

        assert_eq!(
            login_resp.status_code(),
            200,
            "authenticate failed: {}",
            login_resp.text()
        );

        let login_body: Value = login_resp.json();
        let callback_url = login_body["url"]
            .as_str()
            .expect("url field in authenticate response")
            .to_string();

        // Extract the `code` query param from the callback URL
        let parsed_url = url::Url::parse(&callback_url).expect("callback url should be valid");
        let code = parsed_url
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.to_string())
            .expect("code param should be present in callback URL");

        // 5. Exchange the authorization code for tokens
        let token_resp = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
            ))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("client_id", client_id.as_str()),
                ("redirect_uri", redirect_uri),
            ])
            .await;

        assert_eq!(
            token_resp.status_code(),
            200,
            "token exchange failed: {}",
            token_resp.text()
        );

        let token_body: Value = token_resp.json();
        let id_token_str = token_body["id_token"]
            .as_str()
            .expect("id_token should be present in token response");

        // 6. Decode the id_token payload and assert the nonce claim
        let parts: Vec<&str> = id_token_str.split('.').collect();
        assert_eq!(parts.len(), 3, "id_token should have 3 JWT segments");

        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .expect("id_token payload should be valid base64url");

        let payload: Value =
            serde_json::from_slice(&payload_bytes).expect("id_token payload should be valid JSON");

        assert_eq!(
            payload["nonce"],
            json!(nonce),
            "nonce claim is missing or incorrect in id_token payload: {}",
            payload
        );
    }
}
