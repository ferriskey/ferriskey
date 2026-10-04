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
    const WORDS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];
    const SORT_FIELDS: [&str; 3] = ["name", "created_at", "updated_at"];

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Link {
        None,
        First,
        Second,
    }

    struct Seed {
        id: Uuid,
        name: String,
        description: Option<String>,
        require_mfa: bool,
        permissions: i64,
        link: Link,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: format!("role-{i:02}"),
                description: (i % 6 != 5).then(|| format!("desc {}", WORDS[i % WORDS.len()])),
                require_mfa: i.is_multiple_of(3),
                permissions: if i % 5 == 2 { 0 } else { 1 << (i % 4) },
                link: match i % 4 {
                    1 => Link::First,
                    2 => Link::Second,
                    _ => Link::None,
                },
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "name" => SortKey::Text(self.name.clone()),
                "created_at" => SortKey::Minute(self.created_minute),
                "updated_at" => SortKey::Minute(self.updated_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(String),
        Minute(i32),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        viewer_token: String,
        realm: String,
        other_realm: String,
        first_client: Uuid,
        second_client: Uuid,
        foreign_client: Uuid,
        seeds: Vec<Seed>,
        foreign_role_id: Uuid,
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

        let schema = format!("roles_listing_test_{}", Uuid::new_v4().simple());

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
            sqlx::query_scalar("SELECT count(*) FROM roles WHERE realm_id = $1::uuid")
                .bind(realm_id.to_string())
                .fetch_one(&pool)
                .await
                .expect("count roles");
        assert_eq!(existing, 0, "a fresh realm must start without roles");

        let first_client = insert_client(&pool, realm_id, "first-app").await;
        let second_client = insert_client(&pool, realm_id, "second-app").await;
        let foreign_client = insert_client(&pool, other_realm_id, "foreign-app").await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            let client = match seed.link {
                Link::None => None,
                Link::First => Some(first_client),
                Link::Second => Some(second_client),
            };
            insert_seed(&pool, realm_id, seed, client).await;
        }

        let foreign_role_id =
            insert_plain_role(&pool, other_realm_id, "role-50", Some(foreign_client)).await;
        for name in [
            "role-51",
            "role-52",
            "pct%role",
            "pctxrole",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_plain_role(&pool, other_realm_id, name, None).await;
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
            &["view_roles"],
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
            first_client,
            second_client,
            foreign_client,
            seeds,
            foreign_role_id,
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

    async fn insert_client(pool: &PgPool, realm_id: Uuid, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $3, true, 'openid-connect', false, false, 'confidential', now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert client");
        id
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed, client: Option<Uuid>) {
        sqlx::query(
            "INSERT INTO roles (id, realm_id, client_id, name, description, permissions, require_mfa, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $9))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(client.map(|id| id.to_string()))
        .bind(&seed.name)
        .bind(seed.description.as_deref())
        .bind(seed.permissions)
        .bind(seed.require_mfa)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert role");
    }

    async fn insert_plain_role(
        pool: &PgPool,
        realm_id: Uuid,
        name: &str,
        client: Option<Uuid>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO roles (id, realm_id, client_id, name, description, permissions, require_mfa, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $4, 0, false, now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(client.map(|id| id.to_string()))
        .bind(name)
        .execute(pool)
        .await
        .expect("insert plain role");
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
                "description": "roles listing fixture",
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
            .get(&format!("/realms/{realm}/roles?{query}"))
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
            .map(|role| Uuid::parse_str(role["id"].as_str().expect("role id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|role| role["name"].as_str().expect("name").to_string())
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

    fn described(seed: &Seed, needle: &str) -> bool {
        seed.description
            .as_deref()
            .is_some_and(|description| description.contains(needle))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_roles() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_roles() {
        let server = make_server();
        let first_client = ctx().first_client;
        let second_client = ctx().second_client;
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "name=ROLE-1".to_string(),
                matching(|s| s.name.contains("role-1")),
            ),
            (
                "description=GAMMA".to_string(),
                matching(|s| described(s, "gamma")),
            ),
            (
                "description=desc".to_string(),
                matching(|s| s.description.is_some()),
            ),
            ("require_mfa=true".to_string(), matching(|s| s.require_mfa)),
            (
                "require_mfa=false".to_string(),
                matching(|s| !s.require_mfa),
            ),
            (
                format!("client_id={first_client}"),
                matching(|s| s.link == Link::First),
            ),
            (
                format!("client_id={second_client}"),
                matching(|s| s.link == Link::Second),
            ),
            (
                "scope=realm".to_string(),
                matching(|s| s.link == Link::None),
            ),
            (
                "scope=client".to_string(),
                matching(|s| s.link != Link::None),
            ),
            (
                "has_permissions=true".to_string(),
                matching(|s| s.permissions != 0),
            ),
            (
                "has_permissions=false".to_string(),
                matching(|s| s.permissions == 0),
            ),
            (
                "scope=client&has_permissions=false".to_string(),
                matching(|s| s.link != Link::None && s.permissions == 0),
            ),
            (
                "name=role-1&require_mfa=true".to_string(),
                matching(|s| s.name.contains("role-1") && s.require_mfa),
            ),
            ("name=".to_string(), matching(|_| true)),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn search_matches_the_name_the_client_identifier_or_the_qualified_value() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("search=ROLE-1", matching(|s| s.name.contains("role-1"))),
            ("search=FIRST", matching(|s| s.link == Link::First)),
            ("search=app", matching(|s| s.link != Link::None)),
            (
                "search=FIRST-APP.role-0",
                matching(|s| s.link == Link::First && s.name.contains("role-0")),
            ),
            (
                "search=st-app.0",
                matching(|s| s.link == Link::First && s.name.contains('0')),
            ),
            (
                "search=second&name=role-1",
                matching(|s| s.link == Link::Second && s.name.contains("role-1")),
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

            let none = list_ok(&server, &ctx().realm, "search=first-app.zzz").await;
            assert!(ids(&none).is_empty(), "{none}");
            assert_eq!(total(&none), 0);

            let empty = list_ok(&server, &ctx().realm, "search=&limit=100").await;
            assert_eq!(total(&empty), SEED_COUNT as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn search_never_reaches_another_realm() {
        let server = make_server();
        rt().block_on(async {
            for query in [
                "search=foreign",
                "search=foreign-app.role-50",
                "search=role-50",
            ] {
                let witness = list_ok(&server, &ctx().other_realm, query).await;
                assert_eq!(ids(&witness), [ctx().foreign_role_id], "{query}: witness");
                assert_eq!(total(&witness), 1, "{query}: witness total");

                let body = list_ok(&server, &ctx().realm, query).await;
                assert!(ids(&body).is_empty(), "{query}: {body}");
                assert_eq!(total(&body), 0, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn search_matches_like_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "search=pctx").await;
            assert_eq!(names(&witness), ["pctxrole"]);

            let percent = list_ok(&server, &ctx().other_realm, "search=%25").await;
            assert_eq!(names(&percent), ["pct%role"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "search=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let qualified = list_ok(&server, &ctx().other_realm, "search=%25.role").await;
            assert!(ids(&qualified).is_empty(), "{qualified}");
            assert_eq!(total(&qualified), 0);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn client_roles_carry_their_client() {
        let server = make_server();
        rt().block_on(async {
            let first_client = ctx().first_client.to_string();
            let body = list_ok(&server, &ctx().realm, &format!("client_id={first_client}")).await;
            assert!(!rows(&body).is_empty());
            for role in rows(&body) {
                assert_eq!(role["client_id"], first_client.as_str(), "{role}");
                assert_eq!(role["client"]["id"], first_client.as_str(), "{role}");
                assert_eq!(role["client"]["client_id"], "first-app", "{role}");
            }

            let realm_only = list_ok(&server, &ctx().realm, "limit=100").await;
            assert!(
                rows(&realm_only)
                    .iter()
                    .filter(|role| role["client_id"].is_null())
                    .all(|role| role["client"].is_null())
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn roles_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=role-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds role-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=role-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let foreign_client = ctx().foreign_client;
            let witness = list_ok(
                &server,
                &ctx().other_realm,
                &format!("client_id={foreign_client}"),
            )
            .await;
            assert_eq!(ids(&witness), [ctx().foreign_role_id]);
            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("client_id={foreign_client}"),
            )
            .await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let witness = list_ok(&server, &ctx().other_realm, "scope=client").await;
            assert_eq!(ids(&witness), [ctx().foreign_role_id]);
            let body = list_ok(&server, &ctx().realm, "scope=client&limit=100").await;
            assert!(!ids(&body).contains(&ctx().foreign_role_id), "{body}");
            assert_eq!(
                total(&body),
                matching(|s| s.link != Link::None).len() as u64
            );

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                names(&everything)
                    .iter()
                    .all(|name| name.starts_with("role-") && name.len() == 7)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
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
            assert!(!foreign.contains("role-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxrole"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxscore"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%role"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "description=%25").await;
            assert_eq!(names(&percent), ["pct%role"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxslash"]);
            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn ids_select_exactly_the_given_roles_of_the_realm() {
        let server = make_server();
        rt().block_on(async {
            let seeds = &ctx().seeds;
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            let wanted = [seeds[1].id, seeds[6].id, seeds[22].id];
            assert!(wanted.iter().all(|id| listed.contains(id)));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign = ctx().foreign_role_id;
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

            let narrowed = list_ok(
                &server,
                &ctx().realm,
                &format!("ids={joined}&require_mfa=true"),
            )
            .await;
            let expected: HashSet<Uuid> = [&seeds[1], &seeds[6], &seeds[22]]
                .into_iter()
                .filter(|seed| seed.require_mfa)
                .map(|seed| seed.id)
                .collect();
            assert!(!expected.is_empty() && expected.len() < 3);
            assert_eq!(ids(&narrowed).into_iter().collect::<HashSet<_>>(), expected);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test roles_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=permissions", "order_by"),
                ("unknown=1", "unknown"),
                ("require_mfa=maybe", "require_mfa"),
                ("scope=global", "scope"),
                ("has_permissions=maybe", "has_permissions"),
                ("client_id=not-a-uuid", "client_id"),
                ("ids=not-a-uuid", "ids"),
                ("name=a&name=b", "name"),
                ("search=a&search=b", "search"),
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
