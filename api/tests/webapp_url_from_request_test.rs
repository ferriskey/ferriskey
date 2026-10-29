#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::PasswordHashingCost;
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
    use sqlx::Executor;
    use uuid::Uuid;

    const ROOT_PATH: &str = "/api";
    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const S256_CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    struct Context {
        from_request: std::sync::Mutex<Router>,
        configured: std::sync::Mutex<Router>,
        realm_name: String,
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
        let schema = format!("webapp_url_from_request_test_{}", Uuid::new_v4().simple());
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
            config_cache_ttl_seconds: 0,
            password_hashing: PasswordHashingCost::default(),
            refresh_token_format: Default::default(),
            webapp_url: WEBAPP_URL.to_string(),
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema,
                max_connections: 100,
                min_connections: 5,
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

        let mut configured_args = Args::default();
        configured_args.server.root_path = ROOT_PATH.to_string();

        let mut from_request_args = configured_args.clone();
        from_request_args.webapp_url_from_request = true;
        from_request_args.webapp_allowed_hosts = vec!["*.example.com".to_string()];

        Context {
            from_request: std::sync::Mutex::new(
                router(AppState::new(Arc::new(from_request_args), service.clone()))
                    .expect("build router"),
            ),
            configured: std::sync::Mutex::new(
                router(AppState::new(Arc::new(configured_args), service)).expect("build router"),
            ),
            realm_name,
        }
    }

    fn server(router: &std::sync::Mutex<Router>) -> TestServer {
        let app = router.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
    }

    async fn authorize(server: &TestServer, host: &str) -> TestResponse {
        let realm = &ctx().realm_name;

        server
            .get(&format!(
                "{ROOT_PATH}/realms/{realm}/protocol/openid-connect/auth"
            ))
            .add_header("host", HeaderValue::from_str(host).expect("host header"))
            .add_header("x-forwarded-proto", HeaderValue::from_static("https"))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", SEEDED_CLIENT_ID)
            .add_query_param(
                "redirect_uri",
                format!("{WEBAPP_URL}/realms/{realm}/authentication/callback").as_str(),
            )
            .add_query_param("scope", "openid")
            .add_query_param("state", "st")
            .add_query_param("code_challenge", S256_CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await
    }

    fn location(response: &TestResponse) -> String {
        response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webapp_url_from_request_test -- --ignored"]
    fn each_host_is_sent_to_the_login_page_on_its_own_host() {
        rt().block_on(async {
            let server = server(&ctx().from_request);
            let realm = &ctx().realm_name;

            for host in ["acme.example.com", "beta.example.com"] {
                let response = authorize(&server, host).await;
                assert_eq!(response.status_code(), 302, "{host}: {}", response.text());

                let expected = format!("https://{host}/realms/{realm}/authentication/login?");
                let location = location(&response);
                assert!(
                    location.starts_with(&expected),
                    "{host}: expected {expected}…, got {location}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webapp_url_from_request_test -- --ignored"]
    fn a_host_outside_the_allowlist_gets_no_login_page() {
        rt().block_on(async {
            let server = server(&ctx().from_request);

            let response = authorize(&server, "evil.example.net").await;

            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert!(location(&response).is_empty());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webapp_url_from_request_test -- --ignored"]
    fn without_the_flag_the_configured_webapp_url_is_kept_whatever_the_host() {
        rt().block_on(async {
            let server = server(&ctx().configured);
            let realm = &ctx().realm_name;

            let response = authorize(&server, "acme.example.com").await;
            assert_eq!(response.status_code(), 302, "{}", response.text());

            let expected = format!("{WEBAPP_URL}/realms/{realm}/authentication/login?");
            let location = location(&response);
            assert!(
                location.starts_with(&expected),
                "expected {expected}…, got {location}"
            );
        });
    }
}
