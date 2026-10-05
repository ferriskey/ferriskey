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
    const SORT_FIELDS: [&str; 5] = ["name", "alias", "enabled", "created_at", "updated_at"];

    struct Seed {
        id: Uuid,
        name: String,
        alias: String,
        domain: Option<String>,
        description: Option<String>,
        enabled: bool,
        member: bool,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let domain = match i % 5 {
                4 => None,
                n if n % 2 == 0 => Some(format!("d{n}.example.org")),
                n => Some(format!("d{n}.corp.test")),
            };
            let description = match i % 4 {
                0 => None,
                1 => Some(format!("Sales team {i:02}")),
                2 => Some(format!("Support 100% remote {i:02}")),
                _ => Some(format!("support_desk {i:02}")),
            };
            Self {
                id: Uuid::new_v4(),
                name: format!("org-{:02}", i / 2),
                alias: format!("alias-{:02}", (i * 7) % SEED_COUNT),
                domain,
                description,
                enabled: !i.is_multiple_of(4),
                member: i.is_multiple_of(3),
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "name" => SortKey::Text(self.name.clone()),
                "alias" => SortKey::Text(self.alias.clone()),
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
        member_id: Uuid,
        foreign_member_id: Uuid,
        seeds: Vec<Seed>,
        foreign_organization_id: Uuid,
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

        let schema = format!("organizations_listing_test_{}", Uuid::new_v4().simple());

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
            sqlx::query_scalar("SELECT count(*) FROM organizations WHERE realm_id = $1::uuid")
                .bind(realm_id.to_string())
                .fetch_one(&pool)
                .await
                .expect("count organizations");
        assert_eq!(
            existing, 0,
            "a fresh realm must start without organizations"
        );

        let member_id = insert_user(&pool, realm_id, "org-member").await;
        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_organization(&pool, realm_id, seed).await;
            if seed.member {
                link_member(&pool, seed.id, member_id).await;
            }
        }

        let foreign_member_id = insert_user(&pool, other_realm_id, "foreign-member").await;
        let foreign_organization_id =
            insert_plain_organization(&pool, other_realm_id, "org-50", "foreign-50").await;
        link_member(&pool, foreign_organization_id, foreign_member_id).await;
        for (index, name) in [
            "org-51",
            "org-52",
            "pct%org",
            "pctxorg",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ]
        .into_iter()
        .enumerate()
        {
            insert_plain_organization(&pool, other_realm_id, name, &format!("foreign-{index}"))
                .await;
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
            &["view_organizations"],
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
            member_id,
            foreign_member_id,
            seeds,
            foreign_organization_id,
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

    async fn insert_organization(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias, domain, description, enabled, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, $9, $6, \
             TIMESTAMPTZ '2026-01-01 00:00:00+00' + make_interval(mins => $7), \
             TIMESTAMPTZ '2026-01-01 00:00:00+00' + make_interval(mins => $8))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.name)
        .bind(&seed.alias)
        .bind(seed.domain.as_deref())
        .bind(seed.enabled)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .bind(seed.description.as_deref())
        .execute(pool)
        .await
        .expect("insert organization");
    }

    async fn insert_plain_organization(
        pool: &PgPool,
        realm_id: Uuid,
        name: &str,
        alias: &str,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias, domain, enabled, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, 'd0.corp.test', true, now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .bind(alias)
        .execute(pool)
        .await
        .expect("insert plain organization");
        id
    }

    async fn insert_user(pool: &PgPool, realm_id: Uuid, username: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, email_verified, enabled, created_at, updated_at, failed_login_attempts) \
             VALUES ($1::uuid, $2::uuid, $3, false, true, now(), now(), 0)",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(username)
        .execute(pool)
        .await
        .expect("insert user");
        id
    }

    async fn link_member(pool: &PgPool, organization_id: Uuid, user_id: Uuid) {
        sqlx::query(
            "INSERT INTO organization_members (id, organization_id, user_id, created_at) VALUES ($1::uuid, $2::uuid, $3::uuid, now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(organization_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await
        .expect("insert organization member");
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
                "description": "organizations listing fixture",
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
            .get(&format!("/realms/{realm}/organizations?{query}"))
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

    fn ids(body: &Value) -> Vec<Uuid> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|org| Uuid::parse_str(org["id"].as_str().expect("organization id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|org| org["name"].as_str().expect("name").to_string())
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

    fn description_contains(seed: &Seed, needle: &str) -> bool {
        seed.description
            .as_deref()
            .is_some_and(|description| description.to_lowercase().contains(needle))
    }

    fn created_at(minute: i32) -> String {
        format!("2026-01-01T00:{minute:02}:00Z")
    }

    fn created_within(from: Option<i32>, to: Option<i32>) -> HashSet<Uuid> {
        matching(|s| {
            from.is_none_or(|from| s.created_minute >= from)
                && to.is_none_or(|to| s.created_minute < to)
        })
    }

    fn domain_contains(seed: &Seed, needle: &str) -> bool {
        seed.domain
            .as_deref()
            .is_some_and(|domain| domain.contains(needle))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_organizations() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_organizations() {
        let server = make_server();
        let member_id = ctx().member_id;
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "name=ORG-1".to_string(),
                matching(|s| s.name.contains("org-1")),
            ),
            (
                "alias=alias-0".to_string(),
                matching(|s| s.alias.contains("alias-0")),
            ),
            (
                "domain=CORP".to_string(),
                matching(|s| domain_contains(s, "corp")),
            ),
            (
                "domain=example".to_string(),
                matching(|s| domain_contains(s, "example")),
            ),
            ("enabled=true".to_string(), matching(|s| s.enabled)),
            ("enabled=false".to_string(), matching(|s| !s.enabled)),
            (
                "has_domain=true".to_string(),
                matching(|s| s.domain.is_some()),
            ),
            (
                "has_domain=false".to_string(),
                matching(|s| s.domain.is_none()),
            ),
            (
                "search=-1".to_string(),
                matching(|s| s.name.contains("-1") || s.alias.contains("-1")),
            ),
            (
                format!("without_member={member_id}"),
                matching(|s| !s.member),
            ),
            (
                "description=SUPPORT".to_string(),
                matching(|s| description_contains(s, "support")),
            ),
            (
                "description=team".to_string(),
                matching(|s| description_contains(s, "team")),
            ),
            (
                "description=e".to_string(),
                matching(|s| s.description.is_some()),
            ),
            (
                "description=support&enabled=true".to_string(),
                matching(|s| description_contains(s, "support") && s.enabled),
            ),
            (
                "name=org-1&enabled=true".to_string(),
                matching(|s| s.name.contains("org-1") && s.enabled),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn search_matches_the_name_or_the_alias() {
        let server = make_server();
        let by_name_only = ctx()
            .seeds
            .iter()
            .find(|s| s.name.contains("-1") && !s.alias.contains("-1"))
            .expect("a seed matching by name only");
        let by_alias_only = ctx()
            .seeds
            .iter()
            .find(|s| !s.name.contains("-1") && s.alias.contains("-1"))
            .expect("a seed matching by alias only");
        rt().block_on(async {
            let body = list_ok(&server, &ctx().realm, "search=-1&limit=100").await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            assert!(found.contains(&by_name_only.id));
            assert!(found.contains(&by_alias_only.id));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn search_matches_the_domain_and_the_description() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("search=CORP", matching(|s| domain_contains(s, "corp"))),
            (
                "search=example",
                matching(|s| domain_contains(s, "example")),
            ),
            (
                "search=SUPPORT",
                matching(|s| description_contains(s, "support")),
            ),
            ("search=team", matching(|s| description_contains(s, "team"))),
            ("search=%20", matching(|s| s.description.is_some())),
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
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn without_member_keeps_only_the_organizations_the_user_has_not_joined() {
        let server = make_server();
        rt().block_on(async {
            let member_id = ctx().member_id;
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let joined = matching(|s| s.member);
            assert!(!joined.is_empty());
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            assert!(joined.iter().all(|id| listed.contains(id)));

            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("without_member={member_id}&limit=100"),
            )
            .await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            assert!(found.is_disjoint(&joined), "{body}");
            assert_eq!(total(&body), (SEED_COUNT - joined.len()) as u64);

            let foreign_member_id = ctx().foreign_member_id;
            let everything = list_ok(
                &server,
                &ctx().realm,
                &format!("without_member={foreign_member_id}&limit=100"),
            )
            .await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert_eq!(
                ids(&everything).into_iter().collect::<HashSet<_>>(),
                matching(|_| true)
            );

            let foreign = ctx().foreign_organization_id;
            let witness = list_ok(&server, &ctx().other_realm, "limit=100").await;
            assert!(ids(&witness).contains(&foreign), "{witness}");
            let other_total = total(&witness);

            let excluded = list_ok(
                &server,
                &ctx().other_realm,
                &format!("without_member={foreign_member_id}&limit=100"),
            )
            .await;
            assert!(!ids(&excluded).contains(&foreign), "{excluded}");
            assert_eq!(total(&excluded), other_total - 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn organizations_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=org-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds org-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=org-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                names(&everything)
                    .iter()
                    .all(|name| name.starts_with("org-") && name.len() == 6)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
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
            assert!(!foreign.contains("org-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxorg"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxscore"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%org"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxslash"]);
            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);

            let domain_witness = list_ok(&server, &ctx().realm, "domain=corp&limit=100").await;
            assert!(total(&domain_witness) > 0);
            let domain_percent = list_ok(&server, &ctx().realm, "domain=%25").await;
            assert_eq!(total(&domain_percent), 0, "{domain_percent}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn ids_select_exactly_the_given_organizations_of_the_realm() {
        let server = make_server();
        rt().block_on(async {
            let seeds = &ctx().seeds;
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            let wanted = [seeds[1].id, seeds[7].id, seeds[22].id];
            assert!(wanted.iter().all(|id| listed.contains(id)));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign = ctx().foreign_organization_id;
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
            let expected: HashSet<Uuid> = [&seeds[1], &seeds[7], &seeds[22]]
                .into_iter()
                .filter(|seed| seed.enabled)
                .map(|seed| seed.id)
                .collect();
            assert_eq!(ids(&narrowed).into_iter().collect::<HashSet<_>>(), expected);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=domain", "order_by"),
                ("unknown=1", "unknown"),
                ("enabled=maybe", "enabled"),
                ("has_domain=maybe", "has_domain"),
                ("without_member=not-a-uuid", "without_member"),
                ("ids=not-a-uuid", "ids"),
                ("name=a&name=b", "name"),
                ("search=a&search=b", "search"),
                ("description=a&description=b", "description"),
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

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn description_matches_literally_and_never_matches_a_missing_description() {
        let server = make_server();
        rt().block_on(async {
            let percent = list_ok(&server, &ctx().realm, "description=%25&limit=100").await;
            let expected = matching(|s| description_contains(s, "%"));
            assert!(!expected.is_empty() && expected.len() < SEED_COUNT);
            assert_eq!(ids(&percent).into_iter().collect::<HashSet<_>>(), expected);
            assert_eq!(total(&percent), expected.len() as u64);

            let underscore = list_ok(&server, &ctx().realm, "description=_&limit=100").await;
            let expected = matching(|s| description_contains(s, "_"));
            assert_eq!(
                ids(&underscore).into_iter().collect::<HashSet<_>>(),
                expected
            );
            assert_eq!(total(&underscore), expected.len() as u64);

            let backslash = list_ok(&server, &ctx().realm, "description=%5C").await;
            assert_eq!(total(&backslash), 0, "{backslash}");

            let missing = matching(|s| s.description.is_none());
            assert!(!missing.is_empty());
            let any = list_ok(&server, &ctx().realm, "description=%20&limit=100").await;
            let found: HashSet<Uuid> = ids(&any).into_iter().collect();
            assert!(found.is_disjoint(&missing), "{any}");
            assert_eq!(total(&any), (SEED_COUNT - missing.len()) as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn search_matches_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().other_realm;
            let percent = list_ok(&server, realm, "search=%25").await;
            assert_eq!(names(&percent), ["pct%org"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, realm, "search=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let backslash = list_ok(&server, realm, "search=%5C").await;
            assert_eq!(names(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn created_range_is_inclusive_from_and_exclusive_to() {
        let server = make_server();
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                format!("created_from={}", created_at(5)),
                created_within(Some(5), None),
            ),
            (
                format!("created_to={}", created_at(2)),
                created_within(None, Some(2)),
            ),
            (
                format!(
                    "created_from={}&created_to={}",
                    created_at(2),
                    created_at(5)
                ),
                created_within(Some(2), Some(5)),
            ),
            (
                format!(
                    "created_from={}&created_to={}",
                    created_at(3),
                    created_at(4)
                ),
                created_within(Some(3), Some(4)),
            ),
            (
                "created_from=2026-01-01T02:05:00%2B02:00".to_string(),
                created_within(Some(5), None),
            ),
            (
                format!("created_to={}&search=-1", created_at(4)),
                matching(|s| {
                    s.created_minute < 4 && (s.name.contains("-1") || s.alias.contains("-1"))
                }),
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

            let at_bound = matching(|s| s.created_minute == 4);
            assert!(!at_bound.is_empty());
            let body = list_ok(
                &server,
                &ctx().realm,
                &format!("created_to={}&limit=100", created_at(4)),
            )
            .await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            assert!(found.is_disjoint(&at_bound), "{body}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organizations_listing_test -- --ignored"]
    fn an_inverted_created_range_is_an_empty_page() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(
                &server,
                &ctx().realm,
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
}
