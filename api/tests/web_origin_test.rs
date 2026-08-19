#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{
        Router,
        http::{HeaderValue, Method},
    };
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
        realm_name: String,
        neighbour_realm_name: String,
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

        let schema = format!("web_origin_test_{}", Uuid::new_v4().simple());

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

        let neighbour_realm_name = format!("neighbour-{}", Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(&neighbour_realm_name)
        .execute(&pool)
        .await
        .expect("insert neighbour realm");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            neighbour_realm_name,
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

    async fn create_client(server: &TestServer, token: &str, realm_name: &str) -> String {
        let client_id = format!("web-origin-client-{}", Uuid::new_v4().simple());

        let response = server
            .post(&format!("/realms/{}/clients", realm_name))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "client_id": client_id,
                "name": "Web Origin Test Client",
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
            response.status_code(),
            201,
            "client creation failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["id"]
            .as_str()
            .expect("client id in response")
            .to_string()
    }

    async fn preflight(server: &TestServer, path: &str, origin: &str) -> Option<String> {
        let response = server
            .method(Method::OPTIONS, path)
            .add_header("Origin", origin.parse::<HeaderValue>().unwrap())
            .add_header(
                "Access-Control-Request-Method",
                HeaderValue::from_static("POST"),
            )
            .await;

        response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string())
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn a_registered_origin_may_exchange_a_code_at_the_token_endpoint() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;
            let origin = "https://spa.example.com";

            let registered = server
                .post(&format!(
                    "/realms/{}/clients/{}/web-origins",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": origin }))
                .await;

            assert_eq!(
                registered.status_code(),
                201,
                "registering the origin failed: {}",
                registered.text()
            );

            let allowed = preflight(
                &server,
                &format!("/realms/{}/protocol/openid-connect/token", realm()),
                origin,
            )
            .await;

            assert_eq!(allowed.as_deref(), Some(origin));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn an_origin_nobody_registered_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let allowed = preflight(
                &server,
                &format!("/realms/{}/protocol/openid-connect/token", realm()),
                "https://attacker.example.com",
            )
            .await;

            assert_eq!(allowed, None);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn an_origin_registered_in_one_realm_is_refused_on_another() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;
            let origin = "https://tenant-a-only.example.com";

            server
                .post(&format!(
                    "/realms/{}/clients/{}/web-origins",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": origin }))
                .await;

            let allowed = preflight(
                &server,
                &format!(
                    "/realms/{}/protocol/openid-connect/token",
                    shared_ctx().neighbour_realm_name
                ),
                origin,
            )
            .await;

            assert_eq!(allowed, None);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn the_sentinel_allows_the_origin_of_a_literal_redirect_uri() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;

            server
                .post(&format!(
                    "/realms/{}/clients/{}/redirects",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "https://derived.example.com/callback", "enabled": true }))
                .await;

            let registered = server
                .post(&format!(
                    "/realms/{}/clients/{}/web-origins",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "+" }))
                .await;
            assert_eq!(registered.status_code(), 201, "{}", registered.text());

            let allowed = preflight(
                &server,
                &format!("/realms/{}/protocol/openid-connect/token", realm()),
                "https://derived.example.com",
            )
            .await;

            assert_eq!(allowed.as_deref(), Some("https://derived.example.com"));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn a_value_that_is_not_an_origin_is_rejected_on_write() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;

            let response = server
                .post(&format!(
                    "/realms/{}/clients/{}/web-origins",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "https://app.example.com/callback" }))
                .await;

            assert_eq!(
                response.status_code(),
                400,
                "an origin carrying a path must be refused: {}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn the_wildcard_is_rejected_on_write() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;

            let response = server
                .post(&format!(
                    "/realms/{}/clients/{}/web-origins",
                    realm(),
                    client_uuid
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "*" }))
                .await;

            assert_eq!(
                response.status_code(),
                400,
                "the wildcard cannot coexist with credentialed CORS: {}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn registering_the_same_origin_twice_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;
            let path = format!("/realms/{}/clients/{}/web-origins", realm(), client_uuid);

            let first = server
                .post(&path)
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "https://duplicate.example.com" }))
                .await;
            assert_eq!(first.status_code(), 201, "{}", first.text());

            let second = server
                .post(&path)
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": "https://duplicate.example.com" }))
                .await;

            assert_eq!(
                second.status_code(),
                400,
                "a duplicate is the administrator's mistake, not a server fault: {}",
                second.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test web_origin_test -- --ignored"]
    fn removing_an_origin_stops_the_preflight_immediately() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let client_uuid = create_client(&server, &token, realm()).await;
            let origin = "https://revoked.example.com";
            let path = format!("/realms/{}/clients/{}/web-origins", realm(), client_uuid);

            let created = server
                .post(&path)
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "value": origin }))
                .await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let body: Value = created.json();
            let origin_id = body["id"].as_str().expect("web origin id in response");

            let token_path = format!("/realms/{}/protocol/openid-connect/token", realm());
            assert_eq!(
                preflight(&server, &token_path, origin).await.as_deref(),
                Some(origin)
            );

            let deleted = server
                .delete(&format!("{}/{}", path, origin_id))
                .add_header("Authorization", auth_header(&token))
                .await;
            assert_eq!(deleted.status_code(), 200, "{}", deleted.text());

            assert_eq!(preflight(&server, &token_path, origin).await, None);
        });
    }
}
