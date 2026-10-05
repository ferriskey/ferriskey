#[cfg(test)]
mod tests {
    use std::{cmp::Ordering, collections::HashSet, env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
    use chrono::{FixedOffset, NaiveDateTime, SecondsFormat, TimeDelta};
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
        "status",
        "attempt_count",
        "last_attempt_at",
        "created_at",
        "updated_at",
    ];
    const STATUSES: [&str; 4] = ["pending", "delivering", "succeeded", "failed"];
    const EVENTS: [&str; 3] = ["user.created", "client.deleted", "role.created"];
    const SUMMARY_KEYS: [&str; 13] = [
        "id",
        "webhook_id",
        "event",
        "resource_id",
        "status",
        "attempt_count",
        "next_attempt_at",
        "last_attempt_at",
        "last_status_code",
        "last_error_code",
        "last_error_detail",
        "created_at",
        "updated_at",
    ];

    struct Seed {
        id: Uuid,
        event: &'static str,
        status: &'static str,
        resource_id: Uuid,
        attempt_count: i32,
        last_attempt_minute: Option<i32>,
        created_minute: i32,
        updated_minute: i32,
        payload_marker: String,
    }

    impl Seed {
        fn new(i: usize, resources: &[Uuid]) -> Self {
            Self {
                id: Uuid::new_v4(),
                event: EVENTS[i % EVENTS.len()],
                status: STATUSES[i % STATUSES.len()],
                resource_id: resources[i % resources.len()],
                attempt_count: i32::try_from(i % 4).expect("small index"),
                last_attempt_minute: (!i.is_multiple_of(5))
                    .then(|| i32::try_from(((i * 7) % SEED_COUNT) / 2).expect("small index")),
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
                payload_marker: format!("seedpayload{}", Uuid::new_v4().simple()),
            }
        }

        fn failed(&self) -> bool {
            self.status == "failed"
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "status" => SortKey::Text(self.status.to_string()),
                "attempt_count" => SortKey::Number(false, self.attempt_count),
                "last_attempt_at" => SortKey::Number(
                    self.last_attempt_minute.is_none(),
                    self.last_attempt_minute.unwrap_or_default(),
                ),
                "created_at" => SortKey::Number(false, self.created_minute),
                "updated_at" => SortKey::Number(false, self.updated_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(String),
        Number(bool, i32),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        viewer_token: String,
        realm: String,
        other_realm: String,
        webhook: Uuid,
        sibling_webhook: Uuid,
        other_webhook: Uuid,
        resources: Vec<Uuid>,
        seeds: Vec<Seed>,
        seed_base: NaiveDateTime,
        sibling_deliveries: HashSet<Uuid>,
        other_deliveries: HashSet<Uuid>,
        misplaced_delivery: Uuid,
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
            "webhook_deliveries_listing_test_{}",
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
        let realm = format!("deliveries-{}", &suffix[..8]);
        let other_realm = format!("other-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let webhook = insert_webhook(&pool, realm_id, "listed").await;
        let sibling_webhook = insert_webhook(&pool, realm_id, "sibling").await;
        let other_webhook = insert_webhook(&pool, other_realm_id, "foreign").await;

        let resources: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();
        let seeds: Vec<Seed> = (0..SEED_COUNT).map(|i| Seed::new(i, &resources)).collect();
        let seed_base = seed_base(&pool).await;
        for seed in &seeds {
            insert_seed(&pool, realm_id, webhook, &seed_base, seed).await;
        }

        let mut sibling_deliveries = HashSet::new();
        let mut other_deliveries = HashSet::new();
        for _ in 0..3 {
            sibling_deliveries.insert(
                insert_plain_delivery(&pool, realm_id, sibling_webhook, resources[0]).await,
            );
            other_deliveries.insert(
                insert_plain_delivery(&pool, other_realm_id, other_webhook, resources[0]).await,
            );
        }
        let misplaced_delivery =
            insert_plain_delivery(&pool, other_realm_id, webhook, resources[0]).await;

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

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            realm,
            other_realm,
            webhook,
            sibling_webhook,
            other_webhook,
            resources,
            seeds,
            seed_base: NaiveDateTime::parse_from_str(&seed_base, "%Y-%m-%d %H:%M:%S")
                .expect("seed base parses"),
            sibling_deliveries,
            other_deliveries,
            misplaced_delivery,
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

    async fn insert_webhook(pool: &PgPool, realm_id: Uuid, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO webhooks (id, realm_id, name, endpoint, secret, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, 'https://deliveries.example.com/' || $3, 'fixture-secret', now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert webhook");
        id
    }

    async fn seed_base(pool: &PgPool) -> String {
        sqlx::query_scalar(
            "SELECT to_char(date_trunc('minute', now() AT TIME ZONE 'UTC') - interval '1 day', 'YYYY-MM-DD HH24:MI:SS')",
        )
        .fetch_one(pool)
        .await
        .expect("seed base timestamp")
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, webhook_id: Uuid, base: &str, seed: &Seed) {
        let next_attempt = (seed.status == "pending").then_some(seed.created_minute + 30);
        let (status_code, error_code, error_detail) = if seed.failed() {
            (Some(503), Some("http_503"), Some("upstream down"))
        } else {
            (None, None, None)
        };
        sqlx::query(
            "INSERT INTO webhook_deliveries (id, realm_id, webhook_id, event, resource_id, payload, status, attempt_count, \
             next_attempt_at, last_attempt_at, last_status_code, last_error_code, last_error_detail, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5::uuid, jsonb_build_object('marker', $6::text), $7, $8, \
             TIMESTAMP '2099-01-01 00:00:00' + make_interval(mins => $9), \
             $16::timestamp + make_interval(mins => $10), \
             $11, $12, $13, \
             $16::timestamp + make_interval(mins => $14), \
             $16::timestamp + make_interval(mins => $15))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(webhook_id.to_string())
        .bind(seed.event)
        .bind(seed.resource_id.to_string())
        .bind(&seed.payload_marker)
        .bind(seed.status)
        .bind(seed.attempt_count)
        .bind(next_attempt)
        .bind(seed.last_attempt_minute)
        .bind(status_code)
        .bind(error_code)
        .bind(error_detail)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .bind(base)
        .execute(pool)
        .await
        .expect("insert delivery");
    }

    async fn insert_plain_delivery(
        pool: &PgPool,
        realm_id: Uuid,
        webhook_id: Uuid,
        resource_id: Uuid,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO webhook_deliveries (id, realm_id, webhook_id, event, resource_id, payload, status, attempt_count, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'user.created', $4::uuid, '{}'::jsonb, 'failed', 1, now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(webhook_id.to_string())
        .bind(resource_id.to_string())
        .execute(pool)
        .await
        .expect("insert plain delivery");
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
                "description": "webhook deliveries listing fixture",
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

    async fn list(
        server: &TestServer,
        token: &str,
        realm: &str,
        webhook: Uuid,
        query: &str,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{realm}/webhooks/{webhook}/deliveries?{query}"
            ))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok(server: &TestServer, realm: &str, webhook: Uuid, query: &str) -> Value {
        let response = list(server, &ctx().admin_token, realm, webhook, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing {realm}/{webhook} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn list_seeded(server: &TestServer, query: &str) -> Value {
        list_ok(server, &ctx().realm, ctx().webhook, query).await
    }

    fn rows(body: &Value) -> &Vec<Value> {
        body["data"].as_array().expect("data array")
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        rows(body)
            .iter()
            .map(|delivery| {
                Uuid::parse_str(delivery["id"].as_str().expect("delivery id")).expect("uuid")
            })
            .collect()
    }

    fn id_set(body: &Value) -> HashSet<Uuid> {
        ids(body).into_iter().collect()
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

    fn created_at(minute: i64) -> String {
        (ctx().seed_base + TimeDelta::minutes(minute))
            .and_utc()
            .to_rfc3339_opts(SecondsFormat::Secs, true)
    }

    fn created_at_plus_two_hours(minute: i64) -> String {
        let offset = FixedOffset::east_opt(2 * 3600).expect("valid offset");
        (ctx().seed_base + TimeDelta::minutes(minute))
            .and_utc()
            .with_timezone(&offset)
            .to_rfc3339_opts(SecondsFormat::Secs, true)
            .replace('+', "%2B")
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_deliveries() {
        let server = make_server();
        rt().block_on(async {
            let body = list_seeded(&server, "").await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = list_seeded(&server, &query).await;
                    let second = list_seeded(&server, &format!("{query}&page=2")).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn never_attempted_deliveries_sort_last_ascending_and_first_descending() {
        let server = make_server();
        rt().block_on(async {
            let never = matching(|s| s.last_attempt_minute.is_none());
            assert!(never.len() > 1);

            let ascending =
                list_seeded(&server, "order_by=last_attempt_at&order=asc&limit=100").await;
            let tail: HashSet<Uuid> = ids(&ascending)
                .into_iter()
                .rev()
                .take(never.len())
                .collect();
            assert_eq!(tail, never, "asc: nulls last");

            let descending =
                list_seeded(&server, "order_by=last_attempt_at&order=desc&limit=100").await;
            let head: HashSet<Uuid> = ids(&descending).into_iter().take(never.len()).collect();
            assert_eq!(head, never, "desc: nulls first");

            let mut null_ids: Vec<Uuid> = never.iter().copied().collect();
            null_ids.sort();
            let listed: Vec<Uuid> = ids(&ascending)
                .into_iter()
                .filter(|id| never.contains(id))
                .collect();
            assert_eq!(listed, null_ids, "asc: equal nulls ordered by id");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_deliveries() {
        let server = make_server();
        let resource = ctx().resources[2];
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "event=user.created".to_string(),
                matching(|s| s.event == "user.created"),
            ),
            (
                "event=role.created".to_string(),
                matching(|s| s.event == "role.created"),
            ),
            ("status=failed".to_string(), matching(|s| s.failed())),
            (
                "status=pending".to_string(),
                matching(|s| s.status == "pending"),
            ),
            (
                "status=delivering".to_string(),
                matching(|s| s.status == "delivering"),
            ),
            (
                format!("resource_id={resource}"),
                matching(|s| s.resource_id == resource),
            ),
            (
                "event=client.deleted&status=succeeded".to_string(),
                matching(|s| s.event == "client.deleted" && s.status == "succeeded"),
            ),
            (
                format!("status=failed&resource_id={resource}"),
                matching(|s| s.failed() && s.resource_id == resource),
            ),
            ("status=".to_string(), matching(|_| true)),
            ("event=".to_string(), matching(|_| true)),
            ("resource_id=".to_string(), matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query.ends_with('='),
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_seeded(&server, &format!("{query}&limit=100")).await;
                assert_eq!(id_set(&body), expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn search_matches_the_resource_id() {
        let server = make_server();
        let resource = ctx().resources[2];
        let whole = resource.to_string();
        let fragment = whole[..8].to_uppercase();
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                format!("search={whole}"),
                matching(|s| s.resource_id == resource),
            ),
            (
                format!("search={fragment}"),
                matching(|s| s.resource_id.to_string().contains(&whole[..8])),
            ),
            (
                format!("search={fragment}&status=failed"),
                matching(|s| s.resource_id.to_string().contains(&whole[..8]) && s.failed()),
            ),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{query}: the fixture must make this search discriminating"
                );
                let body = list_seeded(&server, &format!("{query}&limit=100")).await;
                assert_eq!(id_set(&body), expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            for query in ["search=zz", "search=%25", "search=_", "search=%5C"] {
                let body = list_seeded(&server, &format!("{query}&limit=100")).await;
                assert!(ids(&body).is_empty(), "{query}: {body}");
                assert_eq!(total(&body), 0, "{query}: total");
            }

            let everything = list_seeded(&server, "search=&limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);

            let shared = ctx().resources[0];
            let body = list_seeded(&server, &format!("search={shared}&limit=100")).await;
            assert_eq!(id_set(&body), matching(|s| s.resource_id == shared));
            assert!(
                id_set(&body).is_disjoint(&ctx().sibling_deliveries),
                "{body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn listed_deliveries_keep_their_summary_shape() {
        let server = make_server();
        rt().block_on(async {
            let response = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                ctx().webhook,
                "status=failed&limit=100",
            )
            .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let text = response.text();
            let body: Value = response.json();
            assert!(!rows(&body).is_empty());

            let expected_keys: HashSet<&str> = SUMMARY_KEYS.into_iter().collect();
            for delivery in rows(&body) {
                let object = delivery.as_object().expect("delivery object");
                let keys: HashSet<&str> = object.keys().map(String::as_str).collect();
                assert_eq!(keys, expected_keys, "{delivery}");
                assert_eq!(delivery["webhook_id"], ctx().webhook.to_string());
                assert_eq!(delivery["status"], "failed");
                assert_eq!(delivery["last_status_code"], 503);
                assert_eq!(delivery["last_error_code"], "http_503");
                assert_eq!(delivery["last_error_detail"], "upstream down");
                assert_eq!(delivery["next_attempt_at"], Value::Null);
            }
            assert!(!text.contains("seedpayload"), "{text}");

            let first = ctx()
                .seeds
                .iter()
                .find(|seed| seed.status == "pending" && seed.last_attempt_minute.is_some())
                .expect("a pending seed already attempted");
            let pending = list_seeded(&server, "status=pending&limit=100").await;
            let row = rows(&pending)
                .iter()
                .find(|row| row["id"] == first.id.to_string())
                .expect("the pending seed is listed");
            assert_eq!(row["event"], first.event);
            assert_eq!(row["resource_id"], first.resource_id.to_string());
            assert_eq!(row["attempt_count"], first.attempt_count);
            assert_eq!(row["last_status_code"], Value::Null);
            assert_eq!(row["last_error_code"], Value::Null);
            assert_eq!(row["last_error_detail"], Value::Null);
            assert!(row["next_attempt_at"].is_string(), "{row}");
            assert!(row["last_attempt_at"].is_string(), "{row}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn deliveries_of_another_webhook_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let resource = ctx().resources[0];
            let witness = list_ok(&server, &ctx().realm, ctx().sibling_webhook, "").await;
            assert_eq!(id_set(&witness), ctx().sibling_deliveries);
            assert_eq!(total(&witness), 3);

            let body = list_seeded(&server, &format!("resource_id={resource}&limit=100")).await;
            let found = id_set(&body);
            assert_eq!(found, matching(|s| s.resource_id == resource));
            assert!(found.is_disjoint(&ctx().sibling_deliveries), "{body}");

            let everything = list_seeded(&server, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert_eq!(id_set(&everything), matching(|_| true));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn deliveries_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, ctx().other_webhook, "").await;
            assert_eq!(id_set(&witness), ctx().other_deliveries);
            assert_eq!(total(&witness), 3);

            let everything = list_seeded(&server, "limit=100").await;
            assert!(
                !id_set(&everything).contains(&ctx().misplaced_delivery),
                "a delivery stamped with another realm leaked: {everything}"
            );
            assert_eq!(total(&everything), SEED_COUNT as u64);

            let foreign_webhook = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                ctx().other_webhook,
                "",
            )
            .await;
            assert_eq!(
                foreign_webhook.status_code(),
                404,
                "{}",
                foreign_webhook.text()
            );
            for id in &ctx().other_deliveries {
                assert!(!foreign_webhook.text().contains(&id.to_string()));
            }

            let crossed = list(
                &server,
                &ctx().admin_token,
                &ctx().other_realm,
                ctx().webhook,
                "",
            )
            .await;
            assert_eq!(crossed.status_code(), 404, "{}", crossed.text());
            assert!(
                !crossed
                    .text()
                    .contains(&ctx().misplaced_delivery.to_string())
            );

            let unknown = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                Uuid::new_v4(),
                "",
            )
            .await;
            assert_eq!(unknown.status_code(), 404, "{}", unknown.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn a_caller_without_rights_on_the_realm_lists_nothing() {
        let server = make_server();
        rt().block_on(async {
            let own = list(
                &server,
                &ctx().viewer_token,
                &ctx().other_realm,
                ctx().other_webhook,
                "",
            )
            .await;
            assert_eq!(own.status_code(), 200, "{}", own.text());
            let own: Value = own.json();
            assert_eq!(total(&own), 3);

            let foreign = list(
                &server,
                &ctx().viewer_token,
                &ctx().realm,
                ctx().webhook,
                "",
            )
            .await;
            assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
            let foreign = foreign.text();
            for seed in &ctx().seeds {
                assert!(!foreign.contains(&seed.id.to_string()), "{foreign}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn like_wildcards_are_never_patterns() {
        let server = make_server();
        rt().block_on(async {
            for query in [
                "event=%25",
                "status=%25",
                "resource_id=%25",
                "event=user.%25",
            ] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().webhook,
                    query,
                )
                .await;
                assert_eq!(response.status_code(), 400, "{query}: {}", response.text());
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn created_range_is_inclusive_from_and_exclusive_to() {
        let server = make_server();
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                format!("created_from={}", created_at(5)),
                matching(|s| s.created_minute >= 5),
            ),
            (
                format!("created_to={}", created_at(2)),
                matching(|s| s.created_minute < 2),
            ),
            (
                format!(
                    "created_from={}&created_to={}",
                    created_at(2),
                    created_at(5)
                ),
                matching(|s| (2..5).contains(&s.created_minute)),
            ),
            (
                format!(
                    "created_from={}&created_to={}",
                    created_at(3),
                    created_at(4)
                ),
                matching(|s| s.created_minute == 3),
            ),
            (
                format!("created_from={}", created_at_plus_two_hours(5)),
                matching(|s| s.created_minute >= 5),
            ),
            (
                format!(
                    "created_from={}&created_to={}&status=failed",
                    created_at(2),
                    created_at(8)
                ),
                matching(|s| (2..8).contains(&s.created_minute) && s.failed()),
            ),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{query}: the fixture must make this range discriminating"
                );
                let body = list_seeded(&server, &format!("{query}&limit=100")).await;
                assert_eq!(id_set(&body), expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let edge =
                list_seeded(&server, &format!("created_to={}&limit=100", created_at(3))).await;
            let found = id_set(&edge);
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn an_inverted_created_range_is_an_empty_page() {
        let server = make_server();
        rt().block_on(async {
            let body = list_seeded(
                &server,
                &format!(
                    "created_from={}&created_to={}",
                    created_at(5),
                    created_at(2)
                ),
            )
            .await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test webhook_deliveries_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=201", "limit"),
                ("limit=0", "limit"),
                ("order_by=payload", "order_by"),
                ("order_by=next_attempt_at", "order_by"),
                ("order_by=event", "order_by"),
                ("unknown=1", "unknown"),
                ("offset=20", "offset"),
                ("payload=x", "payload"),
                ("status=exploded", "status"),
                ("status=FAILED", "status"),
                ("event=user.exploded", "event"),
                ("resource_id=nope", "resource_id"),
                ("status=failed&status=pending", "status"),
                ("created_from=2026-10-05", "created_from"),
                ("created_to=2026-10-05", "created_to"),
                ("created_from=yesterday", "created_from"),
                ("search=a&search=b", "search"),
            ] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().webhook,
                    query,
                )
                .await;
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
