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
    use serde_json::Value;
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
        realm_name: String,
        neighbour_user_id: Uuid,
        neighbour_credential_id: Uuid,
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

        let schema = format!("credential_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let neighbour_user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(neighbour_user_id)
        .bind(neighbour_realm_id)
        .bind("victim")
        .bind("Victim")
        .bind("User")
        .bind("victim@neighbour.local")
        .execute(&pool)
        .await
        .expect("insert neighbour user");

        let neighbour_credential_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data) VALUES ($1, $2, $3, $4, $5::jsonb)",
        )
        .bind(neighbour_credential_id)
        .bind("totp")
        .bind(neighbour_user_id)
        .bind("seed-secret")
        .bind("{}")
        .execute(&pool)
        .await
        .expect("insert neighbour credential");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            neighbour_user_id,
            neighbour_credential_id,
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

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn deleting_a_credential_of_a_user_in_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let response = server
                .delete(&format!(
                    "/realms/{}/users/{}/credentials/{}",
                    realm(),
                    shared_ctx().neighbour_user_id,
                    shared_ctx().neighbour_credential_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "an admin must not delete a credential of a user in another realm: {}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn listing_credentials_of_a_user_in_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let response = server
                .get(&format!(
                    "/realms/{}/users/{}/credentials",
                    realm(),
                    shared_ctx().neighbour_user_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                response.status_code(),
                404,
                "an admin must not list credentials of a user in another realm: {}",
                response.text()
            );
        });
    }
}
