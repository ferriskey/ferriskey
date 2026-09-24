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

    const PASSWORD: &str = "Migrated-Pa55word!";

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

    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("supplied_user_id_test_{}", Uuid::new_v4().simple());

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

        let realm_name = "master".to_string();

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
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            pool,
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

    fn pool() -> &'static sqlx::PgPool {
        &shared_ctx().pool
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    async fn admin_token(server: &TestServer) -> String {
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

    async fn post_user(
        server: &TestServer,
        token: &str,
        target_realm: &str,
        body: Value,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/users", target_realm))
            .add_header("Authorization", auth_header(token))
            .json(&body)
            .await
    }

    fn user_body(username: &str, id: Option<Uuid>) -> Value {
        let mut body = json!({
            "username": username,
            "firstname": "Migrated",
            "lastname": "User",
            "email": format!("{username}@test.local"),
            "email_verified": true,
        });

        if let Some(id) = id {
            body["id"] = json!(id.to_string());
        }

        body
    }

    fn unique_username(prefix: &str) -> String {
        format!("{}-{}", prefix, Uuid::new_v4().simple())
    }

    fn created_user_id(response: TestResponse) -> String {
        let status = response.status_code();
        let text = response.text();
        assert_eq!(status, 200, "create user failed: {text}");
        let body: Value = serde_json::from_str(&text).expect("create user response is JSON");
        body["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string()
    }

    async fn create_realm(server: &TestServer, token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": name }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "creating realm {name} failed: {}",
            response.text()
        );
    }

    async fn realm_of_user(id: Uuid) -> Option<Uuid> {
        sqlx::query_as::<_, (Uuid,)>("SELECT realm_id FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool())
            .await
            .expect("query user row")
            .map(|row| row.0)
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test supplied_user_id_test -- --ignored"]
    fn an_omitted_id_is_still_minted_server_side() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;

            let response = post_user(
                &server,
                &token,
                realm(),
                user_body(&unique_username("minted"), None),
            )
            .await;

            let id =
                Uuid::parse_str(&created_user_id(response)).expect("the response id is a uuid");

            assert_eq!(
                id.get_version_num(),
                7,
                "an omitted id must keep the time-ordered v7 generator"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test supplied_user_id_test -- --ignored"]
    fn a_supplied_id_is_written_verbatim() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let supplied = Uuid::new_v4();

            let response = post_user(
                &server,
                &token,
                realm(),
                user_body(&unique_username("supplied"), Some(supplied)),
            )
            .await;

            assert_eq!(created_user_id(response), supplied.to_string());
            assert!(
                realm_of_user(supplied).await.is_some(),
                "the supplied id must be the primary key of the stored row"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test supplied_user_id_test -- --ignored"]
    fn a_supplied_id_becomes_the_token_subject() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let supplied = Uuid::new_v4();
            let username = unique_username("subject");

            let response = post_user(
                &server,
                &token,
                realm(),
                user_body(&username, Some(supplied)),
            )
            .await;
            let user_id = created_user_id(response);

            let reset = server
                .put(&format!(
                    "/realms/{}/users/{}/reset-password",
                    realm(),
                    user_id
                ))
                .add_header("Authorization", auth_header(&token))
                .json(&json!({
                    "value": PASSWORD,
                    "temporary": false,
                    "credential_type": "password",
                }))
                .await;
            assert_eq!(
                reset.status_code(),
                200,
                "setting the password failed: {}",
                reset.text()
            );

            let login = server
                .post(&format!(
                    "/realms/{}/protocol/openid-connect/token",
                    realm()
                ))
                .form(&[
                    ("grant_type", "password"),
                    ("client_id", "admin-cli"),
                    ("username", username.as_str()),
                    ("password", PASSWORD),
                    ("scope", "openid"),
                ])
                .await;
            assert_eq!(
                login.status_code(),
                200,
                "logging in as the migrated user failed: {}",
                login.text()
            );
            let login_body: Value = login.json();
            let access_token = login_body["access_token"]
                .as_str()
                .expect("access_token in response")
                .to_string();

            let userinfo = server
                .get(&format!(
                    "/realms/{}/protocol/openid-connect/userinfo",
                    realm()
                ))
                .add_header("Authorization", auth_header(&access_token))
                .await;
            assert_eq!(
                userinfo.status_code(),
                200,
                "userinfo failed: {}",
                userinfo.text()
            );
            let userinfo_body: Value = userinfo.json();

            assert_eq!(
                userinfo_body["sub"].as_str(),
                Some(supplied.to_string().as_str()),
                "the migrated id must be the subject a relying party sees"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test supplied_user_id_test -- --ignored"]
    fn an_id_taken_in_another_realm_answers_409() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let supplied = Uuid::new_v4();

            let other_realm = format!("other-{}", Uuid::new_v4().simple());
            create_realm(&server, &token, &other_realm).await;

            let first = post_user(
                &server,
                &token,
                &other_realm,
                user_body(&unique_username("elsewhere"), Some(supplied)),
            )
            .await;
            created_user_id(first);

            let second = post_user(
                &server,
                &token,
                realm(),
                user_body(&unique_username("collides"), Some(supplied)),
            )
            .await;

            let status = second.status_code();
            let text = second.text();
            assert_eq!(
                status, 409,
                "a primary-key collision from another realm must be a conflict, not a 500: {text}"
            );
            let body: Value = serde_json::from_str(&text).expect("conflict response is JSON");
            assert_eq!(body["reason"], json!("user_id_already_exists"));

            let owner = realm_of_user(supplied)
                .await
                .expect("the first user still owns the id");
            let target = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM realms WHERE name = $1")
                .bind(realm())
                .fetch_one(pool())
                .await
                .expect("query target realm")
                .0;
            assert_ne!(
                owner, target,
                "the refused request must not have moved the row"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test supplied_user_id_test -- --ignored"]
    fn an_unknown_field_is_refused() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;

            let mut body = user_body(&unique_username("strict"), None);
            body["enabled"] = json!(true);

            let response = post_user(&server, &token, realm(), body).await;

            assert_eq!(
                response.status_code(),
                400,
                "an unknown field must be refused instead of silently dropped: {}",
                response.text()
            );
        });
    }
}
