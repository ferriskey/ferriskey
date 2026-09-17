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
        realm_name: String,
        realm_id: Uuid,
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

        let schema = format!("webhook_delivery_api_test_{}", Uuid::new_v4().simple());

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
            .expect("load realm id");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            realm_id,
            pool,
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

    async fn create_webhook(server: &TestServer, token: &str) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/webhooks", realm()))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "endpoint": "https://example.com/hook",
                "subscribers": ["user.created"],
                "headers": {},
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "webhook creation failed: {}",
            response.text()
        );
        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .and_then(|id| Uuid::parse_str(id).ok())
            .expect("webhook id in response")
    }

    async fn insert_pending_delivery(webhook_id: Uuid) -> Uuid {
        let ctx = shared_ctx();
        let delivery_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO webhook_deliveries
             (id, realm_id, webhook_id, event, resource_id, payload, status, attempt_count,
              next_attempt_at, created_at, updated_at)
             VALUES ($1, $2, $3, 'user.created', $4, '{\"event\":\"user.created\"}'::jsonb,
                     'pending', 0, NOW(), NOW(), NOW())",
        )
        .bind(delivery_id)
        .bind(ctx.realm_id)
        .bind(webhook_id)
        .bind(Uuid::new_v4())
        .execute(&ctx.pool)
        .await
        .expect("insert pending delivery");

        delivery_id
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_anonymous_caller_cannot_read_deliveries() {
        rt().block_on(async {
            let server = make_server();
            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries",
                    realm(),
                    Uuid::new_v4()
                ))
                .await;

            assert_eq!(response.status_code(), 401);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_fresh_webhook_has_an_empty_delivery_history() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries",
                    realm(),
                    webhook_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            let body: Value = response.json();
            assert_eq!(body["total"], 0);
            assert_eq!(body["data"].as_array().expect("data array").len(), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_page_size_above_the_cap_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries?limit=201",
                    realm(),
                    webhook_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 400, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_unknown_status_filter_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries?status=exploded",
                    realm(),
                    webhook_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 400, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_unknown_delivery_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}",
                    realm(),
                    webhook_id,
                    Uuid::new_v4()
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 404, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_delivery_cannot_be_read_through_another_webhook() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let owner = create_webhook(&server, &token).await;
            let stranger = create_webhook(&server, &token).await;
            let delivery_id = insert_pending_delivery(owner).await;

            let owned = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}",
                    realm(),
                    owner,
                    delivery_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;
            assert_eq!(owned.status_code(), 200, "{}", owned.text());

            let borrowed = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}",
                    realm(),
                    stranger,
                    delivery_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(
                borrowed.status_code(),
                404,
                "a delivery must not be reachable through a webhook that does not own it"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn replaying_a_pending_delivery_is_a_conflict() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;
            let delivery_id = insert_pending_delivery(webhook_id).await;

            let response = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}/retry",
                    realm(),
                    webhook_id,
                    delivery_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 409, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn replaying_a_failed_delivery_is_accepted() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;
            let delivery_id = insert_pending_delivery(webhook_id).await;

            sqlx::query(
                "UPDATE webhook_deliveries SET status = 'failed', next_attempt_at = NULL WHERE id = $1",
            )
            .bind(delivery_id)
            .execute(&shared_ctx().pool)
            .await
            .expect("mark delivery failed");

            let response = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}/retry",
                    realm(),
                    webhook_id,
                    delivery_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 202, "{}", response.text());

            let status: String =
                sqlx::query_scalar("SELECT status FROM webhook_deliveries WHERE id = $1")
                    .bind(delivery_id)
                    .fetch_one(&shared_ctx().pool)
                    .await
                    .expect("reload delivery status");

            assert_eq!(status, "pending");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_delivery_response_never_discloses_the_webhook_secret() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;
            let delivery_id = insert_pending_delivery(webhook_id).await;

            let secret: String = sqlx::query_scalar("SELECT secret FROM webhooks WHERE id = $1")
                .bind(webhook_id)
                .fetch_one(&shared_ctx().pool)
                .await
                .expect("load webhook secret");

            let response = server
                .get(&format!(
                    "/realms/{}/webhooks/{}/deliveries/{}",
                    realm(),
                    webhook_id,
                    delivery_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            let text = response.text();
            assert!(!text.contains(&secret), "the webhook secret leaked");
            assert!(!text.contains("\"secret\""));
        });
    }

    async fn put_realm_settings(server: &TestServer, token: &str, body: Value) -> u16 {
        server
            .put(&format!("/realms/{}/settings", realm()))
            .add_header("Authorization", auth_header(token))
            .json(&body)
            .await
            .status_code()
            .as_u16()
    }

    async fn put_webhook(server: &TestServer, token: &str, webhook_id: Uuid, policy: Value) -> u16 {
        server
            .put(&format!("/realms/{}/webhooks/{}", realm(), webhook_id))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "endpoint": "https://example.com/hook",
                "subscribers": ["user.created"],
                "retry_policy": policy,
            }))
            .await
            .status_code()
            .as_u16()
    }

    async fn effective_max_attempts(server: &TestServer, token: &str, webhook_id: Uuid) -> u64 {
        let response = server
            .get(&format!("/realms/{}/webhooks/{}", realm(), webhook_id))
            .add_header("Authorization", auth_header(token))
            .await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
        let body: Value = response.json();
        body["effective_retry_policy"]["max_attempts"]
            .as_u64()
            .expect("effective_retry_policy in response")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn retry_policy_resolution_walks_webhook_then_realm_then_system_default() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            assert_eq!(
                effective_max_attempts(&server, &token, webhook_id).await,
                5,
                "a webhook with no override anywhere must inherit the system default"
            );

            assert_eq!(
                put_realm_settings(&server, &token, json!({ "webhook_retry_max_attempts": 12 }))
                    .await,
                200
            );
            assert_eq!(
                effective_max_attempts(&server, &token, webhook_id).await,
                12,
                "the realm default must apply when the webhook sets nothing"
            );

            assert_eq!(
                put_webhook(&server, &token, webhook_id, json!({ "max_attempts": 3 })).await,
                200
            );
            assert_eq!(
                effective_max_attempts(&server, &token, webhook_id).await,
                3,
                "the webhook override must win over the realm default"
            );

            assert_eq!(
                put_webhook(&server, &token, webhook_id, json!({})).await,
                200
            );
            assert_eq!(
                effective_max_attempts(&server, &token, webhook_id).await,
                12,
                "clearing the webhook override must fall back to the realm default"
            );

            assert_eq!(
                put_realm_settings(
                    &server,
                    &token,
                    json!({ "webhook_retry_max_attempts": null })
                )
                .await,
                200
            );
            assert_eq!(
                effective_max_attempts(&server, &token, webhook_id).await,
                5,
                "clearing the realm default must fall back to the system default"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_out_of_bounds_webhook_retry_policy_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            assert_eq!(
                put_webhook(&server, &token, webhook_id, json!({ "max_attempts": 0 })).await,
                400
            );
            assert_eq!(
                put_webhook(&server, &token, webhook_id, json!({ "max_attempts": 21 })).await,
                400
            );
            assert_eq!(
                put_webhook(&server, &token, webhook_id, json!({ "base_delay_ms": 10 })).await,
                400
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_out_of_bounds_realm_retry_policy_is_rejected() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            assert_eq!(
                put_realm_settings(&server, &token, json!({ "webhook_retry_max_attempts": 0 }))
                    .await,
                400
            );
            assert_eq!(
                put_realm_settings(&server, &token, json!({ "webhook_retry_max_attempts": -1 }))
                    .await,
                400
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_webhook_can_be_created_without_any_subscriber() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let response = server
                .post(&format!("/realms/{}/webhooks", realm()))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({
                    "endpoint": "https://example.com/hook",
                    "subscribers": [],
                    "headers": {},
                }))
                .await;

            assert_eq!(
                response.status_code(),
                200,
                "a webhook with no subscriber must be creatable: {}",
                response.text()
            );

            let body: Value = response.json();
            let webhook_id = body["data"]["id"].as_str().expect("webhook id");

            let listed = server
                .get(&format!("/realms/{}/webhooks/{}", realm(), webhook_id))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(listed.status_code(), 200);
            let listed_body: Value = listed.json();
            assert_eq!(
                listed_body["subscribers"]
                    .as_array()
                    .expect("subscribers array")
                    .len(),
                0
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn a_rejected_endpoint_says_why() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let plain_http = server
                .post(&format!("/realms/{}/webhooks", realm()))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "endpoint": "http://example.com/hook", "subscribers": [] }))
                .await;

            assert_eq!(plain_http.status_code(), 400);
            assert!(
                plain_http.text().contains("https"),
                "a plain-http endpoint must be told to use https: {}",
                plain_http.text()
            );

            let loopback = server
                .post(&format!("/realms/{}/webhooks", realm()))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({ "endpoint": "https://localhost/hook", "subscribers": [] }))
                .await;

            assert_eq!(loopback.status_code(), 400);
            let body = loopback.text();
            assert!(
                body.contains("loopback or private address"),
                "a loopback endpoint must be told what is wrong with it: {body}"
            );
            assert!(
                !body.contains("localhost") && !body.contains("127.0.0.1"),
                "the rejection must not echo the host back: {body}"
            );
        });
    }

    async fn stored_secret(webhook_id: Uuid) -> String {
        sqlx::query_scalar("SELECT secret FROM webhooks WHERE id = $1")
            .bind(webhook_id)
            .fetch_one(&shared_ctx().pool)
            .await
            .expect("load webhook secret")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn rotating_returns_a_new_secret_once_and_replaces_the_stored_one() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let before = stored_secret(webhook_id).await;

            let response = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/secret/rotate",
                    realm(),
                    webhook_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            let body: Value = response.json();
            let revealed = body["secret"].as_str().expect("secret in response");

            assert_ne!(revealed, before, "rotation must mint a different secret");
            assert!(revealed.len() >= 32, "a secret must not be trivially short");

            let after = stored_secret(webhook_id).await;
            assert_eq!(
                after, revealed,
                "the returned secret must be the stored one"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn the_secret_is_returned_by_rotation_and_by_nothing_else() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;
            let webhook_id = create_webhook(&server, &token).await;

            let rotated = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/secret/rotate",
                    realm(),
                    webhook_id
                ))
                .add_header("Authorization", auth_header(&token))
                .await;
            let secret = rotated.json::<Value>()["secret"]
                .as_str()
                .expect("secret")
                .to_string();

            for path in [
                format!("/realms/{}/webhooks", realm()),
                format!("/realms/{}/webhooks/{}", realm(), webhook_id),
            ] {
                let response = server
                    .get(&path)
                    .add_header("Authorization", auth_header(&token))
                    .await;

                assert!(
                    !response.text().contains(&secret),
                    "{path} disclosed the signing secret"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn rotating_an_unknown_webhook_is_not_found() {
        rt().block_on(async {
            let server = make_server();
            let token = get_admin_token(&server).await;

            let response = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/secret/rotate",
                    realm(),
                    Uuid::new_v4()
                ))
                .add_header("Authorization", auth_header(&token))
                .await;

            assert_eq!(response.status_code(), 404, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_delivery_api_test -- --ignored"]
    fn an_anonymous_caller_cannot_rotate_a_secret() {
        rt().block_on(async {
            let server = make_server();

            let response = server
                .post(&format!(
                    "/realms/{}/webhooks/{}/secret/rotate",
                    realm(),
                    Uuid::new_v4()
                ))
                .await;

            assert_eq!(response.status_code(), 401);
        });
    }
}
