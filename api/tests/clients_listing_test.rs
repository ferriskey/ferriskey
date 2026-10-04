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
    const SECRET_PREFIX: &str = "seed-secret-";
    const SEED_COUNT: usize = 25;
    const CLIENT_TYPES: [&str; 3] = ["confidential", "public", "system"];
    const SORT_FIELDS: [&str; 5] = ["name", "client_id", "enabled", "created_at", "updated_at"];

    struct Seed {
        id: Uuid,
        name: String,
        client_id: String,
        enabled: bool,
        public_client: bool,
        service_account_enabled: bool,
        protocol: &'static str,
        client_type: &'static str,
        redirect: bool,
        maintenance: Option<bool>,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: format!("client-{:02}", i - i % 2),
                client_id: format!("app-{:02}", (i * 7) % SEED_COUNT),
                enabled: i % 4 != 3,
                public_client: i % 3 == 1,
                service_account_enabled: i.is_multiple_of(5),
                protocol: if i.is_multiple_of(6) {
                    "saml"
                } else {
                    "openid-connect"
                },
                client_type: CLIENT_TYPES[(i / 2) % CLIENT_TYPES.len()],
                redirect: i.is_multiple_of(4),
                maintenance: match i % 7 {
                    3 => Some(true),
                    4 => None,
                    _ => Some(false),
                },
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "name" => SortKey::Text(self.name.clone()),
                "client_id" => SortKey::Text(self.client_id.clone()),
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
        Flag(bool),
        Minute(i32),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        viewer_token: String,
        realm: String,
        other_realm: String,
        apps_realm: String,
        seeds: Vec<Seed>,
        foreign_client_id: Uuid,
    }

    struct App {
        name: &'static str,
        service_account: bool,
        device_grant: Option<bool>,
        client_type: &'static str,
        public_client: bool,
        redirect: bool,
        expected: &'static str,
    }

    const APPS: [App; 10] = [
        App {
            name: "m2m-a",
            service_account: true,
            device_grant: Some(true),
            client_type: "confidential",
            public_client: false,
            redirect: false,
            expected: "m2m",
        },
        App {
            name: "m2m-b",
            service_account: true,
            device_grant: None,
            client_type: "public",
            public_client: true,
            redirect: true,
            expected: "m2m",
        },
        App {
            name: "device-a",
            service_account: false,
            device_grant: Some(true),
            client_type: "public",
            public_client: true,
            redirect: false,
            expected: "device",
        },
        App {
            name: "device-b",
            service_account: false,
            device_grant: Some(true),
            client_type: "confidential",
            public_client: false,
            redirect: false,
            expected: "device",
        },
        App {
            name: "spa-a",
            service_account: false,
            device_grant: Some(true),
            client_type: "public",
            public_client: true,
            redirect: true,
            expected: "spa",
        },
        App {
            name: "spa-b",
            service_account: false,
            device_grant: None,
            client_type: "public",
            public_client: true,
            redirect: false,
            expected: "spa",
        },
        App {
            name: "native-a",
            service_account: false,
            device_grant: Some(false),
            client_type: "public",
            public_client: false,
            redirect: true,
            expected: "native",
        },
        App {
            name: "web-a",
            service_account: false,
            device_grant: Some(false),
            client_type: "confidential",
            public_client: false,
            redirect: true,
            expected: "web",
        },
        App {
            name: "web-b",
            service_account: false,
            device_grant: None,
            client_type: "system",
            public_client: false,
            redirect: false,
            expected: "web",
        },
        App {
            name: "web-c",
            service_account: false,
            device_grant: Some(true),
            client_type: "confidential",
            public_client: true,
            redirect: true,
            expected: "web",
        },
    ];

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

        let schema = format!("clients_listing_test_{}", Uuid::new_v4().simple());

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

        let apps_realm = format!("apps-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &apps_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;
        let apps_realm_id = realm_id_of(&pool, &apps_realm).await;

        for cleared in [realm_id, apps_realm_id] {
            sqlx::query("DELETE FROM clients WHERE realm_id = $1::uuid")
                .bind(cleared.to_string())
                .execute(&pool)
                .await
                .expect("clear the default clients of a seeded realm");
        }
        for app in &APPS {
            insert_app(&pool, apps_realm_id, app).await;
        }

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for (i, seed) in seeds.iter().enumerate() {
            insert_seed(&pool, realm_id, seed, i).await;
        }

        let foreign_client_id =
            insert_plain_client(&pool, other_realm_id, "client-50", "foreign-50").await;
        for (name, client_id) in [
            ("client-51", "foreign-51"),
            ("client-52", "foreign-52"),
            ("pct%client", "pct%id"),
            ("pctxclient", "pctxid"),
            ("under_client", "under_id"),
            ("underxclient", "underxid"),
            ("back\\client", "back\\id"),
            ("backxclient", "backxid"),
        ] {
            insert_plain_client(&pool, other_realm_id, name, client_id).await;
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
            &["view_clients"],
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
            apps_realm,
            seeds,
            foreign_client_id,
        }
    }

    async fn insert_app(pool: &PgPool, realm_id: Uuid, app: &App) {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, oauth_device_code_grant_enabled, client_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $3, true, 'openid-connect', $4, $5, $6, $7, now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(app.name)
        .bind(app.public_client)
        .bind(app.service_account)
        .bind(app.device_grant)
        .bind(app.client_type)
        .execute(pool)
        .await
        .expect("insert application");

        if app.redirect {
            sqlx::query(
                "INSERT INTO redirect_uris (id, client_id, value, enabled, created_at, updated_at) \
                 VALUES ($1::uuid, $2::uuid, 'https://app.test/callback', true, now(), now())",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(id.to_string())
            .execute(pool)
            .await
            .expect("insert application redirect uri");
        }
    }

    async fn realm_id_of(pool: &PgPool, name: &str) -> Uuid {
        let id: String = sqlx::query_scalar("SELECT id::text FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("realm id");
        Uuid::parse_str(&id).expect("realm uuid")
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed, index: usize) {
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, secret, enabled, protocol, public_client, service_account_enabled, client_type, maintenance_enabled, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7, $8, $9, $10, $11, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $12), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $13))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.name)
        .bind(&seed.client_id)
        .bind(format!("{SECRET_PREFIX}{index}"))
        .bind(seed.enabled)
        .bind(seed.protocol)
        .bind(seed.public_client)
        .bind(seed.service_account_enabled)
        .bind(seed.client_type)
        .bind(seed.maintenance)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert client");

        if seed.redirect {
            sqlx::query(
                "INSERT INTO redirect_uris (id, client_id, value, enabled, created_at, updated_at) \
                 VALUES ($1::uuid, $2::uuid, 'https://app.test/callback', true, now(), now())",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(seed.id.to_string())
            .execute(pool)
            .await
            .expect("insert redirect uri");
        }
    }

    async fn insert_plain_client(
        pool: &PgPool,
        realm_id: Uuid,
        name: &str,
        client_id: &str,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, true, 'openid-connect', false, false, 'confidential', now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .bind(client_id)
        .execute(pool)
        .await
        .expect("insert plain client");
        id
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
                "description": "clients listing fixture",
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
            .get(&format!("/realms/{realm}/clients?{query}"))
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
            .map(|client| Uuid::parse_str(client["id"].as_str().expect("client id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|client| client["name"].as_str().expect("name").to_string())
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

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_clients() {
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
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_clients() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("name=CLIENT-1", matching(|s| s.name.contains("client-1"))),
            (
                "client_id=APP-0",
                matching(|s| s.client_id.contains("app-0")),
            ),
            ("enabled=true", matching(|s| s.enabled)),
            ("enabled=false", matching(|s| !s.enabled)),
            ("public_client=true", matching(|s| s.public_client)),
            ("public_client=false", matching(|s| !s.public_client)),
            (
                "service_account_enabled=true",
                matching(|s| s.service_account_enabled),
            ),
            (
                "service_account_enabled=false",
                matching(|s| !s.service_account_enabled),
            ),
            ("protocol=saml", matching(|s| s.protocol == "saml")),
            (
                "protocol=openid-connect",
                matching(|s| s.protocol == "openid-connect"),
            ),
            (
                "client_type=confidential",
                matching(|s| s.client_type == "confidential"),
            ),
            (
                "client_type=public",
                matching(|s| s.client_type == "public"),
            ),
            (
                "client_type=system",
                matching(|s| s.client_type == "system"),
            ),
            ("has_redirect_uris=true", matching(|s| s.redirect)),
            ("has_redirect_uris=false", matching(|s| !s.redirect)),
            (
                "maintenance_enabled=true",
                matching(|s| s.maintenance == Some(true)),
            ),
            (
                "maintenance_enabled=false",
                matching(|s| s.maintenance != Some(true)),
            ),
            (
                "name=client-1&enabled=true&public_client=false",
                matching(|s| s.name.contains("client-1") && s.enabled && !s.public_client),
            ),
            ("search=CLIENT-1", matching(|s| s.name.contains("client-1"))),
            ("search=APP-0", matching(|s| s.client_id.contains("app-0"))),
            (
                "search=client-1&enabled=true",
                matching(|s| s.name.contains("client-1") && s.enabled),
            ),
            ("name=", matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "name=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn the_listing_never_reveals_client_secrets() {
        let server = make_server();
        rt().block_on(async {
            let response = list(&server, &ctx().admin_token, &ctx().realm, "limit=100").await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let body: Value = response.json();
            assert_eq!(total(&body), SEED_COUNT as u64);
            assert!(
                !response.text().contains(SECRET_PREFIX),
                "{}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn clients_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=client-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds client-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=client-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let witness = list_ok(&server, &ctx().other_realm, "client_id=foreign").await;
            assert_eq!(total(&witness), 3, "the other realm holds foreign-5x rows");
            let body = list_ok(&server, &ctx().realm, "client_id=foreign").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                names(&everything)
                    .iter()
                    .all(|name| name.starts_with("client-") && name.len() == 9)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
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
            assert!(!foreign.contains("client-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxclient"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxclient"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%client"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "client_id=%25").await;
            assert_eq!(names(&percent), ["pct%client"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_client"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "search=pctx").await;
            assert_eq!(names(&witness), ["pctxclient"]);
            let percent = list_ok(&server, &ctx().other_realm, "search=%25").await;
            assert_eq!(names(&percent), ["pct%client"]);
            assert_eq!(total(&percent), 1);
            let through_client_id = list_ok(&server, &ctx().other_realm, "search=pct%25id").await;
            assert_eq!(names(&through_client_id), ["pct%client"]);
            assert_eq!(total(&through_client_id), 1);

            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxclient"]);
            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\client"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn ids_select_exactly_the_given_clients_of_the_realm() {
        let server = make_server();
        rt().block_on(async {
            let seeds = &ctx().seeds;
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            let wanted = [seeds[1].id, seeds[3].id, seeds[22].id];
            assert!(wanted.iter().all(|id| listed.contains(id)));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign = ctx().foreign_client_id;
            let foreign_witness =
                list_ok(&server, &ctx().other_realm, &format!("ids={foreign}")).await;
            assert_eq!(ids(&foreign_witness), [foreign]);
            let joined = wanted
                .iter()
                .chain([&foreign])
                .map(Uuid::to_string)
                .collect::<Vec<_>>()
                .join(",");

            let body = list_ok(&server, &ctx().realm, &format!("ids={joined}")).await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            assert_eq!(found, wanted.into_iter().collect::<HashSet<_>>());
            assert_eq!(total(&body), 3);

            let narrowed =
                list_ok(&server, &ctx().realm, &format!("ids={joined}&enabled=true")).await;
            let expected: HashSet<Uuid> = [&seeds[1], &seeds[3], &seeds[22]]
                .into_iter()
                .filter(|seed| seed.enabled)
                .map(|seed| seed.id)
                .collect();
            assert!(!expected.is_empty() && expected.len() < 3);
            assert_eq!(ids(&narrowed).into_iter().collect::<HashSet<_>>(), expected);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn more_than_a_hundred_ids_are_refused() {
        let server = make_server();
        rt().block_on(async {
            let hundred = (0..100)
                .map(|_| Uuid::new_v4().to_string())
                .collect::<Vec<_>>();
            let accepted = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                &format!("ids={}", hundred.join(",")),
            )
            .await;
            assert_eq!(accepted.status_code(), 200, "{}", accepted.text());

            let too_many = (0..101)
                .map(|_| Uuid::new_v4().to_string())
                .collect::<Vec<_>>();
            let response = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                &format!("ids={}", too_many.join(",")),
            )
            .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert!(response.text().contains("ids"), "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=secret", "order_by"),
                ("unknown=1", "unknown"),
                ("enabled=maybe", "enabled"),
                ("public_client=maybe", "public_client"),
                ("protocol=oauth", "protocol"),
                ("client_type=robot", "client_type"),
                ("application_type=robot", "application_type"),
                (
                    "oauth_device_code_grant_enabled=maybe",
                    "oauth_device_code_grant_enabled",
                ),
                ("has_redirect_uris=maybe", "has_redirect_uris"),
                ("ids=not-a-uuid", "ids"),
                ("name=a&name=b", "name"),
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

    fn app_names(predicate: impl Fn(&App) -> bool) -> Vec<String> {
        let mut names: Vec<String> = APPS
            .iter()
            .filter(|app| predicate(app))
            .map(|app| app.name.to_string())
            .collect();
        names.sort();
        names
    }

    async fn sorted_names(server: &TestServer, query: &str) -> (Vec<String>, u64) {
        let body = list_ok(server, &ctx().apps_realm, &format!("{query}&limit=100")).await;
        let mut found = names(&body);
        found.sort();
        (found, total(&body))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn every_application_type_returns_exactly_its_clients() {
        let server = make_server();
        rt().block_on(async {
            let (everything, all) = sorted_names(&server, "").await;
            assert_eq!(everything, app_names(|_| true));
            assert_eq!(all, APPS.len() as u64);

            for kind in ["m2m", "device", "spa", "native", "web"] {
                let expected = app_names(|app| app.expected == kind);
                assert!(!expected.is_empty(), "{kind}: the fixture covers this type");
                let (found, count) =
                    sorted_names(&server, &format!("application_type={kind}")).await;
                assert_eq!(found, expected, "{kind}: rows");
                assert_eq!(count, expected.len() as u64, "{kind}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn listed_clients_carry_their_redirect_uris() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(&server, &ctx().apps_realm, "limit=100").await;
            assert_eq!(rows(&body).len(), APPS.len());
            for app in &APPS {
                let row = rows(&body)
                    .iter()
                    .find(|row| row["name"] == app.name)
                    .unwrap_or_else(|| panic!("{} is listed", app.name));
                let uris = row["redirect_uris"]
                    .as_array()
                    .unwrap_or_else(|| panic!("{}: redirect_uris is an array: {row}", app.name));
                let values: Vec<&str> = uris
                    .iter()
                    .map(|uri| uri["value"].as_str().expect("redirect uri value"))
                    .collect();
                let expected: Vec<&str> = if app.redirect {
                    vec!["https://app.test/callback"]
                } else {
                    Vec::new()
                };
                assert_eq!(values, expected, "{}", app.name);

                let id = row["id"].as_str().expect("client id");
                let single = server
                    .get(&format!("/realms/{}/clients/{id}", ctx().apps_realm))
                    .add_header("Authorization", auth_header(&ctx().admin_token))
                    .await;
                assert_eq!(single.status_code(), 200, "{}", single.text());
                let single: Value = single.json();
                assert_eq!(
                    single["data"]["redirect_uris"], row["redirect_uris"],
                    "{}: the listing matches the single fetch",
                    app.name
                );
            }

            let spa = list_ok(
                &server,
                &ctx().apps_realm,
                "application_type=spa&name=spa-a",
            )
            .await;
            assert_eq!(names(&spa), ["spa-a"]);
            assert_eq!(
                rows(&spa)[0]["redirect_uris"][0]["value"],
                "https://app.test/callback"
            );
            assert_eq!(rows(&spa)[0]["oauth_device_code_grant_enabled"], true);

            let seeded = list_ok(&server, &ctx().realm, "limit=100").await;
            for row in rows(&seeded) {
                let id = Uuid::parse_str(row["id"].as_str().expect("id")).expect("uuid");
                let seed = ctx()
                    .seeds
                    .iter()
                    .find(|seed| seed.id == id)
                    .expect("seeded client");
                let count = row["redirect_uris"].as_array().map(Vec::len);
                assert_eq!(count, Some(usize::from(seed.redirect)), "{row}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test clients_listing_test -- --ignored"]
    fn the_device_grant_filter_treats_unset_as_disabled() {
        let server = make_server();
        rt().block_on(async {
            let (on, _) = sorted_names(&server, "oauth_device_code_grant_enabled=true").await;
            assert_eq!(on, app_names(|app| app.device_grant == Some(true)));
            let (off, _) = sorted_names(&server, "oauth_device_code_grant_enabled=false").await;
            assert_eq!(off, app_names(|app| app.device_grant != Some(true)));

            let (missing_callback, count) = sorted_names(
                &server,
                "service_account_enabled=false&oauth_device_code_grant_enabled=false&has_redirect_uris=false",
            )
            .await;
            let expected = app_names(|app| {
                !app.service_account && app.device_grant != Some(true) && !app.redirect
            });
            assert!(!expected.is_empty());
            assert_eq!(missing_callback, expected);
            assert_eq!(count, expected.len() as u64);
        });
    }
}
