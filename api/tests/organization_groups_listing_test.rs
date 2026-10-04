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
    const SORT_FIELDS: [&str; 3] = ["name", "created_at", "updated_at"];
    const DESCRIPTIONS: [&str; 3] = ["Platform team", "sales desk", "Support crew"];

    struct Seed {
        id: Uuid,
        parent: Option<usize>,
        name: String,
        description: Option<String>,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let parent = match i {
                0 | 1 => None,
                n if n < 20 => Some(n % 2),
                n => Some(2 + n % 2),
            };
            let description = if i % 5 == 4 {
                None
            } else {
                Some(format!("{} {i:02}", DESCRIPTIONS[i % 3]))
            };
            Self {
                id: Uuid::new_v4(),
                parent,
                name: format!("grp-{:02}", i.div_ceil(2)),
                description,
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
        organization_id: Uuid,
        sibling_organization_id: Uuid,
        sibling_parent_id: Uuid,
        foreign_organization_id: Uuid,
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
            "organization_groups_listing_test_{}",
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
        let realm = format!("groups-{}", &suffix[..8]);
        let other_realm = format!("other-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let organization_id = insert_organization(&pool, realm_id, "acme").await;
        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, organization_id, &seeds, seed).await;
        }

        let sibling_organization_id = insert_organization(&pool, realm_id, "globex").await;
        let sibling_parent_id = insert_group(&pool, sibling_organization_id, None, "grp-50").await;
        insert_group(
            &pool,
            sibling_organization_id,
            Some(sibling_parent_id),
            "grp-51",
        )
        .await;

        let foreign_organization_id = insert_organization(&pool, other_realm_id, "initech").await;
        for name in [
            "grp-52",
            "pct%grp",
            "pctxgrp",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_group(&pool, foreign_organization_id, None, name).await;
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
            organization_id,
            sibling_organization_id,
            sibling_parent_id,
            foreign_organization_id,
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

    async fn insert_organization(pool: &PgPool, realm_id: Uuid, alias: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organizations (id, realm_id, name, alias, domain, enabled, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $3, NULL, true, now(), now())",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(alias)
        .execute(pool)
        .await
        .expect("insert organization");
        id
    }

    async fn insert_seed(pool: &PgPool, organization_id: Uuid, seeds: &[Seed], seed: &Seed) {
        sqlx::query(
            "INSERT INTO organization_groups (id, organization_id, parent_group_id, name, description, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $6), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $7))",
        )
        .bind(seed.id.to_string())
        .bind(organization_id.to_string())
        .bind(seed.parent.map(|parent| seeds[parent].id.to_string()))
        .bind(&seed.name)
        .bind(seed.description.as_deref())
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert group");
    }

    async fn insert_group(
        pool: &PgPool,
        organization_id: Uuid,
        parent: Option<Uuid>,
        name: &str,
    ) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organization_groups (id, organization_id, parent_group_id, name, description, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, 'Platform team', now(), now())",
        )
        .bind(id.to_string())
        .bind(organization_id.to_string())
        .bind(parent.map(|p| p.to_string()))
        .bind(name)
        .execute(pool)
        .await
        .expect("insert plain group");
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
                "description": "organization groups listing fixture",
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
        organization_id: Uuid,
        query: &str,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{realm}/organizations/{organization_id}/groups?{query}"
            ))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok(
        server: &TestServer,
        realm: &str,
        organization_id: Uuid,
        query: &str,
    ) -> Value {
        let response = list(server, &ctx().admin_token, realm, organization_id, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing groups of {organization_id} in {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn list_main(server: &TestServer, query: &str) -> Value {
        list_ok(server, &ctx().realm, ctx().organization_id, query).await
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|group| Uuid::parse_str(group["id"].as_str().expect("group id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|group| group["name"].as_str().expect("name").to_string())
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

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_groups() {
        let server = make_server();
        rt().block_on(async {
            let body = list_main(&server, "").await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = list_main(&server, &query).await;
                    let second = list_main(&server, &format!("{query}&page=2")).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_groups() {
        let server = make_server();
        let seeds = &ctx().seeds;
        let root_a = seeds[0].id;
        let root_b = seeds[1].id;
        let middle = seeds[2].id;
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "name=GRP-1".to_string(),
                matching(|s| s.name.contains("grp-1")),
            ),
            (
                "description=TEAM".to_string(),
                matching(|s| description_contains(s, "team")),
            ),
            (
                "description=desk".to_string(),
                matching(|s| description_contains(s, "desk")),
            ),
            (
                format!("parent_group_id={root_a}"),
                matching(|s| s.parent == Some(0)),
            ),
            (
                format!("parent_group_id={middle}"),
                matching(|s| s.parent == Some(2)),
            ),
            ("is_root=true".to_string(), matching(|s| s.parent.is_none())),
            (
                "is_root=false".to_string(),
                matching(|s| s.parent.is_some()),
            ),
            (
                format!("name=grp-0&parent_group_id={root_b}"),
                matching(|s| s.name.contains("grp-0") && s.parent == Some(1)),
            ),
            ("name=".to_string(), matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "name=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_main(&server, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn groups_of_another_organization_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let sibling = ctx().sibling_organization_id;
            let parent = ctx().sibling_parent_id;
            let witness = list_ok(&server, &ctx().realm, sibling, "name=grp-5").await;
            assert_eq!(
                total(&witness),
                2,
                "the sibling organization holds grp-5x rows"
            );
            let witness = list_ok(
                &server,
                &ctx().realm,
                sibling,
                &format!("parent_group_id={parent}"),
            )
            .await;
            assert_eq!(names(&witness), ["grp-51"]);

            let body = list_main(&server, "name=grp-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let body = list_main(&server, &format!("parent_group_id={parent}")).await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_main(&server, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert_eq!(
                ids(&everything).into_iter().collect::<HashSet<_>>(),
                matching(|_| true)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn an_organization_outside_the_realm_is_not_found() {
        let server = make_server();
        rt().block_on(async {
            let foreign = ctx().foreign_organization_id;
            let witness = list_ok(&server, &ctx().other_realm, foreign, "name=grp-52").await;
            assert_eq!(names(&witness), ["grp-52"]);

            let response = list(&server, &ctx().admin_token, &ctx().realm, foreign, "").await;
            assert_eq!(response.status_code(), 404, "{}", response.text());
            assert!(!response.text().contains("grp-52"), "{}", response.text());

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn a_caller_without_rights_on_the_realm_lists_nothing() {
        let server = make_server();
        rt().block_on(async {
            let own = list(
                &server,
                &ctx().viewer_token,
                &ctx().other_realm,
                ctx().foreign_organization_id,
                "",
            )
            .await;
            assert_eq!(own.status_code(), 200, "{}", own.text());
            let own: Value = own.json();
            assert!(total(&own) > 0);

            let foreign = list(
                &server,
                &ctx().viewer_token,
                &ctx().realm,
                ctx().organization_id,
                "",
            )
            .await;
            assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
            let foreign = foreign.text();
            assert!(!foreign.contains("grp-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().other_realm;
            let foreign = ctx().foreign_organization_id;
            let witness = list_ok(&server, realm, foreign, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxgrp"]);
            let witness = list_ok(&server, realm, foreign, "name=underx").await;
            assert_eq!(names(&witness), ["underxscore"]);

            let percent = list_ok(&server, realm, foreign, "name=%25").await;
            assert_eq!(names(&percent), ["pct%grp"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, realm, foreign, "name=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, realm, foreign, "name=backx").await;
            assert_eq!(names(&witness), ["backxslash"]);
            let backslash = list_ok(&server, realm, foreign, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);

            let description_witness = list_main(&server, "description=team&limit=100").await;
            assert!(total(&description_witness) > 0);
            let description_percent = list_main(&server, "description=%25").await;
            assert_eq!(total(&description_percent), 0, "{description_percent}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn ids_select_exactly_the_given_groups_of_the_organization() {
        let server = make_server();
        rt().block_on(async {
            let seeds = &ctx().seeds;
            let witness = list_main(&server, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            let wanted = [seeds[1].id, seeds[7].id, seeds[22].id];
            assert!(wanted.iter().all(|id| listed.contains(id)));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let sibling = ctx().sibling_parent_id;
            let sibling_witness = list_ok(
                &server,
                &ctx().realm,
                ctx().sibling_organization_id,
                &format!("ids={sibling}"),
            )
            .await;
            assert_eq!(ids(&sibling_witness), [sibling]);
            let joined = wanted
                .iter()
                .chain([&sibling])
                .map(Uuid::to_string)
                .collect::<Vec<_>>()
                .join(",");

            let body = list_main(&server, &format!("ids={joined}")).await;
            let found: HashSet<Uuid> = ids(&body).into_iter().collect();
            assert_eq!(found, wanted.into_iter().collect::<HashSet<_>>());
            assert_eq!(total(&body), 3);

            let narrowed = list_main(&server, &format!("ids={joined}&is_root=false")).await;
            assert_eq!(
                ids(&narrowed).into_iter().collect::<HashSet<_>>(),
                [seeds[7].id, seeds[22].id]
                    .into_iter()
                    .collect::<HashSet<_>>()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
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
                ctx().organization_id,
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
                ctx().organization_id,
                &format!("ids={}", too_many.join(",")),
            )
            .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert!(response.text().contains("ids"), "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_groups_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=description", "order_by"),
                ("unknown=1", "unknown"),
                ("is_root=maybe", "is_root"),
                ("parent_group_id=not-a-uuid", "parent_group_id"),
                ("ids=not-a-uuid", "ids"),
                ("name=a&name=b", "name"),
            ] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().organization_id,
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
