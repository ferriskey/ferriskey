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
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

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
        pool: sqlx::PgPool,
        realm_name: String,
        client_id: Uuid,
        admin_user_id: Uuid,
        neighbour_client_id: Uuid,
        neighbour_role_id: Uuid,
        neighbour_client_entry_id: Uuid,
        neighbour_realm_entry_id: Uuid,
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

        let schema = format!("maintenance_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let realm_id: Uuid = sqlx::query_scalar("SELECT id FROM realms WHERE name = $1")
            .bind(&realm_name)
            .fetch_one(&pool)
            .await
            .expect("read principal realm id");

        let client_id: Uuid =
            sqlx::query_scalar("SELECT id FROM clients WHERE realm_id = $1 AND client_id = $2")
                .bind(realm_id)
                .bind(SEEDED_CLIENT_ID)
                .fetch_one(&pool)
                .await
                .expect("read principal client id");

        let admin_user_id: Uuid =
            sqlx::query_scalar("SELECT id FROM users WHERE realm_id = $1 AND username = $2")
                .bind(realm_id)
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("read admin user id");

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

        let neighbour_role_id = Uuid::new_v4();
        let neighbour_role_name = format!("neighbour-role-{}", Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO roles (id, name, description, permissions, realm_id, created_at, updated_at) VALUES ($1, $2, NULL, 0, $3, NOW(), NOW())",
        )
        .bind(neighbour_role_id)
        .bind(&neighbour_role_name)
        .bind(neighbour_realm_id)
        .execute(&pool)
        .await
        .expect("insert neighbour role");

        let neighbour_client_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, protocol, client_type, maintenance_enabled, created_at, updated_at) VALUES ($1, $2, $3, $4, 'openid-connect', 'confidential', TRUE, NOW(), NOW())",
        )
        .bind(neighbour_client_id)
        .bind(neighbour_realm_id)
        .bind("neighbour client")
        .bind(format!("neighbour-client-{}", Uuid::new_v4().simple()))
        .execute(&pool)
        .await
        .expect("insert neighbour client");

        let neighbour_client_entry_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO client_maintenance_whitelist (id, client_id, user_id, role_id, created_at) VALUES ($1, $2, NULL, $3, NOW())",
        )
        .bind(neighbour_client_entry_id)
        .bind(neighbour_client_id)
        .bind(neighbour_role_id)
        .execute(&pool)
        .await
        .expect("insert neighbour client whitelist entry");

        let neighbour_realm_entry_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realm_maintenance_whitelist (id, realm_id, user_id, role_id, created_at) VALUES ($1, $2, NULL, $3, NOW())",
        )
        .bind(neighbour_realm_entry_id)
        .bind(neighbour_realm_id)
        .bind(neighbour_role_id)
        .execute(&pool)
        .await
        .expect("insert neighbour realm whitelist entry");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            realm_name,
            client_id,
            admin_user_id,
            neighbour_client_id,
            neighbour_role_id,
            neighbour_client_entry_id,
            neighbour_realm_entry_id,
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

    async fn get_admin_token(server: &TestServer) -> String {
        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
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

    async fn client_entry_count(entry_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM client_maintenance_whitelist WHERE id = $1")
            .bind(entry_id)
            .fetch_one(&shared_ctx().pool)
            .await
            .expect("count client whitelist entries")
    }

    async fn realm_entry_count(entry_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM realm_maintenance_whitelist WHERE id = $1")
            .bind(entry_id)
            .fetch_one(&shared_ctx().pool)
            .await
            .expect("count realm whitelist entries")
    }

    async fn client_whitelist_size(client_id: Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM client_maintenance_whitelist WHERE client_id = $1")
            .bind(client_id)
            .fetch_one(&shared_ctx().pool)
            .await
            .expect("count client whitelist size")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn reading_the_whitelist_of_a_client_of_another_realm_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let foreign_client_id = shared_ctx().neighbour_client_id;
            let foreign_entry_id = shared_ctx().neighbour_client_entry_id;

            let response = server
                .get(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist",
                    realm(),
                    foreign_client_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "the whitelist of a client of another realm must not be readable through the principal realm: {}",
                response.text()
            );
            assert!(
                !response.text().contains(&foreign_entry_id.to_string()),
                "the refusal must not leak the foreign whitelist entries"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn removing_a_whitelist_entry_of_a_client_of_another_realm_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let foreign_client_id = shared_ctx().neighbour_client_id;
            let foreign_entry_id = shared_ctx().neighbour_client_entry_id;

            let response = server
                .delete(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist/{}",
                    realm(),
                    foreign_client_id,
                    foreign_entry_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "a whitelist entry of a client of another realm must not be deletable through the principal realm: {}",
                response.text()
            );
            assert_eq!(
                client_entry_count(foreign_entry_id).await,
                1,
                "the foreign whitelist entry must still be there after the refusal"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn removing_a_whitelist_entry_of_another_client_through_an_owned_client_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let owned_client_id = shared_ctx().client_id;
            let foreign_entry_id = shared_ctx().neighbour_client_entry_id;

            let response = server
                .delete(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist/{}",
                    realm(),
                    owned_client_id,
                    foreign_entry_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "an entry belonging to another client must not be deletable through an owned client: {}",
                response.text()
            );
            assert_eq!(
                client_entry_count(foreign_entry_id).await,
                1,
                "the entry of the other client must still be there after the refusal"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn removing_a_realm_whitelist_entry_of_another_realm_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let foreign_entry_id = shared_ctx().neighbour_realm_entry_id;

            let response = server
                .delete(&format!(
                    "/realms/{}/settings/maintenance/whitelist/{}",
                    realm(),
                    foreign_entry_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "a realm whitelist entry of another realm must not be deletable through the principal realm: {}",
                response.text()
            );
            assert_eq!(
                realm_entry_count(foreign_entry_id).await,
                1,
                "the foreign realm whitelist entry must still be there after the refusal"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn whitelisting_a_user_on_a_client_of_another_realm_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let foreign_client_id = shared_ctx().neighbour_client_id;
            let before = client_whitelist_size(foreign_client_id).await;

            let response = server
                .post(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist",
                    realm(),
                    foreign_client_id
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "user_id": shared_ctx().admin_user_id }))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "a user must not be whitelisted on a client of another realm: {}",
                response.text()
            );
            assert_eq!(
                client_whitelist_size(foreign_client_id).await,
                before,
                "the maintenance whitelist of the foreign client must be untouched"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn whitelisting_a_role_on_a_client_of_another_realm_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let foreign_client_id = shared_ctx().neighbour_client_id;
            let before = client_whitelist_size(foreign_client_id).await;

            let response = server
                .post(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist",
                    realm(),
                    foreign_client_id
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "role_id": shared_ctx().neighbour_role_id }))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "a role must not be whitelisted on a client of another realm: {}",
                response.text()
            );
            assert_eq!(
                client_whitelist_size(foreign_client_id).await,
                before,
                "the maintenance whitelist of the foreign client must be untouched"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn the_whitelist_of_a_client_of_the_caller_realm_stays_manageable() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let owned_client_id = shared_ctx().client_id;

            let listed = server
                .get(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist",
                    realm(),
                    owned_client_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                listed.status_code(),
                200,
                "the whitelist of an owned client must stay readable: {}",
                listed.text()
            );

            let added = server
                .post(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist",
                    realm(),
                    owned_client_id
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "user_id": shared_ctx().admin_user_id }))
                .await;

            assert_eq!(
                added.status_code(),
                201,
                "a user of the caller realm must stay whitelistable on an owned client: {}",
                added.text()
            );

            let body: Value = added.json();
            let entry_id: Uuid = body["data"]["id"]
                .as_str()
                .expect("id in response")
                .parse()
                .expect("parse entry id");

            let removed = server
                .delete(&format!(
                    "/realms/{}/clients/{}/maintenance/whitelist/{}",
                    realm(),
                    owned_client_id,
                    entry_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                removed.status_code(),
                200,
                "an entry of an owned client must stay deletable: {}",
                removed.text()
            );
            assert_eq!(
                client_entry_count(entry_id).await,
                0,
                "the entry of the owned client must be gone after the deletion"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test maintenance_cross_realm_test -- --ignored"]
    fn the_whitelist_of_the_caller_realm_stays_manageable() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let listed = server
                .get(&format!(
                    "/realms/{}/settings/maintenance/whitelist",
                    realm()
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                listed.status_code(),
                200,
                "the realm whitelist must stay readable: {}",
                listed.text()
            );

            let added = server
                .post(&format!(
                    "/realms/{}/settings/maintenance/whitelist",
                    realm()
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "user_id": shared_ctx().admin_user_id }))
                .await;

            assert_eq!(
                added.status_code(),
                201,
                "a user of the caller realm must stay whitelistable at realm level: {}",
                added.text()
            );

            let body: Value = added.json();
            let entry_id: Uuid = body["data"]["id"]
                .as_str()
                .expect("id in response")
                .parse()
                .expect("parse entry id");

            let removed = server
                .delete(&format!(
                    "/realms/{}/settings/maintenance/whitelist/{}",
                    realm(),
                    entry_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                removed.status_code(),
                200,
                "an entry of the caller realm must stay deletable: {}",
                removed.text()
            );
            assert_eq!(
                realm_entry_count(entry_id).await,
                0,
                "the entry of the caller realm must be gone after the deletion"
            );
        });
    }
}
