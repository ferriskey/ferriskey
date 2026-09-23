#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
    use axum_test::{TestResponse, TestServer};
    use bcrypt::Version;
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

    const PASSWORD: &str = "Imported-Pa55word!";

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

        let schema = format!("password_import_test_{}", Uuid::new_v4().simple());

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

    fn bcrypt_hash(password: &str, cost: u32) -> String {
        bcrypt::hash_with_result(password, cost)
            .expect("bcrypt hash")
            .format_for_version(Version::TwoA)
    }

    async fn login(server: &TestServer, username: &str, password: &str) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await
    }

    async fn admin_token(server: &TestServer) -> String {
        let response = login(server, "admin", "admin").await;
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

    async fn create_user(server: &TestServer, token: &str) -> (String, String) {
        let username = format!("imported-{}", Uuid::new_v4().simple());
        let response = server
            .post(&format!("/realms/{}/users", realm()))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "firstname": "Imported",
                "lastname": "User",
                "email": format!("{username}@test.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "create user failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let user_id = body["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();
        (user_id, username)
    }

    async fn import(server: &TestServer, token: &str, user_id: &str, body: Value) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/users/{}/credentials/import",
                realm(),
                user_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&body)
            .await
    }

    fn bcrypt_body(hash: &str, cost: u32) -> Value {
        json!({
            "algorithm": "bcrypt",
            "secret_data": hash,
            "hash_iterations": cost,
        })
    }

    async fn stored_password(user_id: &str) -> (Option<String>, String, Value) {
        let user_id = Uuid::parse_str(user_id).expect("user id");
        sqlx::query_as::<_, (Option<String>, String, Value)>(
            "SELECT salt, secret_data, credential_data FROM credentials \
             WHERE user_id = $1 AND credential_type = 'password'",
        )
        .bind(user_id)
        .fetch_one(pool())
        .await
        .expect("password credential row")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test password_import_test -- --ignored"]
    fn an_imported_bcrypt_password_logs_in_and_is_rehashed_to_argon2id() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let (user_id, username) = create_user(&server, &token).await;
            let hash = bcrypt_hash(PASSWORD, 4);

            let imported = import(&server, &token, &user_id, bcrypt_body(&hash, 4)).await;
            assert_eq!(imported.status_code(), 201, "{}", imported.text());

            let (salt, secret_data, data) = stored_password(&user_id).await;
            assert_eq!(salt, None);
            assert_eq!(secret_data, hash);
            assert_eq!(data["algorithm"], json!("bcrypt"));

            let wrong = login(&server, &username, "not-the-password").await;
            assert_eq!(wrong.status_code(), 400, "{}", wrong.text());

            let first = login(&server, &username, PASSWORD).await;
            assert_eq!(first.status_code(), 200, "{}", first.text());

            let (salt, secret_data, data) = stored_password(&user_id).await;
            assert!(salt.is_some());
            assert!(secret_data.starts_with("$argon2id$"), "{secret_data}");
            assert_eq!(data["algorithm"], json!("argon2id"));

            let second = login(&server, &username, PASSWORD).await;
            assert_eq!(second.status_code(), 200, "{}", second.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test password_import_test -- --ignored"]
    fn importing_over_an_existing_password_is_a_conflict() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let (user_id, username) = create_user(&server, &token).await;
            let hash = bcrypt_hash(PASSWORD, 4);

            let first = import(&server, &token, &user_id, bcrypt_body(&hash, 4)).await;
            assert_eq!(first.status_code(), 201, "{}", first.text());

            let replay = import(
                &server,
                &token,
                &user_id,
                bcrypt_body(&bcrypt_hash("another-password", 4), 4),
            )
            .await;
            assert_eq!(replay.status_code(), 409, "{}", replay.text());
            let body: Value = replay.json();
            assert_eq!(body["reason"], json!("password_credential_already_exists"));

            let (_, secret_data, _) = stored_password(&user_id).await;
            assert_eq!(secret_data, hash);

            let ok = login(&server, &username, PASSWORD).await;
            assert_eq!(ok.status_code(), 200, "{}", ok.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test password_import_test -- --ignored"]
    fn a_malformed_hash_is_rejected_without_creating_a_credential() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let (user_id, _) = create_user(&server, &token).await;
            let hash = bcrypt_hash(PASSWORD, 4);

            for body in [
                bcrypt_body(&hash, 10),
                bcrypt_body(&hash[..59], 4),
                json!({ "algorithm": "md5", "secret_data": "5f4dcc3b5aa765d61d8327deb882cf99", "hash_iterations": 1 }),
            ] {
                let response = import(&server, &token, &user_id, body.clone()).await;
                assert_eq!(response.status_code(), 422, "{body}: {}", response.text());
            }

            let count: (i64,) = sqlx::query_as(
                "SELECT count(*) FROM credentials WHERE user_id = $1 AND credential_type = 'password'",
            )
            .bind(Uuid::parse_str(&user_id).expect("user id"))
            .fetch_one(pool())
            .await
            .expect("count credentials");
            assert_eq!(count.0, 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test password_import_test -- --ignored"]
    fn importing_requires_an_authenticated_admin() {
        let server = make_server();
        rt().block_on(async {
            let token = admin_token(&server).await;
            let (user_id, _) = create_user(&server, &token).await;

            let response = server
                .post(&format!(
                    "/realms/{}/users/{}/credentials/import",
                    realm(),
                    user_id
                ))
                .json(&bcrypt_body(&bcrypt_hash(PASSWORD, 4), 4))
                .await;
            assert_eq!(response.status_code(), 401, "{}", response.text());
        });
    }
}
