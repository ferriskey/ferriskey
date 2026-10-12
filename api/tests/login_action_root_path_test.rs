#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
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
    use sqlx::{Executor, Row};
    use uuid::Uuid;

    const ROOT_PATH: &str = "/api";
    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const TEMPORARY_PASSWORD: &str = "Str0ng!P@ssword#2024";
    const NEW_PASSWORD: &str = "An0ther!P@ssword#2025";

    struct Context {
        app: std::sync::Mutex<Router>,
        pool: sqlx::PgPool,
        realm_name: String,
        realm_id: Uuid,
    }

    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    static CTX: std::sync::OnceLock<Context> = std::sync::OnceLock::new();

    fn rt() -> &'static tokio::runtime::Runtime {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("build shared runtime")
        })
    }

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    fn ctx() -> &'static Context {
        CTX.get_or_init(|| match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(init_ctx())),
            Err(_) => rt().block_on(init_ctx()),
        })
    }

    async fn init_ctx() -> Context {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");
        let schema = format!("login_action_root_path_test_{}", Uuid::new_v4().simple());
        let base = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");

        sqlx::PgPool::connect(&base)
            .await
            .expect("connect admin pool")
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{schema}\""
            )))
            .await
            .expect("create schema");

        let pool = sqlx::PgPool::connect(&format!(
            "{base}?options=-c search_path={}",
            urlencoding::encode(&schema)
        ))
        .await
        .expect("connect schema pool");

        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let service = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: false,
            webapp_url: WEBAPP_URL.to_string(),
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema,
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

        let realm_id: Uuid = sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(&realm_name)
            .fetch_one(&pool)
            .await
            .expect("fetch master realm")
            .get("id");

        let mut args = Args::default();
        args.server.root_path = ROOT_PATH.to_string();
        let app = router(AppState::new(Arc::new(args), service)).expect("build router");

        Context {
            app: std::sync::Mutex::new(app),
            pool,
            realm_name,
            realm_id,
        }
    }

    fn server() -> TestServer {
        let app = ctx().app.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
    }

    fn realm_path(suffix: &str) -> String {
        format!("{ROOT_PATH}/realms/{}{suffix}", ctx().realm_name)
    }

    fn cookie_header(name: &str, value: &str) -> HeaderValue {
        HeaderValue::from_str(&format!("{name}={value}")).expect("valid cookie header")
    }

    async fn admin_token(server: &TestServer) -> String {
        let response = server
            .post(&realm_path("/protocol/openid-connect/token"))
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
            "admin token: {}",
            response.text()
        );
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    async fn seed_user_with_a_temporary_password(server: &TestServer) -> String {
        let token = admin_token(server).await;
        let bearer = HeaderValue::from_str(&format!("Bearer {token}")).expect("bearer header");
        let username = format!("temporary-{}", Uuid::new_v4().simple());

        let created = server
            .post(&realm_path("/users"))
            .add_header("Authorization", bearer.clone())
            .json(&json!({
                "username": username,
                "firstname": "Temporary",
                "lastname": "Password",
                "email": format!("{username}@test.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            created.status_code(),
            200,
            "create user: {}",
            created.text()
        );

        let user_id: Uuid =
            sqlx::query("SELECT id FROM users WHERE realm_id = $1 AND username = $2")
                .bind(ctx().realm_id)
                .bind(&username)
                .fetch_one(&ctx().pool)
                .await
                .expect("fetch the created user")
                .get("id");

        let reset = server
            .put(&realm_path(&format!("/users/{user_id}/reset-password")))
            .add_header("Authorization", bearer)
            .json(&json!({
                "value": TEMPORARY_PASSWORD,
                "temporary": true,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(
            reset.status_code(),
            200,
            "temporary password: {}",
            reset.text()
        );

        sqlx::query(
            "INSERT INTO user_required_actions (id, user_id, action, created_at) VALUES ($1, $2, $3, NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("update_password")
        .execute(&ctx().pool)
        .await
        .expect("insert the update_password required action");

        username
    }

    async fn log_in(server: &TestServer, username: &str, password: &str) -> TestResponse {
        let authorize = server
            .get(&realm_path("/protocol/openid-connect/auth"))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .add_query_param(
                "redirect_uri",
                format!(
                    "{WEBAPP_URL}/realms/{}/authentication/callback",
                    ctx().realm_name
                )
                .as_str(),
            )
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await;
        let status = authorize.status_code().as_u16();
        assert!(
            (300..=399).contains(&status),
            "expected a redirect from /auth, got {status}: {}",
            authorize.text()
        );
        let session_code = authorize.cookie("FERRISKEY_SESSION").value().to_string();

        server
            .post(&realm_path("/login-actions/authenticate"))
            .add_header("Cookie", cookie_header("FERRISKEY_SESSION", &session_code))
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .json(&json!({ "username": username, "password": password }))
            .await
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test login_action_root_path_test -- --ignored"]
    fn the_login_action_cookie_is_scoped_under_the_root_path() {
        rt().block_on(async {
            let server = server();
            let username = seed_user_with_a_temporary_password(&server).await;

            let login = log_in(&server, &username, TEMPORARY_PASSWORD).await;
            assert_eq!(login.status_code(), 200, "password step: {}", login.text());

            let cookie = login.cookie("FERRISKEY_LOGIN_ACTION");
            assert_eq!(cookie.path(), Some(realm_path("/login-actions").as_str()));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test login_action_root_path_test -- --ignored"]
    fn a_temporary_password_can_be_updated_behind_a_root_path() {
        rt().block_on(async {
            let server = server();
            let username = seed_user_with_a_temporary_password(&server).await;

            let login = log_in(&server, &username, TEMPORARY_PASSWORD).await;
            assert_eq!(login.status_code(), 200, "password step: {}", login.text());
            let cookie = login.cookie("FERRISKEY_LOGIN_ACTION");

            let update_path = realm_path("/login-actions/update-password");
            let cookie_path = cookie.path().unwrap_or("/");
            assert!(
                update_path.starts_with(cookie_path),
                "a browser only sends the cookie to {cookie_path}, so it never reaches {update_path}"
            );

            let updated = server
                .post(&update_path)
                .add_header(
                    "Cookie",
                    cookie_header("FERRISKEY_LOGIN_ACTION", cookie.value()),
                )
                .json(&json!({ "value": NEW_PASSWORD }))
                .await;
            assert_eq!(
                updated.status_code(),
                200,
                "update password: {}",
                updated.text()
            );

            let relogin = log_in(&server, &username, NEW_PASSWORD).await;
            assert_eq!(
                relogin.status_code(),
                200,
                "the new password must be accepted: {}",
                relogin.text()
            );
        });
    }
}
