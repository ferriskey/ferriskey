#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
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

    const WEBAPP_URL: &str = "http://localhost:5555";
    const MASTER_REALM: &str = "master";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const MASTER_ADMIN_USERNAME: &str = "admin";
    const MASTER_ADMIN_PASSWORD: &str = "admin";
    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";
    const ALICE_USERNAME: &str = "alice";
    const BOB_USERNAME: &str = "bob";
    const TENANT_PASSWORD: &str = "Tenant-Passw0rd!";

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

        let schema = format!("token_exchange_policy_test_{}", Uuid::new_v4().simple());

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
                admin_username: MASTER_ADMIN_USERNAME.to_string(),
                admin_password: MASTER_ADMIN_PASSWORD.to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        let ctx = SharedContext {
            app: std::sync::Mutex::new(app),
        };

        seed_tenants(&ctx).await;

        ctx
    }

    async fn seed_tenants(ctx: &SharedContext) {
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
        let server = TestServer::new(app).expect("create test server");
        let master = master_token(&server).await;

        for realm in [TENANT_A, TENANT_B] {
            let response = server
                .post("/realms")
                .add_header("Authorization", auth_header(&master))
                .json(&json!({ "name": realm, "display_name": realm }))
                .await;
            assert_eq!(
                response.status_code(),
                201,
                "creating realm {realm} failed: {}",
                response.text()
            );
        }

        let alice_id = create_user(&server, &master, ALICE_USERNAME).await;
        create_user(&server, &master, BOB_USERNAME).await;

        let response = server
            .post(&format!("/realms/{TENANT_A}/roles"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "name": "tenant-a-client-admin",
                "description": "manage clients inside tenant-a",
                "permissions": ["manage_clients", "view_clients"],
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "creating alice's role failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let role_id = body["data"]["id"].as_str().expect("role id").to_string();

        let response = server
            .post(&format!(
                "/realms/{TENANT_A}/users/{alice_id}/roles/{role_id}"
            ))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({}))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "assigning alice's role failed: {}",
            response.text()
        );

        get_token(&server, TENANT_A, ALICE_USERNAME, TENANT_PASSWORD).await;
    }

    async fn create_user(server: &TestServer, master: &str, username: &str) -> String {
        let response = server
            .post(&format!("/realms/{TENANT_A}/users"))
            .add_header("Authorization", auth_header(master))
            .json(&json!({
                "username": username,
                "firstname": username,
                "lastname": "Tenant A",
                "email": format!("{username}@tenant-a.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "creating {username} failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let user_id = body["data"]["id"].as_str().expect("user id").to_string();

        let response = server
            .put(&format!(
                "/realms/{TENANT_A}/users/{user_id}/reset-password"
            ))
            .add_header("Authorization", auth_header(master))
            .json(&json!({
                "value": TENANT_PASSWORD,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "setting {username}'s password failed: {}",
            response.text()
        );

        user_id
    }

    fn make_server() -> TestServer {
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("bearer header is valid ASCII")
    }

    async fn get_token(server: &TestServer, realm: &str, username: &str, password: &str) -> String {
        let response = server
            .post(&format!("/realms/{realm}/protocol/openid-connect/token"))
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
            .expect("access_token in response")
            .to_string()
    }

    async fn master_token(server: &TestServer) -> String {
        get_token(
            server,
            MASTER_REALM,
            MASTER_ADMIN_USERNAME,
            MASTER_ADMIN_PASSWORD,
        )
        .await
    }

    struct TestClient {
        uuid: String,
        client_id: String,
    }

    async fn create_client(
        server: &TestServer,
        token: &str,
        realm: &str,
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
                "direct_access_grants_enabled": false,
                "enabled": true,
                "oauth_device_code_grant_enabled": false,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "creating client {client_id} in {realm} failed: {}",
            response.text()
        );
        let body: Value = response.json();

        TestClient {
            uuid: body["id"].as_str().expect("client id").to_string(),
            client_id,
        }
    }

    fn policies_path(realm: &str, client: &TestClient) -> String {
        format!(
            "/realms/{realm}/clients/{}/token-exchange-policies",
            client.uuid
        )
    }

    async fn create_policy(
        server: &TestServer,
        token: &str,
        realm: &str,
        client: &TestClient,
        target_audience: &str,
    ) -> axum_test::TestResponse {
        server
            .post(&policies_path(realm, client))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "target_audience": target_audience,
                "allowed_scopes": ["orders:read"],
                "allow_impersonation": true,
                "allow_delegation": false,
            }))
            .await
    }

    async fn list_policies(
        server: &TestServer,
        token: &str,
        realm: &str,
        client: &TestClient,
    ) -> Vec<Value> {
        let response = server
            .get(&policies_path(realm, client))
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response
            .json::<Value>()
            .as_array()
            .expect("policies are a JSON array")
            .clone()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn an_admin_creates_a_policy_and_lists_it() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let client = create_client(&server, &master, TENANT_A, "frontend").await;
            let audience = create_client(&server, &master, TENANT_A, "orders").await;

            let created =
                create_policy(&server, &master, TENANT_A, &client, &audience.client_id).await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let body: Value = created.json();
            assert_eq!(body["target_audience"], json!(audience.client_id));
            assert_eq!(body["client_id"], json!(client.uuid));
            assert_eq!(body["allowed_scopes"], json!(["orders:read"]));
            assert_eq!(body["allow_impersonation"], json!(true));

            let listed = list_policies(&server, &master, TENANT_A, &client).await;
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0]["id"], body["id"]);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn a_second_policy_for_the_same_audience_is_a_conflict() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let client = create_client(&server, &master, TENANT_A, "frontend").await;
            let audience = create_client(&server, &master, TENANT_A, "orders").await;

            let first =
                create_policy(&server, &master, TENANT_A, &client, &audience.client_id).await;
            assert_eq!(first.status_code(), 201, "{}", first.text());

            let second =
                create_policy(&server, &master, TENANT_A, &client, &audience.client_id).await;
            assert_eq!(second.status_code(), 409, "{}", second.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn an_audience_that_is_not_a_client_of_the_realm_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let client = create_client(&server, &master, TENANT_A, "frontend").await;
            let foreign = create_client(&server, &master, TENANT_B, "orders").await;

            let unknown =
                create_policy(&server, &master, TENANT_A, &client, "nobody-knows-me").await;
            assert_eq!(unknown.status_code(), 400, "{}", unknown.text());

            let other_realm =
                create_policy(&server, &master, TENANT_A, &client, &foreign.client_id).await;
            assert_eq!(other_realm.status_code(), 400, "{}", other_realm.text());

            assert!(
                list_policies(&server, &master, TENANT_A, &client)
                    .await
                    .is_empty()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn a_deleted_policy_disappears_from_the_list() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let client = create_client(&server, &master, TENANT_A, "frontend").await;
            let audience = create_client(&server, &master, TENANT_A, "orders").await;

            let created =
                create_policy(&server, &master, TENANT_A, &client, &audience.client_id).await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let body: Value = created.json();
            let policy_id = body["id"].as_str().expect("policy id");

            let deleted = server
                .delete(&format!("{}/{policy_id}", policies_path(TENANT_A, &client)))
                .add_header("Authorization", auth_header(&master))
                .await;
            assert_eq!(deleted.status_code(), 200, "{}", deleted.text());

            assert!(
                list_policies(&server, &master, TENANT_A, &client)
                    .await
                    .is_empty()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn a_user_without_client_permissions_is_forbidden() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let bob = get_token(&server, TENANT_A, BOB_USERNAME, TENANT_PASSWORD).await;
            let client = create_client(&server, &master, TENANT_A, "frontend").await;
            let audience = create_client(&server, &master, TENANT_A, "orders").await;

            let created =
                create_policy(&server, &bob, TENANT_A, &client, &audience.client_id).await;
            assert_eq!(created.status_code(), 403, "{}", created.text());

            let listed = server
                .get(&policies_path(TENANT_A, &client))
                .add_header("Authorization", auth_header(&bob))
                .await;
            assert_eq!(listed.status_code(), 403, "{}", listed.text());

            assert!(
                list_policies(&server, &master, TENANT_A, &client)
                    .await
                    .is_empty()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test token_exchange_policy_test -- --ignored"]
    fn a_policy_of_another_realm_cannot_be_deleted() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let alice = get_token(&server, TENANT_A, ALICE_USERNAME, TENANT_PASSWORD).await;

            let victim = create_client(&server, &master, TENANT_B, "victim").await;
            let victim_audience = create_client(&server, &master, TENANT_B, "orders").await;
            let created = create_policy(
                &server,
                &master,
                TENANT_B,
                &victim,
                &victim_audience.client_id,
            )
            .await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let body: Value = created.json();
            let policy_id = body["id"].as_str().expect("policy id");

            let own = create_client(&server, &alice, TENANT_A, "attacker").await;

            let through_own_client = server
                .delete(&format!("{}/{policy_id}", policies_path(TENANT_A, &own)))
                .add_header("Authorization", auth_header(&alice))
                .await;
            assert_eq!(
                through_own_client.status_code(),
                404,
                "{}",
                through_own_client.text()
            );

            let through_victim_client = server
                .delete(&format!("{}/{policy_id}", policies_path(TENANT_A, &victim)))
                .add_header("Authorization", auth_header(&alice))
                .await;
            assert_eq!(
                through_victim_client.status_code(),
                404,
                "{}",
                through_victim_client.text()
            );

            let remaining = list_policies(&server, &master, TENANT_B, &victim).await;
            assert_eq!(remaining.len(), 1);
            assert_eq!(remaining[0]["id"], body["id"]);
        });
    }
}
