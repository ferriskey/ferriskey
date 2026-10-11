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
    const WORDS: [&str; 4] = ["alpha", "beta", "gamma", "delta"];
    const SORT_FIELDS: [&str; 3] = ["name", "created_at", "updated_at"];
    const SCOPE_TYPES: [&str; 3] = ["DEFAULT", "OPTIONAL", "NONE"];

    struct Seed {
        id: Uuid,
        name: String,
        description: Option<String>,
        protocol: &'static str,
        scope_type: &'static str,
        mappers: usize,
        assigned: bool,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: format!("scope-{i:02}"),
                description: (i % 6 != 5).then(|| format!("desc {}", WORDS[i % WORDS.len()])),
                protocol: if i.is_multiple_of(5) {
                    "saml"
                } else {
                    "openid-connect"
                },
                scope_type: SCOPE_TYPES[i % SCOPE_TYPES.len()],
                mappers: match i % 4 {
                    1 => 1,
                    2 => 2,
                    _ => 0,
                },
                assigned: i.is_multiple_of(7),
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
        seeds: Vec<Seed>,
        mapped_client: Uuid,
        other_client: Uuid,
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

        let schema = format!("client_scopes_listing_test_{}", Uuid::new_v4().simple());

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

        clear_scopes(&pool, realm_id).await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, seed).await;
        }

        let mapped_client = insert_client(&pool, realm_id, "mapped-app").await;
        let other_client = insert_client(&pool, realm_id, "other-app").await;
        for seed in seeds.iter().filter(|seed| seed.assigned) {
            map_scope(&pool, mapped_client, seed.id).await;
        }
        let elsewhere = seeds
            .iter()
            .find(|seed| !seed.assigned)
            .expect("an unassigned seed");
        map_scope(&pool, other_client, elsewhere.id).await;

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
            &["view_client_scopes"],
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

        clear_scopes(&pool, other_realm_id).await;
        for name in [
            "scope-50",
            "scope-51",
            "scope-52",
            "pct%scope",
            "pctxscope",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_plain_scope(&pool, other_realm_id, name).await;
        }

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            realm,
            other_realm,
            seeds,
            mapped_client,
            other_client,
        }
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

    async fn map_scope(pool: &PgPool, client_id: Uuid, scope_id: Uuid) {
        sqlx::query(
            "INSERT INTO client_scope_mappings (client_id, client_scope_id, default_scope_type) \
             VALUES ($1::uuid, $2::uuid, 'DEFAULT')",
        )
        .bind(client_id.to_string())
        .bind(scope_id.to_string())
        .execute(pool)
        .await
        .expect("map client scope");
    }

    async fn realm_id_of(pool: &PgPool, name: &str) -> Uuid {
        let id: String = sqlx::query_scalar("SELECT id::text FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .expect("realm id");
        Uuid::parse_str(&id).expect("realm uuid")
    }

    async fn clear_scopes(pool: &PgPool, realm_id: Uuid) {
        sqlx::query("DELETE FROM client_scopes WHERE realm_id = $1::uuid")
            .bind(realm_id.to_string())
            .execute(pool)
            .await
            .expect("delete default client scopes");
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO client_scopes (id, realm_id, name, description, protocol, default_scope_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $7), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $8))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.name)
        .bind(seed.description.as_deref())
        .bind(seed.protocol)
        .bind(seed.scope_type)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert client scope");
        for n in 0..seed.mappers {
            sqlx::query(
                "INSERT INTO client_scope_protocol_mappers (id, client_scope_id, name, mapper_type, config, created_at) \
                 VALUES ($1::uuid, $2::uuid, $3, 'oidc-hardcoded-claim-mapper', '{}'::jsonb, now())",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(seed.id.to_string())
            .bind(format!("{}-mapper-{n}", seed.name))
            .execute(pool)
            .await
            .expect("insert protocol mapper");
        }
    }

    async fn insert_plain_scope(pool: &PgPool, realm_id: Uuid, name: &str) {
        sqlx::query(
            "INSERT INTO client_scopes (id, realm_id, name, description, protocol, default_scope_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $3, 'openid-connect', 'OPTIONAL', now(), now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert plain client scope");
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
                "description": "client scopes listing fixture",
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
            .get(&format!("/realms/{realm}/client-scopes?{query}"))
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
            .map(|scope| Uuid::parse_str(scope["id"].as_str().expect("scope id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|scope| scope["name"].as_str().expect("name").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_scopes() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_scopes() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("name=SCOPE-1", matching(|s| s.name.contains("scope-1"))),
            ("description=GAMMA", matching(|s| described(s, "gamma"))),
            ("description=desc", matching(|s| s.description.is_some())),
            ("protocol=saml", matching(|s| s.protocol == "saml")),
            (
                "protocol=openid-connect",
                matching(|s| s.protocol == "openid-connect"),
            ),
            ("protocol=open", HashSet::new()),
            (
                "default_scope_type=DEFAULT",
                matching(|s| s.scope_type == "DEFAULT"),
            ),
            (
                "default_scope_type=OPTIONAL",
                matching(|s| s.scope_type == "OPTIONAL"),
            ),
            (
                "default_scope_type=NONE",
                matching(|s| s.scope_type == "NONE"),
            ),
            ("has_protocol_mappers=true", matching(|s| s.mappers > 0)),
            ("has_protocol_mappers=false", matching(|s| s.mappers == 0)),
            ("search=SCOPE-2", matching(|s| s.name.contains("scope-2"))),
            ("search=beta", matching(|s| described(s, "beta"))),
            (
                "default_scope_type=DEFAULT&has_protocol_mappers=false",
                matching(|s| s.scope_type == "DEFAULT" && s.mappers == 0),
            ),
            (
                "name=scope-1&protocol=openid-connect",
                matching(|s| s.name.contains("scope-1") && s.protocol == "openid-connect"),
            ),
            ("name=", matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT)
                        || query == "name="
                        || query == "protocol=open",
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn not_assigned_to_client_returns_exactly_the_unassigned_scopes() {
        let server = make_server();
        rt().block_on(async {
            let assigned = matching(|s| s.assigned);
            assert!(!assigned.is_empty() && assigned.len() < SEED_COUNT);
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            assert!(assigned.is_subset(&listed));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let mapped = ctx().mapped_client;
            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("not_assigned_to_client={mapped}&limit=100"),
            )
            .await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            let expected = matching(|s| !s.assigned);
            assert_eq!(found, expected);
            assert_eq!(total(&body), expected.len() as u64);

            let narrowed = list_ok(
                &server,
                &ctx().realm,
                &format!("not_assigned_to_client={mapped}&search=desc&limit=100"),
            )
            .await;
            let expected = matching(|s| !s.assigned && s.description.is_some());
            assert_eq!(ids(&narrowed).into_iter().collect::<HashSet<_>>(), expected);
            assert_eq!(total(&narrowed), expected.len() as u64);

            let other = ctx().other_client;
            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("not_assigned_to_client={other}&limit=100"),
            )
            .await;
            let elsewhere = ctx()
                .seeds
                .iter()
                .find(|seed| !seed.assigned)
                .expect("an unassigned seed")
                .id;
            let expected = matching(|s| s.id != elsewhere);
            assert_eq!(ids(&body).into_iter().collect::<HashSet<_>>(), expected);
            assert_eq!(total(&body), SEED_COUNT as u64 - 1);
            let unknown = Uuid::new_v4();
            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("not_assigned_to_client={unknown}"),
            )
            .await;
            assert_eq!(total(&body), SEED_COUNT as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn listed_scopes_carry_their_protocol_mappers() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(&server, &ctx().realm, "limit=100").await;
            let counts: HashMap<Uuid, usize> = rows(&body)
                .iter()
                .map(|scope| {
                    let id =
                        Uuid::parse_str(scope["id"].as_str().expect("scope id")).expect("uuid");
                    let mappers = scope["protocol_mappers"]
                        .as_array()
                        .expect("protocol_mappers array");
                    for mapper in mappers {
                        assert_eq!(mapper["client_scope_id"], id.to_string(), "{scope}");
                    }
                    (id, mappers.len())
                })
                .collect();
            assert_eq!(counts.len(), SEED_COUNT);
            for seed in &ctx().seeds {
                assert_eq!(counts.get(&seed.id), Some(&seed.mappers), "{}", seed.name);
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn scopes_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=scope-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds scope-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=scope-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let witness = list_ok(&server, &ctx().other_realm, "search=pct").await;
            assert_eq!(total(&witness), 2);
            let body = list_ok(&server, &ctx().realm, "search=pct").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                names(&everything)
                    .iter()
                    .all(|name| name.starts_with("scope-") && name.len() == 8)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
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
            assert!(!foreign.contains("scope-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxscope"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxscore"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%scope"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "description=%25").await;
            assert_eq!(names(&percent), ["pct%scope"]);
            assert_eq!(total(&percent), 1);

            let percent = list_ok(&server, &ctx().other_realm, "search=%25").await;
            assert_eq!(names(&percent), ["pct%scope"]);
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
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
                "created_from=2026-01-01T00:02:00Z&created_to=2026-01-01T00:05:00Z&name=scope-1",
                matching(|s| (2..5).contains(&s.created_minute) && s.name.contains("scope-1")),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_scopes_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=protocol", "order_by"),
                ("unknown=1", "unknown"),
                ("default_scope_type=MANDATORY", "default_scope_type"),
                ("has_protocol_mappers=maybe", "has_protocol_mappers"),
                ("not_assigned_to_client=nope", "not_assigned_to_client"),
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
