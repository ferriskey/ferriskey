#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::HashSet, env, sync::Arc};

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
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const MASTER_REALM: &str = "master";
    const VIEWER_PASSWORD: &str = "V1ewer-Tenant-Pw!";
    const SEED_COUNT: usize = 25;
    const SECRET_MARK: &str = "s3cr3t-value";
    const DISPLAY_NAMES: [&str; 4] = ["Alpha Login", "Beta Gate", "Gamma SSO", "Delta Sign-in"];
    const PROVIDER_TYPES: [&str; 5] = ["oidc", "oauth2", "google", "saml", "github"];
    const SORT_FIELDS: [&str; 6] = [
        "alias",
        "display_name",
        "provider_id",
        "enabled",
        "created_at",
        "updated_at",
    ];
    const CONFIG_VARIANTS: usize = 14;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Health {
        Healthy,
        Degraded,
        Error,
    }

    impl Health {
        fn param(self) -> &'static str {
            match self {
                Health::Healthy => "healthy",
                Health::Degraded => "degraded",
                Health::Error => "error",
            }
        }
    }

    fn full_config(i: usize) -> serde_json::Map<String, Value> {
        let mut config = serde_json::Map::new();
        config.insert("client_id".into(), json!(format!("client-{i:02}")));
        config.insert(
            "client_secret".into(),
            json!(format!("{SECRET_MARK}-{i:02}")),
        );
        config.insert(
            "authorization_url".into(),
            json!(format!("https://idp-{i:02}.example/authorize")),
        );
        config.insert(
            "token_url".into(),
            json!(format!("https://idp-{i:02}.example/token")),
        );
        config
    }

    fn config_variant(i: usize) -> (Value, Health) {
        let mut config = full_config(i);
        let health = match i % CONFIG_VARIANTS {
            0 => {
                config.insert("scopes".into(), json!("openid email"));
                Health::Healthy
            }
            1 => {
                config.insert("scopes".into(), json!(["openid"]));
                Health::Healthy
            }
            2 => Health::Degraded,
            3 => {
                config.insert("scopes".into(), json!(""));
                Health::Degraded
            }
            4 => {
                config.remove("client_id");
                config.insert("scopes".into(), json!("openid"));
                Health::Error
            }
            5 => {
                config.insert("client_secret".into(), Value::Null);
                config.insert("scopes".into(), json!("openid"));
                Health::Error
            }
            6 => {
                config.insert("authorization_url".into(), json!(""));
                config.insert("scopes".into(), json!("openid"));
                Health::Error
            }
            7 => {
                config.remove("token_url");
                Health::Error
            }
            8 => {
                config.insert("client_secret".into(), json!(""));
                config.insert("scopes".into(), json!("x"));
                Health::Healthy
            }
            9 => {
                config.insert("scopes".into(), Value::Null);
                Health::Degraded
            }
            10 => {
                config.insert("scopes".into(), json!([]));
                Health::Healthy
            }
            11 => {
                config.insert("token_url".into(), json!(false));
                config.insert("scopes".into(), json!("openid"));
                Health::Error
            }
            12 => {
                config.insert("scopes".into(), json!(0));
                Health::Degraded
            }
            _ => {
                config.insert("client_id".into(), json!(""));
                config.insert("scopes".into(), json!("openid"));
                Health::Error
            }
        };
        (Value::Object(config), health)
    }

    struct Seed {
        id: Uuid,
        alias: String,
        display_name: Option<&'static str>,
        provider_id: &'static str,
        enabled: bool,
        config: Value,
        health: Health,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let (config, health) = config_variant(i);
            Self {
                id: Uuid::new_v4(),
                alias: format!("idp-{i:02}"),
                display_name: if i % 7 == 6 {
                    None
                } else {
                    Some(DISPLAY_NAMES[i % DISPLAY_NAMES.len()])
                },
                provider_id: PROVIDER_TYPES[i % PROVIDER_TYPES.len()],
                enabled: !i.is_multiple_of(4),
                config,
                health,
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "alias" => SortKey::Text(self.alias.clone()),
                "display_name" => SortKey::Nullable(
                    self.display_name.is_none(),
                    self.display_name.unwrap_or_default().to_string(),
                ),
                "provider_id" => SortKey::Text(self.provider_id.to_string()),
                "enabled" => SortKey::Flag(self.enabled),
                "created_at" => SortKey::Minute(self.created_minute),
                "updated_at" => SortKey::Minute(self.updated_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(String),
        Nullable(bool, String),
        Flag(bool),
        Minute(i32),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        viewer_token: String,
        realm: String,
        other_realm: String,
        seeds: Vec<Seed>,
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

    fn ctx() -> &'static SharedContext {
        CTX.get_or_init(|| rt().block_on(init_shared_ctx()))
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

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!(
            "identity_providers_listing_test_{}",
            Uuid::new_v4().simple()
        );

        let admin_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");
        let admin_pool = PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{schema}\""
            )))
            .await
            .expect("create schema");

        let schema_url = format!(
            "postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}?options=-c search_path={}",
            urlencoding::encode(&schema)
        );
        let pool = PgPool::connect(&schema_url)
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
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");
        let server = TestServer::new(app.clone()).expect("create fixture server");

        let admin_token = direct_grant(&server, MASTER_REALM, "admin", "admin").await;

        let suffix = Uuid::new_v4().simple().to_string();
        let realm = format!("listing-{}", &suffix[..8]);
        let other_realm = format!("other-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let existing: i64 =
            sqlx::query_scalar("SELECT count(*) FROM identity_providers WHERE realm_id = $1::uuid")
                .bind(realm_id.to_string())
                .fetch_one(&pool)
                .await
                .expect("count identity providers");
        assert_eq!(
            existing, 0,
            "a fresh realm must start without identity providers"
        );

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, seed).await;
        }

        for alias in [
            "idp-50",
            "idp-51",
            "idp-52",
            "pct%idp",
            "pctxidp",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_plain_provider(&pool, other_realm_id, alias).await;
        }

        let viewer_username = format!("viewer-{}", &suffix[..8]);
        let viewer_id = create_user(&server, &admin_token, &other_realm, &viewer_username).await;
        set_password(
            &server,
            &admin_token,
            &other_realm,
            &viewer_id,
            VIEWER_PASSWORD,
        )
        .await;
        let viewer_role = create_role(
            &server,
            &admin_token,
            &other_realm,
            "tenant-viewer",
            &["view_identity_providers"],
        )
        .await;
        assign_role(
            &server,
            &admin_token,
            &other_realm,
            &viewer_id,
            &viewer_role,
        )
        .await;
        let viewer_token =
            direct_grant(&server, &other_realm, &viewer_username, VIEWER_PASSWORD).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            realm,
            other_realm,
            seeds,
        }
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO identity_providers (id, realm_id, alias, provider_id, enabled, display_name, config, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7::jsonb, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $9))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.alias)
        .bind(seed.provider_id)
        .bind(seed.enabled)
        .bind(seed.display_name)
        .bind(seed.config.to_string())
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert identity provider");
    }

    async fn insert_plain_provider(pool: &PgPool, realm_id: Uuid, alias: &str) {
        sqlx::query(
            "INSERT INTO identity_providers (id, realm_id, alias, provider_id, enabled, display_name, config, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, 'oidc', true, $3, '{}'::jsonb, now(), now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(alias)
        .execute(pool)
        .await
        .expect("insert plain identity provider");
    }

    async fn realm_id_of(pool: &PgPool, name: &str) -> Uuid {
        let id: String = sqlx::query_scalar("SELECT id::text FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("realm id");
        Uuid::parse_str(&id).expect("realm uuid")
    }

    fn make_server() -> TestServer {
        let app = ctx().app.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {token}")
            .parse()
            .expect("valid header value")
    }

    async fn direct_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
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

    async fn create_realm(server: &TestServer, token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": name }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "create realm {name} failed: {}",
            response.text()
        );
    }

    async fn create_user(server: &TestServer, token: &str, realm: &str, username: &str) -> String {
        let response = server
            .post(&format!("/realms/{realm}/users"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "firstname": "Tenant",
                "lastname": "Viewer",
                "email": format!("{username}@test.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "create user {username} in {realm} failed: {}",
            response.text()
        );
        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string()
    }

    async fn set_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = server
            .put(&format!("/realms/{realm}/users/{user_id}/reset-password"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "set password for {user_id} failed: {}",
            response.text()
        );
    }

    async fn create_role(
        server: &TestServer,
        token: &str,
        realm: &str,
        name: &str,
        permissions: &[&str],
    ) -> String {
        let response = server
            .post(&format!("/realms/{realm}/roles"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "name": name,
                "description": "identity providers listing fixture",
                "permissions": permissions,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "create role {name} in {realm} failed: {}",
            response.text()
        );
        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .expect("created role id")
            .to_string()
    }

    async fn assign_role(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        role_id: &str,
    ) {
        let response = server
            .post(&format!("/realms/{realm}/users/{user_id}/roles/{role_id}"))
            .add_header("Authorization", auth_header(token))
            .json(&json!({}))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "assign role {role_id} to {user_id} failed: {}",
            response.text()
        );
    }

    async fn list(server: &TestServer, token: &str, realm: &str, query: &str) -> TestResponse {
        server
            .get(&format!("/realms/{realm}/identity-providers?{query}"))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok(server: &TestServer, realm: &str, query: &str) -> Value {
        let response = list(server, &ctx().admin_token, realm, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    fn rows(body: &Value) -> &Vec<Value> {
        body["data"].as_array().expect("data array")
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        rows(body)
            .iter()
            .map(|provider| {
                Uuid::parse_str(provider["internal_id"].as_str().expect("internal_id"))
                    .expect("uuid")
            })
            .collect()
    }

    fn aliases(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|provider| provider["alias"].as_str().expect("alias").to_string())
            .collect()
    }

    fn total(body: &Value) -> u64 {
        body["metadata"]["total"].as_u64().expect("metadata.total")
    }

    fn expected_order(field: &str, ascending: bool) -> Vec<Uuid> {
        let mut seeds: Vec<&Seed> = ctx().seeds.iter().collect();
        seeds.sort_by(|a, b| match a.key(field).cmp(&b.key(field)) {
            Ordering::Equal => a.id.cmp(&b.id),
            other => other,
        });
        let ordered = seeds.into_iter().map(|seed| seed.id);
        if ascending {
            ordered.collect()
        } else {
            ordered.rev().collect()
        }
    }

    fn matching(predicate: impl Fn(&Seed) -> bool) -> HashSet<Uuid> {
        ctx()
            .seeds
            .iter()
            .filter(|seed| predicate(seed))
            .map(|seed| seed.id)
            .collect()
    }

    fn js_falsy(value: Option<&Value>) -> bool {
        match value {
            None | Some(Value::Null) => true,
            Some(Value::Bool(flag)) => !flag,
            Some(Value::Number(number)) => number.as_f64() == Some(0.0),
            Some(Value::String(text)) => text.is_empty(),
            Some(Value::Array(_)) | Some(Value::Object(_)) => false,
        }
    }

    fn console_health(provider: &Value) -> Health {
        let config = provider["config"].as_object();
        let get = |key: &str| config.and_then(|config| config.get(key));
        let incomplete = [
            "client_id",
            "client_secret",
            "authorization_url",
            "token_url",
        ]
        .into_iter()
        .any(|key| js_falsy(get(key)));
        if incomplete {
            Health::Error
        } else if js_falsy(get("scopes")) {
            Health::Degraded
        } else {
            Health::Healthy
        }
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_providers() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(&server, &ctx().realm, "").await;

            let expected: Vec<Uuid> = expected_order("created_at", false)
                .into_iter()
                .take(20)
                .collect();
            assert_eq!(ids(&body), expected);
            let metadata = &body["metadata"];
            assert_eq!(metadata["page"], 1);
            assert_eq!(metadata["limit"], 20);
            assert_eq!(metadata["total"], 25);
            assert_eq!(metadata["total_pages"], 2);
            assert_eq!(metadata["first_page"], 1);
            assert_eq!(metadata["last_page"], 2);
            assert_eq!(metadata["next_page"], 2);
            assert_eq!(metadata["prev_page"], Value::Null);

            let first = &rows(&body)[0];
            let seed = ctx()
                .seeds
                .iter()
                .find(|seed| seed.id == expected[0])
                .expect("first seed");
            assert_eq!(first["alias"], seed.alias.as_str());
            assert_eq!(first["provider_id"], seed.provider_id);
            assert_eq!(first["enabled"], seed.enabled);
            assert!(first["created_at"].is_string(), "{first}");
            assert!(first["updated_at"].is_string(), "{first}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = list_ok(&server, &ctx().realm, &query).await;
                    let second = list_ok(&server, &ctx().realm, &format!("{query}&page=2")).await;

                    let first_ids = ids(&first);
                    let second_ids = ids(&second);
                    assert_eq!(first_ids.len(), 20, "{query}: page 1 size");
                    assert_eq!(second_ids.len(), 5, "{query}: page 2 size");

                    let first_set: HashSet<Uuid> = first_ids.iter().copied().collect();
                    assert!(
                        second_ids.iter().all(|id| !first_set.contains(id)),
                        "{query}: pages 1 and 2 share a row"
                    );

                    let combined: Vec<Uuid> = first_ids.into_iter().chain(second_ids).collect();
                    assert_eq!(combined, expected_order(field, ascending), "{query}: order");
                    assert_eq!(second["metadata"]["next_page"], Value::Null, "{query}");
                    assert_eq!(second["metadata"]["prev_page"], 1, "{query}");
                }
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_providers() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("alias=IDP-1", matching(|s| s.alias.contains("idp-1"))),
            (
                "display_name=ga",
                matching(|s| {
                    s.display_name
                        .is_some_and(|name| name.to_lowercase().contains("ga"))
                }),
            ),
            (
                "display_name=gamma",
                matching(|s| s.display_name == Some("Gamma SSO")),
            ),
            ("provider_id=oidc", matching(|s| s.provider_id == "oidc")),
            (
                "provider_id=github",
                matching(|s| s.provider_id == "github"),
            ),
            ("enabled=true", matching(|s| s.enabled)),
            ("enabled=false", matching(|s| !s.enabled)),
            (
                "provider_id=saml&enabled=true",
                matching(|s| s.provider_id == "saml" && s.enabled),
            ),
            (
                "alias=idp-1&display_name=beta",
                matching(|s| s.alias.contains("idp-1") && s.display_name == Some("Beta Gate")),
            ),
            ("alias=", matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "alias=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let partial = list_ok(&server, &ctx().realm, "provider_id=oid&limit=100").await;
            assert!(ids(&partial).is_empty(), "provider_id is an exact match");
            assert_eq!(total(&partial), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn health_mirrors_the_console_inference_for_every_branch() {
        let server = make_server();
        rt().block_on(async {
            let covered: HashSet<usize> = (0..SEED_COUNT).map(|i| i % CONFIG_VARIANTS).collect();
            assert_eq!(covered.len(), CONFIG_VARIANTS, "one row per branch");

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            for provider in rows(&everything) {
                let id = Uuid::parse_str(provider["internal_id"].as_str().expect("internal_id"))
                    .expect("uuid");
                let seed = ctx()
                    .seeds
                    .iter()
                    .find(|seed| seed.id == id)
                    .expect("seeded provider");
                assert_eq!(
                    console_health(provider),
                    seed.health,
                    "{}: fixture disagrees with the console",
                    seed.alias
                );
            }

            for health in [Health::Healthy, Health::Degraded, Health::Error] {
                let expected = matching(|s| s.health == health);
                assert!(!expected.is_empty(), "{health:?}");
                let body = list_ok(
                    &server,
                    &ctx().realm,
                    &format!("health={}&limit=100", health.param()),
                )
                .await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{health:?}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{health:?}: total");
                assert!(
                    rows(&body)
                        .iter()
                        .all(|provider| console_health(provider) == health),
                    "{health:?}"
                );
            }

            let narrowed =
                list_ok(&server, &ctx().realm, "health=error&enabled=true&limit=100").await;
            let expected = matching(|s| s.health == Health::Error && s.enabled);
            assert_eq!(ids(&narrowed).into_iter().collect::<HashSet<_>>(), expected);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn client_secrets_stay_masked_in_the_listing() {
        let server = make_server();
        rt().block_on(async {
            let stored = ctx()
                .seeds
                .iter()
                .filter(|seed| {
                    seed.config["client_secret"]
                        .as_str()
                        .is_some_and(|secret| secret.starts_with(SECRET_MARK))
                })
                .count();
            assert!(stored > 0, "the fixture stores client secrets");

            for query in [
                "limit=100",
                "health=healthy&limit=100",
                "order_by=alias&page=2",
            ] {
                let response = list(&server, &ctx().admin_token, &ctx().realm, query).await;
                assert_eq!(response.status_code(), 200, "{query}: {}", response.text());
                let text = response.text();
                assert!(!text.contains(SECRET_MARK), "{query}: {text}");
            }

            let body = list_ok(&server, &ctx().realm, "alias=idp-00").await;
            assert_eq!(aliases(&body), ["idp-00"]);
            assert_eq!(rows(&body)[0]["config"]["client_secret"], "***");
            assert_eq!(rows(&body)[0]["config"]["client_id"], "client-00");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn providers_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "alias=idp-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds idp-5x rows");

            let body = list_ok(&server, &ctx().realm, "alias=idp-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                aliases(&everything)
                    .iter()
                    .all(|alias| alias.starts_with("idp-") && alias.len() == 6)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn a_caller_without_rights_on_the_realm_lists_nothing() {
        let server = make_server();
        rt().block_on(async {
            let own = list(&server, &ctx().viewer_token, &ctx().other_realm, "").await;
            assert_eq!(own.status_code(), 200, "{}", own.text());
            let own: Value = own.json();
            assert!(total(&own) > 0);

            let foreign = list(&server, &ctx().viewer_token, &ctx().realm, "").await;
            assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
            let foreign = foreign.text();
            assert!(!foreign.contains("idp-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "alias=pctx").await;
            assert_eq!(aliases(&witness), ["pctxidp"]);
            let witness = list_ok(&server, &ctx().other_realm, "alias=underx").await;
            assert_eq!(aliases(&witness), ["underxscore"]);
            let witness = list_ok(&server, &ctx().other_realm, "alias=backx").await;
            assert_eq!(aliases(&witness), ["backxslash"]);

            let percent = list_ok(&server, &ctx().other_realm, "alias=%25").await;
            assert_eq!(aliases(&percent), ["pct%idp"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "display_name=%25").await;
            assert_eq!(aliases(&percent), ["pct%idp"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "alias=_").await;
            assert_eq!(aliases(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let backslash = list_ok(&server, &ctx().other_realm, "alias=%5C").await;
            assert_eq!(aliases(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test identity_providers_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=config", "order_by"),
                ("unknown=1", "unknown"),
                ("brief_representation=true", "brief_representation"),
                ("enabled=maybe", "enabled"),
                ("health=broken", "health"),
                ("alias=a&alias=b", "alias"),
            ] {
                let response = list(&server, &ctx().admin_token, &ctx().realm, query).await;
                assert_eq!(response.status_code(), 400, "{query}: {}", response.text());
                assert!(
                    response.text().contains(param),
                    "{query}: {}",
                    response.text()
                );
            }
        });
    }
}
