/// Integration tests for the organization HTTP endpoints.
///
/// These tests require a running PostgreSQL instance.  They are marked `#[ignore]`
/// so they do not block regular `cargo test` runs.  Run them explicitly with:
///
///   cargo test -p ferriskey-api -- --ignored
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

        let schema = format!("org_test_{}", Uuid::new_v4().simple());

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

        service
            .initialize_application(StartupConfig {
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "admin-cli".to_string(),
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

        assert_eq!(response.status_code(), 200, "token request failed");
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
    async fn list_organizations_returns_empty_initially() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let response = ctx
            .server
            .get(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn create_and_get_organization() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let create_response = ctx
            .server
            .post(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "name": "Acme Corp",
                "alias": "acme-corp",
                "description": "Acme Corporation",
                "enabled": true
            }))
            .await;

        assert_eq!(create_response.status_code(), 201);
        let created: Value = create_response.json();
        assert_eq!(created["name"], "Acme Corp");
        assert_eq!(created["alias"], "acme-corp");
        assert_eq!(created["enabled"], true);

        let org_id = created["id"].as_str().expect("organization id as string");

        let get_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(get_response.status_code(), 200);
        let fetched: Value = get_response.json();
        assert_eq!(fetched["name"], "Acme Corp");
        assert_eq!(fetched["alias"], "acme-corp");
    }

    #[tokio::test]
    #[ignore]
    async fn update_organization_details() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let create_response = ctx
            .server
            .post(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "name": "Initial Name",
                "alias": "initial-alias"
            }))
            .await;

        assert_eq!(create_response.status_code(), 201);
        let created: Value = create_response.json();
        let org_id = created["id"].as_str().expect("organization id");

        let update_response = ctx
            .server
            .put(&format!(
                "/realms/{}/organizations/{}",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "name": "Updated Name",
                "description": "Now with a description"
            }))
            .await;

        assert_eq!(update_response.status_code(), 200);
        let updated: Value = update_response.json();
        assert_eq!(updated["name"], "Updated Name");
        assert_eq!(updated["description"], "Now with a description");
        assert_eq!(updated["alias"], "initial-alias");
    }

    #[tokio::test]
    #[ignore]
    async fn delete_organization() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let create_response = ctx
            .server
            .post(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({
                "name": "To Delete",
                "alias": "to-delete"
            }))
            .await;

        assert_eq!(create_response.status_code(), 201);
        let created: Value = create_response.json();
        let org_id = created["id"].as_str().expect("organization id");

        let delete_response = ctx
            .server
            .delete(&format!(
                "/realms/{}/organizations/{}",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(delete_response.status_code(), 204);

        let get_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(get_response.status_code(), 404);
    }

    #[tokio::test]
    #[ignore]
    async fn create_organization_requires_name_and_alias() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let response = ctx
            .server
            .post(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "description": "missing name and alias" }))
            .await;

        assert_eq!(response.status_code(), 422);
    }

    #[tokio::test]
    #[ignore]
    async fn get_organization_returns_404_for_unknown_id() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;

        let unknown_id = Uuid::new_v4();
        let response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}",
                ctx.realm_name, unknown_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(response.status_code(), 404);
    }

    // ── Attribute tests ────────────────────────────────────────────────────────

    async fn create_test_org(ctx: &TestContext, token: &str, alias: &str) -> String {
        let response = ctx
            .server
            .post(&format!("/realms/{}/organizations", ctx.realm_name))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": alias, "alias": alias }))
            .await;
        assert_eq!(response.status_code(), 201);
        let body: Value = response.json();
        body["id"].as_str().expect("org id").to_string()
    }

    #[tokio::test]
    #[ignore]
    async fn list_attributes_returns_empty_initially() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "attr-test-org").await;

        let response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/attributes",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn upsert_and_list_attribute() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "upsert-attr-org").await;

        // Create attribute
        let put_response = ctx
            .server
            .put(&format!(
                "/realms/{}/organizations/{}/attributes/color",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "value": "blue" }))
            .await;

        assert_eq!(put_response.status_code(), 200);
        let attr: Value = put_response.json();
        assert_eq!(attr["key"], "color");
        assert_eq!(attr["value"], "blue");

        // List should now contain the attribute
        let list_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/attributes",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(list_response.status_code(), 200);
        let attrs: Value = list_response.json();
        assert_eq!(attrs.as_array().unwrap().len(), 1);
        assert_eq!(attrs[0]["key"], "color");
        assert_eq!(attrs[0]["value"], "blue");
    }

    #[tokio::test]
    #[ignore]
    async fn upsert_attribute_updates_existing_value() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "upsert-update-org").await;

        let attrs_url = format!(
            "/realms/{}/organizations/{}/attributes/theme",
            ctx.realm_name, org_id
        );

        ctx.server
            .put(&attrs_url)
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "value": "light" }))
            .await;

        let update_response = ctx
            .server
            .put(&attrs_url)
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "value": "dark" }))
            .await;

        assert_eq!(update_response.status_code(), 200);
        let attr: Value = update_response.json();
        assert_eq!(attr["value"], "dark");

        // List should still have only one entry
        let list_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/attributes",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        let attrs: Value = list_response.json();
        assert_eq!(attrs.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    #[ignore]
    async fn delete_attribute() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "delete-attr-org").await;

        let attrs_url = format!(
            "/realms/{}/organizations/{}/attributes/role",
            ctx.realm_name, org_id
        );

        ctx.server
            .put(&attrs_url)
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "value": "primary" }))
            .await;

        let delete_response = ctx
            .server
            .delete(&attrs_url)
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(delete_response.status_code(), 204);

        let list_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/attributes",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        let attrs: Value = list_response.json();
        assert_eq!(attrs.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn upsert_attribute_requires_value() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "validate-attr-org").await;

        let response = ctx
            .server
            .put(&format!(
                "/realms/{}/organizations/{}/attributes/key1",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "value": "" }))
            .await;

        assert_eq!(response.status_code(), 422);
    }

    // ── Member tests ───────────────────────────────────────────────────────────

    async fn create_test_user(ctx: &TestContext, token: &str, username: &str) -> String {
        let response = ctx
            .server
            .post(&format!("/realms/{}/users", ctx.realm_name))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "email": format!("{}@test.local", username),
                "enabled": true
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "failed to create user {username}"
        );
        let body: Value = response.json();
        body["id"].as_str().expect("user id").to_string()
    }

    #[tokio::test]
    #[ignore]
    async fn list_members_returns_empty_initially() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "member-empty-org").await;

        let response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(response.status_code(), 200);
        let body: Value = response.json();
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn add_and_list_member() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "add-member-org").await;
        let user_id = create_test_user(&ctx, &token, "member-user-1").await;

        let add_response = ctx
            .server
            .post(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "user_id": user_id }))
            .await;

        assert_eq!(add_response.status_code(), 201);
        let member: Value = add_response.json();
        assert_eq!(member["user_id"], user_id);
        assert_eq!(member["organization_id"], org_id);

        let list_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(list_response.status_code(), 200);
        let members: Value = list_response.json();
        assert_eq!(members.as_array().unwrap().len(), 1);
        assert_eq!(members[0]["user_id"], user_id);
    }

    #[tokio::test]
    #[ignore]
    async fn remove_member() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "remove-member-org").await;
        let user_id = create_test_user(&ctx, &token, "member-user-2").await;

        ctx.server
            .post(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "user_id": user_id }))
            .await;

        let remove_response = ctx
            .server
            .delete(&format!(
                "/realms/{}/organizations/{}/members/{}",
                ctx.realm_name, org_id, user_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(remove_response.status_code(), 204);

        let list_response = ctx
            .server
            .get(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        let members: Value = list_response.json();
        assert_eq!(members.as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    #[ignore]
    async fn list_user_organizations() {
        let ctx = setup().await;
        let token = get_admin_token(&ctx).await;
        let org_id = create_test_org(&ctx, &token, "user-org-list-org").await;
        let user_id = create_test_user(&ctx, &token, "member-user-3").await;

        ctx.server
            .post(&format!(
                "/realms/{}/organizations/{}/members",
                ctx.realm_name, org_id
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "user_id": user_id }))
            .await;

        let response = ctx
            .server
            .get(&format!(
                "/realms/{}/users/{}/organizations",
                ctx.realm_name, user_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await;

        assert_eq!(response.status_code(), 200);
        let orgs: Value = response.json();
        assert_eq!(orgs.as_array().unwrap().len(), 1);
        assert_eq!(orgs[0]["organization_id"], org_id);
    }
}
