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
    const PASSWORD: &str = "C0mpass-Listing-Pw!";
    const SEED_COUNT: usize = 25;
    const CLIENTS: [&str; 3] = ["web-app", "mobile-app", "admin-cli"];
    const GRANTS: [&str; 4] = [
        "authorization_code",
        "password",
        "refresh_token",
        "client_credentials",
    ];
    const STATUSES: [&str; 4] = ["pending", "success", "failure", "expired"];
    const SORT_FIELDS: [&str; 4] = ["status", "started_at", "duration_ms", "created_at"];

    struct Seed {
        id: Uuid,
        client_id: Option<&'static str>,
        user: Option<usize>,
        grant_type: &'static str,
        status: &'static str,
        ip_address: Option<String>,
        started_minute: i32,
        duration_ms: Option<i64>,
        created_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            Self {
                id: Uuid::new_v4(),
                client_id: if i % 8 == 7 {
                    None
                } else {
                    Some(CLIENTS[i % CLIENTS.len()])
                },
                user: match i % 4 {
                    0 => Some(0),
                    1 => Some(1),
                    _ => None,
                },
                grant_type: GRANTS[(i / 2) % GRANTS.len()],
                status: STATUSES[(i / 3) % STATUSES.len()],
                ip_address: if i % 6 == 5 {
                    None
                } else {
                    Some(format!("192.168.{}.{i}", i % 3))
                },
                started_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
                duration_ms: if i % 5 == 2 {
                    None
                } else {
                    Some(i64::try_from(((i * 13) % SEED_COUNT) / 3).expect("small index") * 100)
                },
                created_minute: i32::try_from(((i * 7) % SEED_COUNT) / 3).expect("small index"),
            }
        }

        fn completed(&self) -> bool {
            matches!(self.status, "success" | "failure")
        }

        fn step_count(&self) -> usize {
            if self.status == "failure" { 2 } else { 1 }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "status" => SortKey::Text(self.status),
                "started_at" => SortKey::Number(i64::from(self.started_minute)),
                "duration_ms" => SortKey::Nullable(
                    self.duration_ms.is_none(),
                    self.duration_ms.unwrap_or_default(),
                ),
                "created_at" => SortKey::Number(i64::from(self.created_minute)),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(&'static str),
        Number(i64),
        Nullable(bool, i64),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: PgPool,
        admin_token: String,
        viewer_token: String,
        foreign_viewer_token: String,
        no_rights_token: String,
        realm: String,
        other_realm: String,
        realm_id: Uuid,
        other_realm_id: Uuid,
        users: [Uuid; 2],
        foreign_user: Uuid,
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

        let schema = format!("compass_flows_listing_test_{}", Uuid::new_v4().simple());

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
        let short = &suffix[..8];
        let realm = format!("compass-{short}");
        let other_realm = format!("other-{short}");
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;
        for id in [realm_id, other_realm_id] {
            sqlx::query(
                "UPDATE realm_settings SET compass_enabled = false WHERE realm_id = $1::uuid",
            )
            .bind(id.to_string())
            .execute(&pool)
            .await
            .expect("disable compass tracing for the fixture realms");
        }

        let alice = uuid_of(&create_user(&server, &admin_token, &realm, "alice").await);
        let bob = uuid_of(&create_user(&server, &admin_token, &realm, "bob").await);
        let foreign_user =
            uuid_of(&create_user(&server, &admin_token, &other_realm, "foreign").await);
        let users = [alice, bob];

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, &users, seed).await;
        }

        for (client_id, ip) in [
            ("web-app", "192.168.1.1"),
            ("web-app", "192.168.1.2"),
            ("foreign-app", "ip%literal"),
            ("foreign-app", "ipxliteral"),
            ("foreign-app", "under_ip"),
            ("foreign-app", "underxip"),
            ("foreign-app", "back\\ip"),
            ("foreign-app", "backxip"),
        ] {
            insert_plain_flow(&pool, other_realm_id, client_id, Some(foreign_user), ip).await;
        }

        let viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("viewer-{short}"),
            "flow-viewer",
            &["view_events"],
        )
        .await;
        let foreign_viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &other_realm,
            &format!("foreign-viewer-{short}"),
            "foreign-flow-viewer",
            &["view_events"],
        )
        .await;
        let no_rights_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("no-rights-{short}"),
            "realm-viewer",
            &["view_realm"],
        )
        .await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            admin_token,
            viewer_token,
            foreign_viewer_token,
            no_rights_token,
            realm,
            other_realm,
            realm_id,
            other_realm_id,
            users,
            foreign_user,
            seeds,
        }
    }

    fn uuid_of(id: &str) -> Uuid {
        Uuid::parse_str(id).expect("user uuid")
    }

    async fn user_with_permissions(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        username: &str,
        role_name: &str,
        permissions: &[&str],
    ) -> String {
        let user_id = create_user(server, admin_token, realm, username).await;
        set_password(server, admin_token, realm, &user_id, PASSWORD).await;
        let role_id = create_role(server, admin_token, realm, role_name, permissions).await;
        assign_role(server, admin_token, realm, &user_id, &role_id).await;
        direct_grant(server, realm, username, PASSWORD).await
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, users: &[Uuid; 2], seed: &Seed) {
        sqlx::query(
            "INSERT INTO compass_flows (id, realm_id, client_id, user_id, grant_type, status, ip_address, user_agent, started_at, completed_at, duration_ms, created_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4::uuid, $5, $6, $7, 'seed-agent', \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8), \
             CASE WHEN $11 THEN TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8, secs => 30) END, $9, \
             TIMESTAMP '2026-02-01 00:00:00' + make_interval(mins => $10))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(seed.client_id)
        .bind(seed.user.map(|index| users[index].to_string()))
        .bind(seed.grant_type)
        .bind(seed.status)
        .bind(seed.ip_address.as_deref())
        .bind(seed.started_minute)
        .bind(seed.duration_ms)
        .bind(seed.created_minute)
        .bind(seed.completed())
        .execute(pool)
        .await
        .expect("insert compass flow");

        insert_step(pool, seed.id, "authorize", "success", 0).await;
        if seed.status == "failure" {
            insert_step(pool, seed.id, "credential_validation", "failure", 1).await;
        }
    }

    async fn insert_step(pool: &PgPool, flow_id: Uuid, name: &str, status: &str, second: i32) {
        sqlx::query(
            "INSERT INTO compass_flow_steps (id, flow_id, step_name, status, started_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, TIMESTAMP '2026-01-01 00:00:00' + make_interval(secs => $5))",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(flow_id.to_string())
        .bind(name)
        .bind(status)
        .bind(f64::from(second))
        .execute(pool)
        .await
        .expect("insert compass flow step");
    }

    async fn insert_plain_flow(
        pool: &PgPool,
        realm_id: Uuid,
        client_id: &str,
        user_id: Option<Uuid>,
        ip_address: &str,
    ) {
        sqlx::query(
            "INSERT INTO compass_flows (id, realm_id, client_id, user_id, grant_type, status, ip_address, started_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4::uuid, 'password', 'failure', $5, now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(client_id)
        .bind(user_id.map(|id| id.to_string()))
        .bind(ip_address)
        .execute(pool)
        .await
        .expect("insert plain compass flow");
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
                "firstname": "Flow",
                "lastname": "Owner",
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
                "description": "compass flows listing fixture",
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
            .get(&format!("/realms/{realm}/compass/v1/flows?{query}"))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok_as(server: &TestServer, token: &str, realm: &str, query: &str) -> Value {
        let response = list(server, token, realm, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing the flows of {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn list_realm(server: &TestServer, query: &str) -> Value {
        list_ok_as(server, &ctx().admin_token, &ctx().realm, query).await
    }

    async fn list_other(server: &TestServer, query: &str) -> Value {
        list_ok_as(server, &ctx().admin_token, &ctx().other_realm, query).await
    }

    fn rows(body: &Value) -> &Vec<Value> {
        body["data"].as_array().expect("data array")
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        rows(body)
            .iter()
            .map(|flow| Uuid::parse_str(flow["id"].as_str().expect("id")).expect("uuid"))
            .collect()
    }

    fn ips(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|flow| flow["ip_address"].as_str().expect("ip_address").to_string())
            .collect()
    }

    fn total(body: &Value) -> u64 {
        body["metadata"]["total"].as_u64().expect("metadata.total")
    }

    fn seed_of(id: Uuid) -> &'static Seed {
        ctx()
            .seeds
            .iter()
            .find(|seed| seed.id == id)
            .expect("seeded flow")
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

    fn ip_contains(seed: &Seed, needle: &str) -> bool {
        seed.ip_address
            .as_deref()
            .is_some_and(|ip| ip.contains(needle))
    }

    fn user_is(seed: &Seed, index: usize) -> bool {
        seed.user == Some(index)
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_flows() {
        let server = make_server();
        rt().block_on(async {
            let stored: i64 =
                sqlx::query_scalar("SELECT count(*) FROM compass_flows WHERE realm_id = $1::uuid")
                    .bind(ctx().realm_id.to_string())
                    .fetch_one(&ctx().pool)
                    .await
                    .expect("count stored flows");
            assert_eq!(
                stored, SEED_COUNT as i64,
                "only the seeded flows are stored"
            );

            let body = list_realm(&server, "").await;

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
            let seed = seed_of(expected[0]);
            assert_eq!(first["client_id"].as_str(), seed.client_id);
            assert_eq!(first["grant_type"], seed.grant_type);
            assert_eq!(first["status"], seed.status);
            assert_eq!(first["ip_address"].as_str(), seed.ip_address.as_deref());
            assert_eq!(first["duration_ms"].as_i64(), seed.duration_ms);
            assert!(first["started_at"].is_string(), "{first}");
            assert_eq!(
                first["completed_at"].is_string(),
                seed.completed(),
                "{first}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn every_listed_flow_carries_its_steps() {
        let server = make_server();
        rt().block_on(async {
            let body = list_realm(&server, "limit=100").await;
            assert_eq!(rows(&body).len(), SEED_COUNT);
            for flow in rows(&body) {
                let id = Uuid::parse_str(flow["id"].as_str().expect("id")).expect("uuid");
                let seed = seed_of(id);
                let steps = flow["steps"].as_array().expect("steps array");
                assert_eq!(steps.len(), seed.step_count(), "{flow}");
                assert!(
                    steps.iter().all(|step| step["flow_id"] == id.to_string()),
                    "{flow}"
                );
                if seed.status == "failure" {
                    assert!(
                        steps.iter().any(|step| step["status"] == "failure"
                            && step["step_name"] == "credential_validation"),
                        "{flow}"
                    );
                }
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = list_realm(&server, &query).await;
                    let second = list_realm(&server, &format!("{query}&page=2")).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn unfinished_flows_sort_last_by_duration_ascending_and_first_descending() {
        let server = make_server();
        rt().block_on(async {
            let unmeasured = matching(|s| s.duration_ms.is_none());
            assert!(
                !unmeasured.is_empty() && unmeasured.len() < SEED_COUNT,
                "the fixture mixes measured and unmeasured rows"
            );

            let ascending = list_realm(&server, "order_by=duration_ms&order=asc&limit=100").await;
            let ascending = ids(&ascending);
            let (_, tail) = ascending.split_at(SEED_COUNT - unmeasured.len());
            assert_eq!(
                tail.iter().copied().collect::<HashSet<_>>(),
                unmeasured,
                "asc: nulls last"
            );

            let descending = list_realm(&server, "order_by=duration_ms&order=desc&limit=100").await;
            let descending = ids(&descending);
            let (head, _) = descending.split_at(unmeasured.len());
            assert_eq!(
                head.iter().copied().collect::<HashSet<_>>(),
                unmeasured,
                "desc: nulls first"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_flows() {
        let server = make_server();
        let alice = ctx().users[0];
        let bob = ctx().users[1];
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "ip_address=192.168.1.".to_string(),
                matching(|s| ip_contains(s, "192.168.1.")),
            ),
            (
                "ip_address=.2".to_string(),
                matching(|s| ip_contains(s, ".2")),
            ),
            (
                "grant_type=password".to_string(),
                matching(|s| s.grant_type == "password"),
            ),
            (
                "grant_type=refresh_token".to_string(),
                matching(|s| s.grant_type == "refresh_token"),
            ),
            (
                "status=failure".to_string(),
                matching(|s| s.status == "failure"),
            ),
            (
                "status=expired".to_string(),
                matching(|s| s.status == "expired"),
            ),
            (
                "client_id=web-app".to_string(),
                matching(|s| s.client_id == Some("web-app")),
            ),
            (
                "client_id=admin-cli".to_string(),
                matching(|s| s.client_id == Some("admin-cli")),
            ),
            (format!("user_id={alice}"), matching(|s| user_is(s, 0))),
            (format!("user_id={bob}"), matching(|s| user_is(s, 1))),
            (
                "from=2026-01-01T00:05:00Z".to_string(),
                matching(|s| s.started_minute >= 5),
            ),
            (
                "to=2026-01-01T00:08:00Z".to_string(),
                matching(|s| s.started_minute <= 8),
            ),
            (
                "from=2026-01-01T00:03:00Z&to=2026-01-01T00:06:00Z".to_string(),
                matching(|s| (3..=6).contains(&s.started_minute)),
            ),
            (
                "identified=true".to_string(),
                matching(|s| s.user.is_some()),
            ),
            (
                "identified=false".to_string(),
                matching(|s| s.user.is_none()),
            ),
            ("completed=true".to_string(), matching(Seed::completed)),
            ("completed=false".to_string(), matching(|s| !s.completed())),
            (
                "identified=false&completed=false".to_string(),
                matching(|s| s.user.is_none() && !s.completed()),
            ),
            (
                "status=success&client_id=web-app".to_string(),
                matching(|s| s.status == "success" && s.client_id == Some("web-app")),
            ),
            (
                format!("user_id={alice}&grant_type=authorization_code&ip_address=192.168."),
                matching(|s| {
                    user_is(s, 0)
                        && s.grant_type == "authorization_code"
                        && ip_contains(s, "192.168.")
                }),
            ),
            ("ip_address=".to_string(), matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "ip_address=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_realm(&server, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let witness = list_realm(&server, "grant_type=password").await;
            assert!(total(&witness) > 0);
            let partial = list_realm(&server, "grant_type=pass").await;
            assert_eq!(total(&partial), 0, "grant_type is an exact match");
            let witness = list_realm(&server, "client_id=web-app").await;
            assert!(total(&witness) > 0);
            let partial = list_realm(&server, "client_id=web").await;
            assert_eq!(total(&partial), 0, "client_id is an exact match");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn flows_of_other_realms_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let foreign_web = list_other(&server, "client_id=web-app").await;
            assert_eq!(
                total(&foreign_web),
                2,
                "the other realm holds web-app flows"
            );
            let body = list_realm(&server, "client_id=web-app&limit=100").await;
            assert_eq!(
                ids(&body).into_iter().collect::<HashSet<_>>(),
                matching(|s| s.client_id == Some("web-app"))
            );

            let foreign_user = ctx().foreign_user;
            let witness = list_other(&server, &format!("user_id={foreign_user}")).await;
            assert_eq!(total(&witness), 8);
            let body = list_realm(&server, &format!("user_id={foreign_user}")).await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let witness = list_other(&server, "ip_address=ipx").await;
            assert_eq!(total(&witness), 1);
            let body = list_realm(&server, "ip_address=ipx").await;
            assert_eq!(total(&body), 0);

            let everything = list_realm(&server, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            let expected: HashSet<Uuid> = ctx().seeds.iter().map(|seed| seed.id).collect();
            assert_eq!(
                ids(&everything).into_iter().collect::<HashSet<_>>(),
                expected
            );

            let stored: i64 =
                sqlx::query_scalar("SELECT count(*) FROM compass_flows WHERE realm_id = $1::uuid")
                    .bind(ctx().other_realm_id.to_string())
                    .fetch_one(&ctx().pool)
                    .await
                    .expect("count foreign flows");
            assert_eq!(total(&list_other(&server, "").await), stored as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn callers_without_rights_list_nothing() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok_as(&server, &ctx().viewer_token, &ctx().realm, "limit=100").await;
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign_witness =
                list_ok_as(&server, &ctx().foreign_viewer_token, &ctx().other_realm, "").await;
            assert!(total(&foreign_witness) > 0);

            for (token, status, what) in [
                (
                    &ctx().no_rights_token,
                    403,
                    "a realm user without view_events",
                ),
                (
                    &ctx().foreign_viewer_token,
                    404,
                    "a viewer of another realm",
                ),
            ] {
                let refused = list(&server, token, &ctx().realm, "").await;
                assert_eq!(refused.status_code(), status, "{what}: {}", refused.text());
                let text = refused.text();
                assert!(
                    !text.contains(&ctx().seeds[0].id.to_string()),
                    "{what}: {text}"
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_other(&server, "ip_address=ipx").await;
            assert_eq!(ips(&witness), ["ipxliteral"]);
            let witness = list_other(&server, "ip_address=underx").await;
            assert_eq!(ips(&witness), ["underxip"]);
            let witness = list_other(&server, "ip_address=backx").await;
            assert_eq!(ips(&witness), ["backxip"]);

            let percent = list_other(&server, "ip_address=%25").await;
            assert_eq!(ips(&percent), ["ip%literal"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_other(&server, "ip_address=_").await;
            assert_eq!(ips(&underscore), ["under_ip"]);
            assert_eq!(total(&underscore), 1);

            let backslash = list_other(&server, "ip_address=%5C").await;
            assert_eq!(ips(&backslash), ["back\\ip"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn search_matches_the_ip_address_only() {
        let server = make_server();
        let cases: [(&str, HashSet<Uuid>); 3] = [
            ("192.168.1.", matching(|s| ip_contains(s, "192.168.1."))),
            (".2", matching(|s| ip_contains(s, ".2"))),
            ("192.168.", matching(|s| s.ip_address.is_some())),
        ];

        rt().block_on(async {
            for (value, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{value}: the fixture must make this search discriminating"
                );
                let body = list_realm(&server, &format!("search={value}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{value}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{value}: total");
                assert!(
                    found.iter().all(|id| seed_of(*id).ip_address.is_some()),
                    "{value}: a row without an address never matches"
                );
            }

            let unknown = list_realm(&server, "search=seed-agent&limit=100").await;
            assert_eq!(total(&unknown), 0, "search ignores the user agent");

            let combined = list_realm(&server, "search=192.168.1.&status=failure&limit=100").await;
            let expected = matching(|s| ip_contains(s, "192.168.1.") && s.status == "failure");
            assert!(!expected.is_empty());
            let found: HashSet<Uuid> = ids(&combined).into_iter().collect();
            assert_eq!(found, expected);
            assert_eq!(total(&combined), expected.len() as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn search_matches_like_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            for (value, expected) in [
                ("%25", "ip%literal"),
                ("_", "under_ip"),
                ("%5C", "back\\ip"),
            ] {
                let body = list_other(&server, &format!("search={value}")).await;
                assert_eq!(ips(&body), [expected], "{value}");
                assert_eq!(total(&body), 1, "{value}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test compass_flows_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("offset=10", "offset"),
                ("order_by=ip_address", "order_by"),
                ("order_by=grant_type", "order_by"),
                ("order=sideways", "order"),
                ("unknown=1", "unknown"),
                ("status=done", "status"),
                ("user_id=nope", "user_id"),
                ("from=yesterday", "from"),
                ("identified=maybe", "identified"),
                ("completed=1", "completed"),
                ("ip_address=a&ip_address=b", "ip_address"),
                ("search=a&search=b", "search"),
                ("created_from=2026-01-01T00:00:00Z", "created_from"),
                ("created_to=2026-01-01T00:00:00Z", "created_to"),
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
