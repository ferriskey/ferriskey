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
    const PASSWORD: &str = "Seawatch-Listing-Pw1!";
    const SEED_COUNT: usize = 25;
    const EVENT_TYPES: [&str; 4] = [
        "login_success",
        "login_failure",
        "user_created",
        "session_revoked",
    ];
    const STATUSES: [&str; 2] = ["success", "failure"];
    const TARGET_TYPES: [Option<&str>; 5] =
        [Some("user"), Some("client"), Some("session"), None, None];
    const SORT_FIELDS: [&str; 4] = ["event_type", "status", "timestamp", "created_at"];

    struct Seed {
        id: Uuid,
        event_type: &'static str,
        status: &'static str,
        actor: Option<usize>,
        target_type: Option<&'static str>,
        target: Option<usize>,
        ip_address: Option<String>,
        minute: i32,
        created_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let target_type = TARGET_TYPES[i % TARGET_TYPES.len()];
            Self {
                id: Uuid::new_v4(),
                event_type: EVENT_TYPES[(i / 2) % EVENT_TYPES.len()],
                status: STATUSES[(i / 3) % STATUSES.len()],
                actor: match i % 4 {
                    0 => Some(0),
                    1 => Some(1),
                    _ => None,
                },
                target_type,
                target: target_type.map(|_| i % 3),
                ip_address: if i % 6 == 5 {
                    None
                } else {
                    Some(format!("10.0.{}.{i}", i % 3))
                },
                minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
                created_minute: i32::try_from(((i * 7) % SEED_COUNT) / 3).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "event_type" => SortKey::Text(self.event_type),
                "status" => SortKey::Text(self.status),
                "timestamp" => SortKey::Number(self.minute),
                "created_at" => SortKey::Number(self.created_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(&'static str),
        Number(i32),
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
        actors: [Uuid; 2],
        targets: [Uuid; 3],
        foreign_actor: Uuid,
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

        let schema = format!("security_events_listing_test_{}", Uuid::new_v4().simple());

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
        let realm = format!("events-{short}");
        let other_realm = format!("other-{short}");
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("viewer-{short}"),
            "event-viewer",
            &["view_events"],
        )
        .await;
        let foreign_viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &other_realm,
            &format!("foreign-viewer-{short}"),
            "foreign-event-viewer",
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

        sqlx::query("DELETE FROM security_events WHERE realm_id = ANY($1::uuid[])")
            .bind(vec![realm_id.to_string(), other_realm_id.to_string()])
            .execute(&pool)
            .await
            .expect("clear the fixture events");

        let actors = [Uuid::new_v4(), Uuid::new_v4()];
        let targets = [Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let foreign_actor = Uuid::new_v4();

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, &actors, &targets, seed).await;
        }

        for ip in [
            "10.0.1.1",
            "10.0.1.2",
            "ip%literal",
            "ipxliteral",
            "under_ip",
            "underxip",
            "back\\ip",
            "backxip",
        ] {
            insert_plain_event(&pool, other_realm_id, foreign_actor, targets[0], ip).await;
        }

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
            actors,
            targets,
            foreign_actor,
            seeds,
        }
    }

    async fn insert_seed(
        pool: &PgPool,
        realm_id: Uuid,
        actors: &[Uuid; 2],
        targets: &[Uuid; 3],
        seed: &Seed,
    ) {
        sqlx::query(
            "INSERT INTO security_events (id, realm_id, actor_id, actor_type, event_type, status, target_type, target_id, timestamp, ip_address, user_agent, created_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'user', $4, $5, $6, $7::uuid, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8), $9, 'seed-agent', \
             TIMESTAMP '2026-02-01 00:00:00' + make_interval(mins => $10))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(seed.actor.map(|index| actors[index].to_string()))
        .bind(seed.event_type)
        .bind(seed.status)
        .bind(seed.target_type)
        .bind(seed.target.map(|index| targets[index].to_string()))
        .bind(seed.minute)
        .bind(seed.ip_address.as_deref())
        .bind(seed.created_minute)
        .execute(pool)
        .await
        .expect("insert security event");
    }

    async fn insert_plain_event(
        pool: &PgPool,
        realm_id: Uuid,
        actor_id: Uuid,
        target_id: Uuid,
        ip_address: &str,
    ) {
        sqlx::query(
            "INSERT INTO security_events (id, realm_id, actor_id, event_type, status, target_type, target_id, timestamp, ip_address) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, 'login_failure', 'failure', 'client', $4::uuid, now(), $5)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(actor_id.to_string())
        .bind(target_id.to_string())
        .bind(ip_address)
        .execute(pool)
        .await
        .expect("insert plain security event");
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
                "firstname": "Event",
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
                "description": "security events listing fixture",
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

    const NO_QUERY: &str = "";

    async fn list(server: &TestServer, token: &str, realm: &str, query: &str) -> TestResponse {
        server
            .get(&format!(
                "/realms/{realm}/seawatch/v1/security-events?{query}"
            ))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok_as(server: &TestServer, token: &str, realm: &str, query: &str) -> Value {
        let response = list(server, token, realm, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing the security events of {realm} with `{query}` failed: {}",
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
            .map(|event| Uuid::parse_str(event["id"].as_str().expect("id")).expect("uuid"))
            .collect()
    }

    fn ips(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|event| {
                event["ip_address"]
                    .as_str()
                    .expect("ip_address")
                    .to_string()
            })
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
            .expect("seeded event")
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

    async fn stored_in(realm_id: Uuid) -> u64 {
        let count: i64 =
            sqlx::query_scalar("SELECT count(*) FROM security_events WHERE realm_id = $1::uuid")
                .bind(realm_id.to_string())
                .fetch_one(&ctx().pool)
                .await
                .expect("count stored events");
        u64::try_from(count).expect("non-negative count")
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_events() {
        let server = make_server();
        rt().block_on(async {
            assert_eq!(
                stored_in(ctx().realm_id).await,
                SEED_COUNT as u64,
                "only the seeded events are stored"
            );

            let body = list_realm(&server, NO_QUERY).await;

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
            assert_eq!(first["event_type"], seed.event_type);
            assert_eq!(first["status"], seed.status);
            assert_eq!(first["target_type"].as_str(), seed.target_type);
            assert_eq!(first["ip_address"].as_str(), seed.ip_address.as_deref());
            assert!(first["timestamp"].is_string(), "{first}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_events() {
        let server = make_server();
        let [alice, bob] = ctx().actors;
        let [first_target, second_target, _] = ctx().targets;
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "ip_address=10.0.1.".to_string(),
                matching(|s| ip_contains(s, "10.0.1.")),
            ),
            (
                "ip_address=.2".to_string(),
                matching(|s| ip_contains(s, ".2")),
            ),
            (
                "event_types=login_failure".to_string(),
                matching(|s| s.event_type == "login_failure"),
            ),
            (
                "event_types=user_created,session_revoked".to_string(),
                matching(|s| matches!(s.event_type, "user_created" | "session_revoked")),
            ),
            (
                "status=failure".to_string(),
                matching(|s| s.status == "failure"),
            ),
            (
                "status=success".to_string(),
                matching(|s| s.status == "success"),
            ),
            (
                "target_type=client".to_string(),
                matching(|s| s.target_type == Some("client")),
            ),
            (
                "target_type=session".to_string(),
                matching(|s| s.target_type == Some("session")),
            ),
            (
                format!("actor_id={alice}"),
                matching(|s| s.actor == Some(0)),
            ),
            (format!("actor_id={bob}"), matching(|s| s.actor == Some(1))),
            (
                format!("client_id={first_target}"),
                matching(|s| s.target == Some(0)),
            ),
            (
                format!("client_id={second_target}"),
                matching(|s| s.target == Some(1)),
            ),
            (
                "from_timestamp=2026-01-01T00:05:00Z".to_string(),
                matching(|s| s.minute >= 5),
            ),
            (
                "to_timestamp=2026-01-01T00:08:00Z".to_string(),
                matching(|s| s.minute <= 8),
            ),
            (
                "from_timestamp=2026-01-01T00:03:00Z&to_timestamp=2026-01-01T00:06:00Z".to_string(),
                matching(|s| (3..=6).contains(&s.minute)),
            ),
            (
                "status=failure&target_type=user".to_string(),
                matching(|s| s.status == "failure" && s.target_type == Some("user")),
            ),
            (
                format!("actor_id={alice}&event_types=login_success,user_created&ip_address=10.0."),
                matching(|s| {
                    s.actor == Some(0)
                        && matches!(s.event_type, "login_success" | "user_created")
                        && ip_contains(s, "10.0.")
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

            let witness = list_realm(&server, "target_type=client").await;
            assert!(total(&witness) > 0);
            let partial = list_realm(&server, "target_type=cli").await;
            assert_eq!(total(&partial), 0, "target_type is an exact match");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn events_of_other_realms_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let first_target = ctx().targets[0];
            let foreign = list_other(&server, &format!("client_id={first_target}")).await;
            assert_eq!(total(&foreign), 8, "the other realm targets the same id");
            let body = list_realm(&server, &format!("client_id={first_target}&limit=100")).await;
            assert_eq!(
                ids(&body).into_iter().collect::<HashSet<_>>(),
                matching(|s| s.target == Some(0))
            );

            let foreign_actor = ctx().foreign_actor;
            let witness = list_other(&server, &format!("actor_id={foreign_actor}")).await;
            assert_eq!(total(&witness), 8);
            let body = list_realm(&server, &format!("actor_id={foreign_actor}")).await;
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

            assert_eq!(
                total(&list_other(&server, NO_QUERY).await),
                stored_in(ctx().other_realm_id).await
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn callers_without_rights_list_nothing() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok_as(&server, &ctx().viewer_token, &ctx().realm, "limit=100").await;
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign_witness = list_ok_as(
                &server,
                &ctx().foreign_viewer_token,
                &ctx().other_realm,
                NO_QUERY,
            )
            .await;
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
                let refused = list(&server, token, &ctx().realm, NO_QUERY).await;
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn search_matches_the_ip_address_only() {
        let server = make_server();
        let cases: [(&str, HashSet<Uuid>); 3] = [
            ("10.0.1.", matching(|s| ip_contains(s, "10.0.1."))),
            (".2", matching(|s| ip_contains(s, ".2"))),
            ("10.0.", matching(|s| s.ip_address.is_some())),
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

            let combined = list_realm(&server, "search=10.0.1.&status=failure&limit=100").await;
            let expected = matching(|s| ip_contains(s, "10.0.1.") && s.status == "failure");
            assert!(!expected.is_empty());
            let found: HashSet<Uuid> = ids(&combined).into_iter().collect();
            assert_eq!(found, expected);
            assert_eq!(total(&combined), expected.len() as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test security_events_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=1000", "limit"),
                ("limit=0", "limit"),
                ("offset=10", "offset"),
                ("order_by=ip_address", "order_by"),
                ("order_by=actor_id", "order_by"),
                ("order=sideways", "order"),
                ("unknown=1", "unknown"),
                ("status=done", "status"),
                ("event_types=nope", "event_types"),
                ("event_types=login_failure,nope", "event_types"),
                ("event_types=login_failure,", "event_types"),
                ("actor_id=nope", "actor_id"),
                ("client_id=nope", "client_id"),
                ("from_timestamp=yesterday", "from_timestamp"),
                ("to_timestamp=tomorrow", "to_timestamp"),
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
