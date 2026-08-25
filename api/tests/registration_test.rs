#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
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
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("registration_test_{}", Uuid::new_v4().simple());

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
            webapp_url: "http://localhost:5555".to_string(),
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
                webapp_url: "http://localhost:5555".to_string(),
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service.clone());
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
        }
    }

    fn make_server() -> TestServer {
        let ctx = shared_ctx();
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
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

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    fn enable_self_registration(server: &TestServer, token: &str) {
        rt().block_on(async {
            let response = server
                .put(&format!("/realms/{}/settings", realm()))
                .add_header("Authorization", auth_header(token))
                .json(&json!({ "user_registration_enabled": true }))
                .await;

            assert_eq!(
                response.status_code(),
                200,
                "enabling self-registration failed: {}",
                response.text()
            );
        });
    }

    async fn register(server: &TestServer, body: Value) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/registrations",
                realm()
            ))
            .json(&body)
            .await
    }

    fn strong_password(seed: &str) -> String {
        format!("Zq7!{seed}xV2#pLm9")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test registration_test -- --ignored"]
    fn a_self_registered_account_is_never_marked_verified() {
        let server = make_server();
        let admin = rt().block_on(get_admin_token(&server));
        enable_self_registration(&server, &admin);

        rt().block_on(async {
            let username = format!("self-{}", Uuid::new_v4().simple());
            let email = format!("{username}@example.com");

            let response = register(
                &server,
                json!({
                    "username": username,
                    "email": email,
                    "password": strong_password("Ab"),
                }),
            )
            .await;

            assert_eq!(
                response.status_code(),
                201,
                "registration failed: {}",
                response.text()
            );

            let listed = server
                .get(&format!("/realms/{}/users", realm()))
                .add_header("Authorization", auth_header(&admin))
                .await;
            assert_eq!(listed.status_code(), 200, "listing users: {}", listed.text());

            let body: Value = listed.json();
            let created = body["data"]
                .as_array()
                .expect("user list")
                .iter()
                .find(|u| u["username"] == username.as_str())
                .unwrap_or_else(|| panic!("the registered user must exist: {body}"));

            assert_eq!(
                created["email_verified"], false,
                "nobody proved control of this address, so the claim must not assert otherwise: {created}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test registration_test -- --ignored"]
    fn a_registration_body_without_credentials_is_refused() {
        let server = make_server();
        let admin = rt().block_on(get_admin_token(&server));
        enable_self_registration(&server, &admin);

        rt().block_on(async {
            let empty = register(&server, json!({})).await;
            assert_ne!(
                empty.status_code(),
                201,
                "an empty body must not create an account: {}",
                empty.text()
            );

            let blank = register(
                &server,
                json!({ "username": "", "email": "", "password": "" }),
            )
            .await;
            assert_ne!(
                blank.status_code(),
                201,
                "blank identifiers must not squat the realm's empty values: {}",
                blank.text()
            );

            let malformed = register(
                &server,
                json!({
                    "username": format!("u-{}", Uuid::new_v4().simple()),
                    "email": "not-an-address",
                    "password": strong_password("Cd"),
                }),
            )
            .await;
            assert_ne!(
                malformed.status_code(),
                201,
                "a malformed address must be refused: {}",
                malformed.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test registration_test -- --ignored"]
    fn an_address_differing_only_by_case_cannot_be_registered_twice() {
        let server = make_server();
        let admin = rt().block_on(get_admin_token(&server));
        enable_self_registration(&server, &admin);

        rt().block_on(async {
            let seed = Uuid::new_v4().simple().to_string();
            let email = format!("Case-{seed}@Example.com");

            let first = register(
                &server,
                json!({
                    "username": format!("first-{seed}"),
                    "email": email,
                    "password": strong_password("Ef"),
                }),
            )
            .await;
            assert_eq!(
                first.status_code(),
                201,
                "the first registration must succeed: {}",
                first.text()
            );

            let second = register(
                &server,
                json!({
                    "username": format!("second-{seed}"),
                    "email": email.to_lowercase(),
                    "password": strong_password("Gh"),
                }),
            )
            .await;

            assert!(
                second.status_code().is_client_error(),
                "the same address in another case must be refused as a client error, neither created nor a server error: {} {}",
                second.status_code(),
                second.text()
            );
        });
    }
}
