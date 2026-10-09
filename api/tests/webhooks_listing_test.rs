#[cfg(test)]
mod tests {
    use std::{
        cmp::Ordering,
        collections::{HashMap, HashSet},
        env,
        sync::Arc,
    };

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
    const SORT_FIELDS: [&str; 5] = [
        "name",
        "endpoint",
        "triggered_at",
        "created_at",
        "updated_at",
    ];
    const TRIGGERS: [&str; 3] = ["user.created", "client.deleted", "role.created"];

    struct Seed {
        id: Uuid,
        name: Option<String>,
        endpoint: String,
        subscribers: Vec<&'static str>,
        secret: String,
        header_value: String,
        triggered_minute: Option<i32>,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let scheme = if i % 4 == 3 { "http" } else { "https" };
            Self {
                id: Uuid::new_v4(),
                name: (i % 6 != 5).then(|| format!("hook-{:02}", i / 2)),
                endpoint: format!("{scheme}://e{:02}.example.com/hook", i % 10),
                subscribers: TRIGGERS.iter().take(i % 4).copied().collect(),
                secret: format!("seedsecret{}", Uuid::new_v4().simple()),
                header_value: format!("seedheader{}", Uuid::new_v4().simple()),
                triggered_minute: (!i.is_multiple_of(5))
                    .then(|| i32::try_from(((i * 7) % SEED_COUNT) / 2).expect("small index")),
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "name" => SortKey::Text(self.name.is_none(), self.name.clone().unwrap_or_default()),
                "endpoint" => SortKey::Text(false, self.endpoint.clone()),
                "triggered_at" => SortKey::Minute(
                    self.triggered_minute.is_none(),
                    self.triggered_minute.unwrap_or_default(),
                ),
                "created_at" => SortKey::Minute(false, self.created_minute),
                "updated_at" => SortKey::Minute(false, self.updated_minute),
                other => panic!("no sort key for {other}"),
            }
        }

        fn secure(&self) -> bool {
            self.endpoint.starts_with("https://")
        }

        fn named(&self, needle: &str) -> bool {
            self.name
                .as_deref()
                .is_some_and(|name| name.contains(needle))
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(bool, String),
        Minute(bool, i32),
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

        let schema = format!("webhooks_listing_test_{}", Uuid::new_v4().simple());

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
            client_metadata_allow_private_endpoints: false,
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

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, seed).await;
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
            &["view_webhooks"],
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

        for name in [
            "hook-50",
            "hook-51",
            "hook-52",
            "pct%hook",
            "pctxhook",
            "under_hook",
            "underxhook",
            "back\\hook",
            "backxhook",
        ] {
            insert_plain_webhook(&pool, other_realm_id, name).await;
        }

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            realm,
            other_realm,
            seeds,
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

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO webhooks (id, realm_id, name, endpoint, headers, secret, triggered_at, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, jsonb_build_object('Authorization', $5::text), $6, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $7), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $9))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(seed.name.as_deref())
        .bind(&seed.endpoint)
        .bind(&seed.header_value)
        .bind(&seed.secret)
        .bind(seed.triggered_minute)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert webhook");
        for trigger in &seed.subscribers {
            sqlx::query(
                "INSERT INTO webhook_subscribers (id, name, webhook_id) VALUES ($1::uuid, $2, $3::uuid)",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(*trigger)
            .bind(seed.id.to_string())
            .execute(pool)
            .await
            .expect("insert webhook subscriber");
        }
    }

    async fn insert_plain_webhook(pool: &PgPool, realm_id: Uuid, name: &str) {
        sqlx::query(
            "INSERT INTO webhooks (id, realm_id, name, endpoint, secret, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, 'https://plain.example.com/' || $3, 'plain-secret', now(), now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert plain webhook");
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
                "description": "webhooks listing fixture",
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
            .get(&format!("/realms/{realm}/webhooks?{query}"))
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
            .map(|webhook| {
                Uuid::parse_str(webhook["id"].as_str().expect("webhook id")).expect("uuid")
            })
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|webhook| webhook["name"].as_str().expect("name").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_webhooks() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn nullable_sort_columns_put_nulls_last_ascending_and_first_descending() {
        let server = make_server();
        rt().block_on(async {
            let unnamed = matching(|s| s.name.is_none());
            let never = matching(|s| s.triggered_minute.is_none());
            assert!(unnamed.len() > 1 && never.len() > 1);

            for (field, nulls) in [("name", &unnamed), ("triggered_at", &never)] {
                let ascending = list_ok(
                    &server,
                    &ctx().realm,
                    &format!("order_by={field}&order=asc&limit=100"),
                )
                .await;
                let tail: HashSet<Uuid> = ids(&ascending)
                    .into_iter()
                    .rev()
                    .take(nulls.len())
                    .collect();
                assert_eq!(&tail, nulls, "{field} asc: nulls last");

                let descending = list_ok(
                    &server,
                    &ctx().realm,
                    &format!("order_by={field}&order=desc&limit=100"),
                )
                .await;
                let head: HashSet<Uuid> = ids(&descending).into_iter().take(nulls.len()).collect();
                assert_eq!(&head, nulls, "{field} desc: nulls first");

                let mut null_ids: Vec<Uuid> = nulls.iter().copied().collect();
                null_ids.sort();
                let listed: Vec<Uuid> = ids(&ascending)
                    .into_iter()
                    .filter(|id| nulls.contains(id))
                    .collect();
                assert_eq!(listed, null_ids, "{field} asc: equal nulls ordered by id");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_webhooks() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("name=HOOK-1", matching(|s| s.named("hook-1"))),
            ("name=hook", matching(|s| s.name.is_some())),
            ("endpoint=E03", matching(|s| s.endpoint.contains("e03"))),
            (
                "endpoint=http://",
                matching(|s| s.endpoint.starts_with("http://")),
            ),
            ("triggered=true", matching(|s| s.triggered_minute.is_some())),
            (
                "triggered=false",
                matching(|s| s.triggered_minute.is_none()),
            ),
            (
                "has_subscribers=true",
                matching(|s| !s.subscribers.is_empty()),
            ),
            (
                "has_subscribers=false",
                matching(|s| s.subscribers.is_empty()),
            ),
            ("secure_endpoint=true", matching(|s| s.secure())),
            ("secure_endpoint=false", matching(|s| !s.secure())),
            (
                "has_subscribers=false&triggered=true",
                matching(|s| s.subscribers.is_empty() && s.triggered_minute.is_some()),
            ),
            (
                "name=hook-0&secure_endpoint=true",
                matching(|s| s.named("hook-0") && s.secure()),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn listed_webhooks_carry_their_subscribers() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashMap<Uuid, Vec<String>> = rows(&body)
                .iter()
                .map(|webhook| {
                    let id =
                        Uuid::parse_str(webhook["id"].as_str().expect("webhook id")).expect("uuid");
                    let subscribers = webhook["subscribers"]
                        .as_array()
                        .expect("subscribers array");
                    let mut triggers: Vec<String> = subscribers
                        .iter()
                        .map(|subscriber| {
                            assert_eq!(subscriber["webhook_id"], id.to_string(), "{webhook}");
                            subscriber["name"].as_str().expect("trigger").to_string()
                        })
                        .collect();
                    triggers.sort();
                    (id, triggers)
                })
                .collect();
            assert_eq!(listed.len(), SEED_COUNT);
            assert!(listed.values().any(|triggers| triggers.len() == 3));
            for seed in &ctx().seeds {
                let mut expected: Vec<String> =
                    seed.subscribers.iter().map(|t| t.to_string()).collect();
                expected.sort();
                assert_eq!(listed.get(&seed.id), Some(&expected), "{:?}", seed.name);
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn listed_webhooks_never_disclose_their_secret_or_headers() {
        let server = make_server();
        rt().block_on(async {
            let response = list(&server, &ctx().admin_token, &ctx().realm, "limit=100").await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let text = response.text();
            let body: Value = response.json();
            assert_eq!(total(&body), SEED_COUNT as u64);
            assert!(text.contains(&ctx().seeds[0].endpoint), "{text}");
            for webhook in rows(&body) {
                let object = webhook.as_object().expect("webhook object");
                assert!(!object.contains_key("secret"), "{webhook}");
                assert!(!object.contains_key("headers"), "{webhook}");
            }
            for seed in &ctx().seeds {
                assert!(
                    !text.contains(&seed.secret),
                    "secret of {:?} leaked",
                    seed.name
                );
                assert!(
                    !text.contains(&seed.header_value),
                    "header of {:?} leaked",
                    seed.name
                );
            }
            assert!(!text.contains("seedsecret"));
            assert!(!text.contains("seedheader"));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn webhooks_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=hook-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds hook-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=hook-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let witness = list_ok(&server, &ctx().other_realm, "endpoint=plain.example").await;
            assert_eq!(total(&witness), 9);
            let body = list_ok(&server, &ctx().realm, "endpoint=plain.example").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            let seeded: HashSet<Uuid> = ctx().seeds.iter().map(|seed| seed.id).collect();
            assert_eq!(ids(&everything).into_iter().collect::<HashSet<_>>(), seeded);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn a_caller_without_rights_on_the_realm_lists_nothing() {
        let server = make_server();
        rt().block_on(async {
            let own = list(&server, &ctx().viewer_token, &ctx().other_realm, "").await;
            assert_eq!(own.status_code(), 200, "{}", own.text());
            let own: Value = own.json();
            assert_eq!(total(&own), 9);

            let foreign = list(&server, &ctx().viewer_token, &ctx().realm, "").await;
            assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
            let foreign = foreign.text();
            assert!(!foreign.contains("hook-00"), "{foreign}");
            assert!(!foreign.contains("example.com"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxhook"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxhook"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%hook"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "endpoint=%25").await;
            assert_eq!(names(&percent), ["pct%hook"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_hook"]);
            assert_eq!(total(&underscore), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "endpoint=_").await;
            assert_eq!(names(&underscore), ["under_hook"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxhook"]);
            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\hook"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn search_matches_the_name_or_the_endpoint() {
        let server = make_server();
        let cases: [(&str, HashSet<Uuid>); 5] = [
            ("search=HOOK-1", matching(|s| s.named("hook-1"))),
            ("search=E03", matching(|s| s.endpoint.contains("e03"))),
            ("search=hook-", matching(|s| s.name.is_some())),
            (
                "search=1",
                matching(|s| s.named("1") || s.endpoint.contains('1')),
            ),
            (
                "search=e05&name=hook",
                matching(|s| s.endpoint.contains("e05") && s.name.is_some()),
            ),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{query}: the fixture must make this search discriminating"
                );
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let unnamed = list_ok(&server, &ctx().realm, "search=e05&limit=100").await;
            let found: HashSet<Uuid> = ids(&unnamed).into_iter().collect();
            assert_eq!(found, matching(|s| s.endpoint.contains("e05")));
            assert!(
                ctx()
                    .seeds
                    .iter()
                    .any(|s| s.name.is_none() && found.contains(&s.id)),
                "an unnamed webhook still matches by endpoint"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn search_matches_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            for (query, expected) in [
                ("search=%25", "pct%hook"),
                ("search=_", "under_hook"),
                ("search=%5C", "back\\hook"),
            ] {
                let body = list_ok(&server, &ctx().other_realm, query).await;
                assert_eq!(names(&body), [expected], "{query}");
                assert_eq!(total(&body), 1, "{query}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn created_range_is_inclusive_from_and_exclusive_to() {
        let server = make_server();
        let cases: [(&str, HashSet<Uuid>); 6] = [
            (
                "created_from=2026-01-01T00:05:00Z",
                matching(|s| s.created_minute >= 5),
            ),
            (
                "created_to=2026-01-01T00:02:00Z",
                matching(|s| s.created_minute < 2),
            ),
            (
                "created_from=2026-01-01T00:02:00Z&created_to=2026-01-01T00:05:00Z",
                matching(|s| (2..5).contains(&s.created_minute)),
            ),
            (
                "created_from=2026-01-01T00:03:00Z&created_to=2026-01-01T00:04:00Z",
                matching(|s| s.created_minute == 3),
            ),
            (
                "created_from=2026-01-01T02:05:00%2B02:00",
                matching(|s| s.created_minute >= 5),
            ),
            (
                "created_from=2026-01-01T00:02:00Z&created_to=2026-01-01T00:05:00Z&search=hook-0",
                matching(|s| (2..5).contains(&s.created_minute) && s.named("hook-0")),
            ),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{query}: the fixture must make this range discriminating"
                );
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let edge = list_ok(
                &server,
                &ctx().realm,
                "created_to=2026-01-01T00:03:00Z&limit=100",
            )
            .await;
            let found: HashSet<Uuid> = ids(&edge).into_iter().collect();
            assert!(
                ctx()
                    .seeds
                    .iter()
                    .filter(|s| s.created_minute == 3)
                    .all(|s| !found.contains(&s.id)),
                "a row created exactly at created_to is excluded"
            );
            assert_eq!(found, matching(|s| s.created_minute < 3));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn an_inverted_created_range_is_an_empty_page() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(
                &server,
                &ctx().realm,
                "created_from=2026-01-01T00:05:00Z&created_to=2026-01-01T00:02:00Z",
            )
            .await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhooks_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=secret", "order_by"),
                ("order_by=last_delivery_status", "order_by"),
                ("unknown=1", "unknown"),
                ("secret=x", "secret"),
                ("triggered=maybe", "triggered"),
                ("has_subscribers=maybe", "has_subscribers"),
                ("secure_endpoint=maybe", "secure_endpoint"),
                ("last_delivery_status=failed", "last_delivery_status"),
                ("name=a&name=b", "name"),
                ("search=a&search=b", "search"),
                ("created_from=2026-10-05", "created_from"),
                ("created_to=2026-10-05", "created_to"),
                ("created_from=yesterday", "created_from"),
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
