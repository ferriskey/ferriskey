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
    const PASSWORD: &str = "S3ssion-Listing-Pw!";
    const SEED_COUNT: usize = 25;
    const SSO_MARK: &str = "sso-hash-mark";
    const BROWSERS: [&str; 4] = ["Firefox", "Chrome", "Safari", "Edge"];
    const SORT_FIELDS: [&str; 3] = ["last_seen_at", "expires_at", "created_at"];

    struct Seed {
        id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
        persistent: bool,
        last_seen_minute: Option<i32>,
        expires_day: i32,
        created_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let index = i32::try_from(i).expect("small index");
            let expired = i.is_multiple_of(5);
            Self {
                id: Uuid::new_v4(),
                ip_address: if i % 7 == 3 {
                    None
                } else {
                    Some(format!("10.0.{}.{i}", i % 3))
                },
                user_agent: if i % 5 == 4 {
                    None
                } else {
                    Some(format!("{}/{i}", BROWSERS[i % BROWSERS.len()]))
                },
                persistent: i.is_multiple_of(3),
                last_seen_minute: if i % 6 == 1 { None } else { Some(index / 4) },
                expires_day: if expired {
                    index / 3
                } else {
                    40_000 + index / 3
                },
                created_minute: i32::try_from(((i * 7) % SEED_COUNT) / 3).expect("small index"),
            }
        }

        fn expired(&self) -> bool {
            self.expires_day < 40_000
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "last_seen_at" => SortKey::Nullable(
                    self.last_seen_minute.is_none(),
                    self.last_seen_minute.unwrap_or_default(),
                ),
                "expires_at" => SortKey::Number(self.expires_day),
                "created_at" => SortKey::Number(self.created_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Number(i32),
        Nullable(bool, i32),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: PgPool,
        admin_token: String,
        viewer_token: String,
        foreign_viewer_token: String,
        no_rights_token: String,
        self_token: String,
        realm: String,
        other_realm: String,
        target_id: Uuid,
        neighbour_id: Uuid,
        self_id: Uuid,
        foreign_id: Uuid,
        stray_session: Uuid,
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

        let schema = format!("user_sessions_listing_test_{}", Uuid::new_v4().simple());

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
        let realm = format!("sessions-{short}");
        let other_realm = format!("other-{short}");
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let target_id =
            uuid_of(&create_user(&server, &admin_token, &realm, &format!("target-{short}")).await);
        let neighbour_id = uuid_of(
            &create_user(&server, &admin_token, &realm, &format!("neighbour-{short}")).await,
        );
        let foreign_id = uuid_of(
            &create_user(
                &server,
                &admin_token,
                &other_realm,
                &format!("foreign-{short}"),
            )
            .await,
        );

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, target_id, realm_id, seed).await;
        }

        for _ in 0..3 {
            insert_plain_session(
                &pool,
                neighbour_id,
                realm_id,
                Some("10.9.9.9"),
                "neighbour-agent",
            )
            .await;
        }
        let stray_session = insert_plain_session(
            &pool,
            target_id,
            other_realm_id,
            Some("10.0.1.1"),
            "stray-agent",
        )
        .await;

        for (ip, agent) in [
            ("10.8.0.1", "foreign-agent"),
            ("10.8.0.2", "foreign-agent"),
            ("ip%literal", "pct%agent"),
            ("ipxliteral", "pctxagent"),
            ("10.8.0.3", "under_score"),
            ("10.8.0.4", "underxscore"),
            ("10.8.0.5", "back\\slash"),
            ("10.8.0.6", "backxslash"),
        ] {
            insert_plain_session(&pool, foreign_id, other_realm_id, Some(ip), agent).await;
        }

        let viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("viewer-{short}"),
            "session-viewer",
            &["view_users"],
        )
        .await;
        let foreign_viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &other_realm,
            &format!("foreign-viewer-{short}"),
            "foreign-session-viewer",
            &["view_users"],
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

        let self_name = format!("self-{short}");
        let self_user = create_user(&server, &admin_token, &realm, &self_name).await;
        set_password(&server, &admin_token, &realm, &self_user, PASSWORD).await;
        let self_id = uuid_of(&self_user);
        for agent in ["self-agent/1", "self-agent/2"] {
            insert_plain_session(&pool, self_id, realm_id, Some("10.7.0.1"), agent).await;
        }
        let self_token = direct_grant(&server, &realm, &self_name, PASSWORD).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            admin_token,
            viewer_token,
            foreign_viewer_token,
            no_rights_token,
            self_token,
            realm,
            other_realm,
            target_id,
            neighbour_id,
            self_id,
            foreign_id,
            stray_session,
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

    async fn insert_seed(pool: &PgPool, user_id: Uuid, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, realm_id, user_agent, ip_address, created_at, expires_at, last_seen_at, sso_token_hash, persistent, authenticated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $6), \
             TIMESTAMP '2001-01-01 00:00:00' + make_interval(days => $7), \
             CASE WHEN $8::int IS NULL THEN NULL ELSE TIMESTAMP '2026-03-01 00:00:00' + make_interval(mins => $8::int) END, \
             $9, $10, now())",
        )
        .bind(seed.id.to_string())
        .bind(user_id.to_string())
        .bind(realm_id.to_string())
        .bind(seed.user_agent.as_deref())
        .bind(seed.ip_address.as_deref())
        .bind(seed.created_minute)
        .bind(seed.expires_day)
        .bind(seed.last_seen_minute)
        .bind(format!("{SSO_MARK}-{}", seed.id))
        .bind(seed.persistent)
        .execute(pool)
        .await
        .expect("insert user session");
    }

    async fn insert_plain_session(
        pool: &PgPool,
        user_id: Uuid,
        realm_id: Uuid,
        ip_address: Option<&str>,
        user_agent: &str,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO user_sessions (id, user_id, realm_id, user_agent, ip_address, created_at, expires_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, now(), now() + INTERVAL '1 hour')",
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(realm_id.to_string())
        .bind(user_agent)
        .bind(ip_address)
        .execute(pool)
        .await
        .expect("insert plain user session");
        id
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
                "firstname": "Session",
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
                "description": "user sessions listing fixture",
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
        user_id: Uuid,
        query: &str,
    ) -> TestResponse {
        server
            .get(&format!("/realms/{realm}/users/{user_id}/sessions?{query}"))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok_as(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: Uuid,
        query: &str,
    ) -> Value {
        let response = list(server, token, realm, user_id, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing the sessions of {user_id} in {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn list_target(server: &TestServer, query: &str) -> Value {
        list_ok_as(
            server,
            &ctx().admin_token,
            &ctx().realm,
            ctx().target_id,
            query,
        )
        .await
    }

    fn rows(body: &Value) -> &Vec<Value> {
        body["data"].as_array().expect("data array")
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        rows(body)
            .iter()
            .map(|session| Uuid::parse_str(session["id"].as_str().expect("id")).expect("uuid"))
            .collect()
    }

    fn agents(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|session| {
                session["user_agent"]
                    .as_str()
                    .expect("user_agent")
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
            .expect("seeded session")
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

    fn agent_contains(seed: &Seed, needle: &str) -> bool {
        seed.user_agent
            .as_deref()
            .is_some_and(|agent| agent.to_lowercase().contains(needle))
    }

    fn ip_contains(seed: &Seed, needle: &str) -> bool {
        seed.ip_address
            .as_deref()
            .is_some_and(|ip| ip.contains(needle))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_sessions() {
        let server = make_server();
        rt().block_on(async {
            let body = list_target(&server, "").await;

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
            assert_eq!(first["user_id"], ctx().target_id.to_string());
            assert_eq!(first["user_agent"].as_str(), seed.user_agent.as_deref());
            assert_eq!(first["ip_address"].as_str(), seed.ip_address.as_deref());
            assert_eq!(first["persistent"], seed.persistent);
            assert_eq!(
                first["last_seen_at"].is_null(),
                seed.last_seen_minute.is_none()
            );
            assert!(first["created_at"].is_string(), "{first}");
            assert!(first["expires_at"].is_string(), "{first}");
            assert!(first.get("sso_token_hash").is_none(), "{first}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = list_target(&server, &query).await;
                    let second = list_target(&server, &format!("{query}&page=2")).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn never_seen_sessions_sort_last_ascending_and_first_descending() {
        let server = make_server();
        rt().block_on(async {
            let never = matching(|s| s.last_seen_minute.is_none());
            assert!(
                !never.is_empty() && never.len() < SEED_COUNT,
                "the fixture mixes seen and never-seen rows"
            );

            let ascending = list_target(&server, "order_by=last_seen_at&order=asc&limit=100").await;
            let ascending = ids(&ascending);
            let (seen, tail) = ascending.split_at(SEED_COUNT - never.len());
            assert!(
                seen.iter().all(|id| !never.contains(id)),
                "asc: seen rows first"
            );
            assert_eq!(
                tail.iter().copied().collect::<HashSet<_>>(),
                never,
                "asc: nulls last"
            );

            let descending =
                list_target(&server, "order_by=last_seen_at&order=desc&limit=100").await;
            let descending = ids(&descending);
            let (head, seen) = descending.split_at(never.len());
            assert_eq!(
                head.iter().copied().collect::<HashSet<_>>(),
                never,
                "desc: nulls first"
            );
            assert!(
                seen.iter().all(|id| !never.contains(id)),
                "desc: seen rows after"
            );

            let mut tie_breaker: Vec<Uuid> = never.iter().copied().collect();
            tie_breaker.sort();
            assert_eq!(tail, tie_breaker.as_slice(), "asc: nulls ordered by id");
            tie_breaker.reverse();
            assert_eq!(head, tie_breaker.as_slice(), "desc: nulls ordered by id");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_sessions() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            (
                "ip_address=10.0.1.",
                matching(|s| ip_contains(s, "10.0.1.")),
            ),
            ("ip_address=.2", matching(|s| ip_contains(s, ".2"))),
            (
                "user_agent=CHROME",
                matching(|s| agent_contains(s, "chrome")),
            ),
            (
                "user_agent=fari/1",
                matching(|s| agent_contains(s, "fari/1")),
            ),
            ("persistent=true", matching(|s| s.persistent)),
            ("persistent=false", matching(|s| !s.persistent)),
            (
                "user_agent=firefox&persistent=true",
                matching(|s| agent_contains(s, "firefox") && s.persistent),
            ),
            (
                "ip_address=10.0.2.&user_agent=e",
                matching(|s| ip_contains(s, "10.0.2.") && agent_contains(s, "e")),
            ),
            ("ip_address=", matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "ip_address=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_target(&server, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn search_matches_the_ip_address_or_the_user_agent() {
        let server = make_server();
        let cases: [(&str, &str, HashSet<Uuid>); 3] = [
            (
                "10.0.1.",
                "10.0.1.",
                matching(|s| ip_contains(s, "10.0.1.")),
            ),
            (
                "CHROME",
                "chrome",
                matching(|s| agent_contains(s, "chrome")),
            ),
            ("/", "/", matching(|s| agent_contains(s, "/"))),
        ];

        rt().block_on(async {
            for (value, needle, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{value}: the fixture must make this search discriminating"
                );
                assert_eq!(
                    matching(|s| ip_contains(s, needle) || agent_contains(s, needle)),
                    expected,
                    "{value}: only one field may carry the needle"
                );
                let body = list_target(&server, &format!("search={value}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{value}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{value}: total");
            }

            let combined = list_target(&server, "search=firefox&persistent=true&limit=100").await;
            let expected = matching(|s| agent_contains(s, "firefox") && s.persistent);
            let found: HashSet<Uuid> = ids(&combined).into_iter().collect();
            assert_eq!(found, expected);
            assert_eq!(total(&combined), expected.len() as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn search_matches_like_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().other_realm;
            let foreign = ctx().foreign_id;
            let token = &ctx().admin_token;
            for (value, expected) in [
                ("%25", "pct%agent"),
                ("_", "under_score"),
                ("%5C", "back\\slash"),
            ] {
                let body =
                    list_ok_as(&server, token, realm, foreign, &format!("search={value}")).await;
                assert_eq!(agents(&body), [expected], "{value}");
                assert_eq!(total(&body), 1, "{value}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
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
                "created_to=2026-01-01T00:03:00Z",
                matching(|s| s.created_minute < 3),
            ),
            (
                "created_from=2026-01-01T02:05:00%2B02:00",
                matching(|s| s.created_minute >= 5),
            ),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{query}: the fixture must make this range discriminating"
                );
                let body = list_target(&server, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }

            let inverted = list_target(
                &server,
                "created_from=2026-01-01T00:05:00Z&created_to=2026-01-01T00:02:00Z",
            )
            .await;
            assert!(ids(&inverted).is_empty(), "{inverted}");
            assert_eq!(total(&inverted), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn expired_sessions_stay_listed() {
        let server = make_server();
        rt().block_on(async {
            let expired = matching(Seed::expired);
            assert!(
                !expired.is_empty() && expired.len() < SEED_COUNT,
                "the fixture mixes expired and live rows"
            );

            let body = list_target(&server, "order_by=expires_at&order=asc&limit=100").await;
            let listed = ids(&body);
            assert_eq!(total(&body), SEED_COUNT as u64);
            assert_eq!(
                listed[..expired.len()]
                    .iter()
                    .copied()
                    .collect::<HashSet<_>>(),
                expired
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn the_sso_token_hash_is_never_serialized() {
        let server = make_server();
        rt().block_on(async {
            let stored: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM user_sessions WHERE user_id = $1::uuid AND sso_token_hash LIKE $2",
            )
            .bind(ctx().target_id.to_string())
            .bind(format!("{SSO_MARK}%"))
            .fetch_one(&ctx().pool)
            .await
            .expect("count stored sso hashes");
            assert_eq!(stored, SEED_COUNT as i64, "the fixture stores sso hashes");

            for query in ["limit=100", "order_by=last_seen_at&page=2", "persistent=true"] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().target_id,
                    query,
                )
                .await;
                assert_eq!(response.status_code(), 200, "{query}: {}", response.text());
                let text = response.text();
                assert!(
                    text.contains(&ctx().target_id.to_string()),
                    "{query}: rows are listed"
                );
                assert!(!text.contains(SSO_MARK), "{query}: {text}");
                assert!(!text.contains("sso_token_hash"), "{query}: {text}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn sessions_of_other_users_and_realms_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let neighbour = list_ok_as(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                ctx().neighbour_id,
                "user_agent=neighbour",
            )
            .await;
            assert_eq!(total(&neighbour), 3, "the neighbour holds its own sessions");
            let body = list_target(&server, "user_agent=neighbour").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let stray: i64 = sqlx::query_scalar(
                "SELECT count(*) FROM user_sessions WHERE id = $1::uuid AND user_id = $2::uuid",
            )
            .bind(ctx().stray_session.to_string())
            .bind(ctx().target_id.to_string())
            .fetch_one(&ctx().pool)
            .await
            .expect("count stray session");
            assert_eq!(stray, 1, "the target owns a row bound to the other realm");
            let body = list_target(&server, "user_agent=stray&limit=100").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_target(&server, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            let expected: HashSet<Uuid> = ctx().seeds.iter().map(|seed| seed.id).collect();
            assert_eq!(
                ids(&everything).into_iter().collect::<HashSet<_>>(),
                expected
            );

            let foreign = list_ok_as(
                &server,
                &ctx().admin_token,
                &ctx().other_realm,
                ctx().foreign_id,
                "user_agent=foreign-agent",
            )
            .await;
            assert_eq!(
                total(&foreign),
                2,
                "the foreign user is listed in its realm"
            );
            let refused = list(
                &server,
                &ctx().admin_token,
                &ctx().realm,
                ctx().foreign_id,
                "",
            )
            .await;
            assert_eq!(refused.status_code(), 404, "{}", refused.text());
            assert!(
                !refused.text().contains("foreign-agent"),
                "{}",
                refused.text()
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn callers_without_rights_list_nothing() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok_as(
                &server,
                &ctx().viewer_token,
                &ctx().realm,
                ctx().target_id,
                "limit=100",
            )
            .await;
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign_witness = list_ok_as(
                &server,
                &ctx().foreign_viewer_token,
                &ctx().other_realm,
                ctx().foreign_id,
                "",
            )
            .await;
            assert!(total(&foreign_witness) > 0);

            for (token, status, what) in [
                (
                    &ctx().no_rights_token,
                    403,
                    "a realm user without view_users",
                ),
                (
                    &ctx().foreign_viewer_token,
                    404,
                    "a viewer of another realm",
                ),
                (&ctx().self_token, 403, "a user listing somebody else"),
            ] {
                let refused = list(&server, token, &ctx().realm, ctx().target_id, "").await;
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn a_user_lists_its_own_sessions_without_any_permission() {
        let server = make_server();
        rt().block_on(async {
            let refused = list(
                &server,
                &ctx().self_token,
                &ctx().realm,
                ctx().target_id,
                "",
            )
            .await;
            assert_eq!(refused.status_code(), 403, "{}", refused.text());

            let own = list_ok_as(
                &server,
                &ctx().self_token,
                &ctx().realm,
                ctx().self_id,
                "user_agent=self-agent&order_by=created_at&order=asc",
            )
            .await;
            assert_eq!(total(&own), 2);
            let mut listed = agents(&own);
            listed.sort();
            assert_eq!(listed, ["self-agent/1", "self-agent/2"]);
            assert!(
                rows(&own)
                    .iter()
                    .all(|session| session["user_id"] == ctx().self_id.to_string())
            );

            let paged = list_ok_as(
                &server,
                &ctx().self_token,
                &ctx().realm,
                ctx().self_id,
                "user_agent=self-agent&limit=1&page=2",
            )
            .await;
            assert_eq!(ids(&paged).len(), 1);
            assert_eq!(paged["metadata"]["total"], 2);
            assert_eq!(paged["metadata"]["prev_page"], 1);

            let invalid = list(
                &server,
                &ctx().self_token,
                &ctx().realm,
                ctx().self_id,
                "limit=101",
            )
            .await;
            assert_eq!(invalid.status_code(), 400, "{}", invalid.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().other_realm;
            let foreign = ctx().foreign_id;
            let token = &ctx().admin_token;

            let witness = list_ok_as(&server, token, realm, foreign, "user_agent=pctx").await;
            assert_eq!(agents(&witness), ["pctxagent"]);
            let witness = list_ok_as(&server, token, realm, foreign, "user_agent=underx").await;
            assert_eq!(agents(&witness), ["underxscore"]);
            let witness = list_ok_as(&server, token, realm, foreign, "user_agent=backx").await;
            assert_eq!(agents(&witness), ["backxslash"]);
            let witness = list_ok_as(&server, token, realm, foreign, "ip_address=ipx").await;
            assert_eq!(agents(&witness), ["pctxagent"]);

            let percent = list_ok_as(&server, token, realm, foreign, "user_agent=%25").await;
            assert_eq!(agents(&percent), ["pct%agent"]);
            assert_eq!(total(&percent), 1);

            let percent_ip = list_ok_as(&server, token, realm, foreign, "ip_address=%25").await;
            assert_eq!(agents(&percent_ip), ["pct%agent"]);
            assert_eq!(total(&percent_ip), 1);

            let underscore = list_ok_as(&server, token, realm, foreign, "user_agent=_").await;
            assert_eq!(agents(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let backslash = list_ok_as(&server, token, realm, foreign, "user_agent=%5C").await;
            assert_eq!(agents(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_sessions_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=user_agent", "order_by"),
                ("order_by=sso_token_hash", "order_by"),
                ("order=sideways", "order"),
                ("unknown=1", "unknown"),
                ("persistent=maybe", "persistent"),
                ("user_agent=a&user_agent=b", "user_agent"),
                ("search=a&search=b", "search"),
                ("created_from=2026-10-05", "created_from"),
                ("created_to=2026-10-05", "created_to"),
            ] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().target_id,
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
