/// Integration tests for the token preview endpoint:
/// `POST /realms/{realm_name}/clients/{client_id}/token-preview`.
///
/// Verifies the ticket's acceptance criteria end-to-end:
///   - returns the exact ticket-shaped response (5 fields, no effective_roles)
///   - without `user_id`, user-attribute mappers resolve to placeholder values
///   - with `user_id`, user-attribute mappers resolve real values
///   - applied mappers are attributed to their originating scope
///   - requires `ManageClientScopes` or `ManageRealm` (403 otherwise)
///   - never issues a real token
///
/// Runs as a single test with one shared setup: the router installs a global
/// Prometheus recorder via `PrometheusMetricLayer::pair()`, which panics if the
/// router is built more than once per process, so all criteria are exercised in
/// one server.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]` so it does not run
/// in regular `cargo test` (no local Postgres). Run explicitly with:
///
///   cargo test -p ferriskey-api --test token_preview_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST=localhost  DATABASE_PORT=5432
///   DATABASE_NAME=ferriskey  DATABASE_USER=ferriskey  DATABASE_PASSWORD=ferriskey
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

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

        let schema = format!("token_preview_test_{}", Uuid::new_v4().simple());

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
        })
        .await
        .expect("create service");

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());

        // Use a non-"admin-cli" default_client_id so the dedicated admin-cli
        // seeding (public/System, direct_access_grants_enabled=true) is not
        // short-circuited by the generic confidential default-client creation
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

        // Mirrors production startup (api/src/main.rs): data migrations seed the
        // realm default client scopes (openid/profile/email/roles + protocol
        // mappers) without which clients get no scopes/mappers.
        service
            .run_data_migrations()
            .await
            .expect("run data migrations");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::new(app).expect("create test server");

        TestContext { server, realm_name }
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
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

        assert_eq!(
            response.status_code(),
            200,
            "admin token request failed: {}",
            response.text()
        );
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    /// Create a client and return its UUID.
    async fn create_client(ctx: &TestContext, admin_token: &str) -> String {
        let client_id = format!("preview-client-{}", Uuid::new_v4().simple());

        let resp = ctx
            .server
            .post(&format!("/realms/{}/clients", ctx.realm_name))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "client_id": client_id,
                "name": "Preview Test Client",
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": false,
                "direct_access_grants_enabled": false,
                "enabled": true,
                "oauth_device_code_grant_enabled": false
            }))
            .await;

        assert_eq!(
            resp.status_code(),
            201,
            "client creation failed: {}",
            resp.text()
        );
        let body: Value = resp.json();
        body["id"]
            .as_str()
            .expect("client id in response")
            .to_string()
    }

    /// Look up a user by username, filtering in Rust (the `?username=` query
    /// parameter is not honored and would return the first user, e.g. admin).
    async fn user_id_by_username(ctx: &TestContext, admin_token: &str, username: &str) -> String {
        let users = ctx
            .server
            .get(&format!("/realms/{}/users", ctx.realm_name))
            .add_header("Authorization", auth_header(admin_token))
            .await;
        assert_eq!(
            users.status_code(),
            200,
            "list users failed: {}",
            users.text()
        );

        let users_body: Value = users.json();
        users_body["data"]
            .as_array()
            .expect("users data array")
            .iter()
            .find(|u| u["username"].as_str() == Some(username))
            .and_then(|u| u["id"].as_str().map(|id| id.to_string()))
            .unwrap_or_else(|| panic!("user {} not found in user list", username))
    }

    /// Create a user with a password and return (user_id, username, password).
    async fn create_user(
        ctx: &TestContext,
        admin_token: &str,
        username: &str,
    ) -> (String, String, String) {
        let resp = ctx
            .server
            .post(&format!("/realms/{}/users", ctx.realm_name))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "enabled": true,
                "email": format!("{}@test.local", username),
                "email_verified": true
            }))
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "user creation failed: {}",
            resp.text()
        );

        let user_id = user_id_by_username(ctx, admin_token, username).await;

        let password = "Preview#User2026!";

        let reset = ctx
            .server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                ctx.realm_name, user_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "value": password, "temporary": false }))
            .await;
        assert_eq!(
            reset.status_code(),
            200,
            "reset password failed: {}",
            reset.text()
        );

        (user_id, username.to_string(), password.to_string())
    }

    /// Obtain an access token via the password grant.
    async fn login(ctx: &TestContext, username: &str, password: &str) -> String {
        let token_resp = ctx
            .server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                ctx.realm_name
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await;

        assert_eq!(
            token_resp.status_code(),
            200,
            "token request for {} failed: {}",
            username,
            token_resp.text()
        );
        let body: Value = token_resp.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    /// Covers all acceptance criteria in one shared server:
    ///   C1 + C2 + C4  no user_id -> ticket shape, placeholders, scope attribution
    ///   C3             with user_id -> real user-attribute values
    ///   C5             403 without ManageClientScopes/ManageRealm
    #[tokio::test]
    #[ignore]
    async fn token_preview_acceptance_criteria() {
        let ctx = setup().await;
        let admin_token = get_admin_token(&ctx).await;
        let client_uuid = create_client(&ctx, &admin_token).await;

        // --- C1 + C2 + C4: no user_id -> ticket shape + placeholders + scope attribution ---
        let resp = ctx
            .server
            .post(&format!(
                "/realms/{}/clients/{}/token-preview",
                ctx.realm_name, client_uuid
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({ "scope": "openid profile email" }))
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "token-preview failed: {}",
            resp.text()
        );

        let body: Value = resp.json();

        // Exact ticket shape: five top-level fields, no effective_roles.
        let mut keys: Vec<&str> = body
            .as_object()
            .unwrap()
            .keys()
            .map(|s| s.as_str())
            .collect();
        keys.sort();
        assert_eq!(
            keys,
            vec![
                "access_token_claims",
                "active_scopes",
                "applied_mappers",
                "id_token_claims",
                "userinfo_claims",
            ]
        );

        // Placeholder values (criterion 2).
        assert_eq!(
            body["access_token_claims"]["preferred_username"],
            "preview_user"
        );
        assert_eq!(body["access_token_claims"]["given_name"], "Preview");
        assert_eq!(body["access_token_claims"]["family_name"], "User");
        assert_eq!(body["access_token_claims"]["email"], "preview@example.com");
        assert_eq!(body["access_token_claims"]["email_verified"], false);

        // active_scopes items are { name, type }.
        let scopes = body["active_scopes"]
            .as_array()
            .expect("active_scopes array");
        assert!(!scopes.is_empty());
        for scope in scopes {
            let obj = scope.as_object().expect("scope object");
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("type"));
            assert!(!obj.contains_key("protocol"));
        }

        // applied_mappers items are { scope, mapper, type } with scope attribution (criterion 4).
        let mappers = body["applied_mappers"]
            .as_array()
            .expect("applied_mappers array");
        assert!(!mappers.is_empty());
        for mapper in mappers {
            let obj = mapper.as_object().expect("mapper object");
            assert!(obj.contains_key("scope"));
            assert!(obj.contains_key("mapper"));
            assert!(obj.contains_key("type"));
            assert!(!obj.contains_key("config"));
        }
        // Every mapper is attributed to a non-empty scope.
        assert!(
            mappers
                .iter()
                .all(|m| !m["scope"].as_str().unwrap_or("").is_empty())
        );

        // --- C3: with user_id -> user-attribute mappers resolve real values ---
        let (real_user_id, real_username, _) =
            create_user(&ctx, &admin_token, "preview_real_user").await;

        let resp = ctx
            .server
            .post(&format!(
                "/realms/{}/clients/{}/token-preview",
                ctx.realm_name, client_uuid
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({ "scope": "openid profile email", "user_id": real_user_id }))
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "token-preview with user failed: {}",
            resp.text()
        );

        let body: Value = resp.json();
        assert_eq!(body["access_token_claims"]["sub"], real_user_id);
        assert_eq!(
            body["access_token_claims"]["preferred_username"],
            real_username
        );
        assert_eq!(
            body["access_token_claims"]["email"],
            format!("{}@test.local", real_username)
        );

        // --- C5: a user without ManageClientScopes/ManageRealm is rejected with 403 ---
        let (_, limited_username, limited_password) =
            create_user(&ctx, &admin_token, "preview_limited_user").await;
        let limited_token = login(&ctx, &limited_username, &limited_password).await;

        let resp = ctx
            .server
            .post(&format!(
                "/realms/{}/clients/{}/token-preview",
                ctx.realm_name, client_uuid
            ))
            .add_header("Authorization", auth_header(&limited_token))
            .json(&json!({ "scope": "openid" }))
            .await;

        assert_eq!(
            resp.status_code(),
            403,
            "expected 403 for user without manage permission, got: {}",
            resp.text()
        );
        let body: Value = resp.json();
        assert_eq!(body["status"], 403);
    }
}
