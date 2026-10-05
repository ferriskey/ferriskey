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

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Pages {
        Empty,
        Complete,
        Nested,
        OneBlockShort,
    }

    impl Pages {
        fn activatable(self) -> bool {
            matches!(self, Pages::Complete | Pages::Nested)
        }
    }

    struct Requirement {
        page_type: String,
        blocks: Vec<String>,
    }

    struct Seed {
        id: Uuid,
        name: String,
        layout: Option<usize>,
        pages: Pages,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            Self {
                id: Uuid::new_v4(),
                name: format!("theme-{:02}", i / 2),
                layout: (i % 3 != 2).then_some(i % 3),
                pages: match i % 4 {
                    0 if i.is_multiple_of(8) => Pages::Nested,
                    0 => Pages::Complete,
                    1 => Pages::OneBlockShort,
                    _ => Pages::Empty,
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
        layouts: Vec<Uuid>,
        foreign_layout: Uuid,
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

        let schema = format!("portal_themes_listing_test_{}", Uuid::new_v4().simple());

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

        clear_themes(&pool, realm_id).await;
        clear_themes(&pool, other_realm_id).await;

        let requirements = page_requirements(&server, &admin_token, &realm).await;
        assert!(
            requirements.iter().any(|r| r.blocks.len() > 1),
            "the fixture needs a page with several required blocks"
        );

        let layouts = vec![
            insert_layout(&pool, realm_id, "layout-a").await,
            insert_layout(&pool, realm_id, "layout-b").await,
        ];
        let foreign_layout = insert_layout(&pool, other_realm_id, "layout-foreign").await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, &layouts, &requirements, seed).await;
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
            "tenant-manager",
            &["manage_realm"],
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

        for (name, layout) in [
            ("theme-50", Some(foreign_layout)),
            ("theme-51", Some(foreign_layout)),
            ("theme-52", None),
            ("pct%theme", None),
            ("pctxtheme", None),
            ("under_theme", None),
            ("underxtheme", None),
            ("back\\theme", None),
            ("backxtheme", None),
        ] {
            insert_plain_theme(&pool, other_realm_id, name, layout).await;
        }

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            realm,
            other_realm,
            layouts,
            foreign_layout,
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

    async fn clear_themes(pool: &PgPool, realm_id: Uuid) {
        sqlx::query("DELETE FROM portal_themes WHERE realm_id = $1::uuid")
            .bind(realm_id.to_string())
            .execute(pool)
            .await
            .expect("clear realm themes");
    }

    async fn page_requirements(server: &TestServer, token: &str, realm: &str) -> Vec<Requirement> {
        let response = server
            .get(&format!("/realms/{realm}/portal/page-requirements"))
            .add_header("Authorization", auth_header(token))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let body: Value = response.json();
        body["data"]
            .as_array()
            .expect("requirements array")
            .iter()
            .map(|entry| Requirement {
                page_type: entry["page_type"].as_str().expect("page type").to_string(),
                blocks: entry["required_blocks"]
                    .as_array()
                    .expect("required blocks")
                    .iter()
                    .map(|block| block.as_str().expect("block").to_string())
                    .collect(),
            })
            .collect()
    }

    fn page_tree(pages: Pages, blocks: &[String]) -> Value {
        let nodes: Vec<Value> = match pages {
            Pages::Empty => vec![],
            Pages::Complete => blocks.iter().map(|b| json!({ "type": b })).collect(),
            Pages::Nested => {
                return json!([{
                    "type": "container",
                    "children": [{
                        "type": "container",
                        "props": { "label": "nested" },
                        "children": blocks.iter().map(|b| json!({ "type": b })).collect::<Vec<_>>(),
                    }],
                }]);
            }
            Pages::OneBlockShort => {
                let mut nodes: Vec<Value> = blocks.iter().map(|b| json!({ "type": b })).collect();
                if let Some(last) = blocks.last() {
                    nodes.pop();
                    nodes.push(json!({ "type": "text", "content": last }));
                    nodes.push(json!({ "type": [last] }));
                }
                nodes
            }
        };
        Value::Array(nodes)
    }

    async fn insert_layout(pool: &PgPool, realm_id: Uuid, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO portal_layouts (id, realm_id, name) VALUES ($1::uuid, $2::uuid, $3)",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert layout");
        id
    }

    async fn insert_seed(
        pool: &PgPool,
        realm_id: Uuid,
        layouts: &[Uuid],
        requirements: &[Requirement],
        seed: &Seed,
    ) {
        sqlx::query(
            "INSERT INTO portal_themes (id, realm_id, name, layout_id, design_tokens, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4::uuid, '{}'::jsonb, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $5), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $6))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.name)
        .bind(seed.layout.map(|index| layouts[index].to_string()))
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert theme");

        let last_page_with_blocks = requirements
            .iter()
            .rposition(|r| !r.blocks.is_empty())
            .expect("a page with required blocks");
        for (index, requirement) in requirements.iter().enumerate() {
            let pages = match seed.pages {
                Pages::OneBlockShort if index != last_page_with_blocks => Pages::Complete,
                other => other,
            };
            sqlx::query(&format!(
                "UPDATE portal_themes SET page_{} = $1::jsonb WHERE id = $2::uuid",
                requirement.page_type
            ))
            .bind(page_tree(pages, &requirement.blocks).to_string())
            .bind(seed.id.to_string())
            .execute(pool)
            .await
            .expect("set page tree");
        }
    }

    async fn insert_plain_theme(pool: &PgPool, realm_id: Uuid, name: &str, layout: Option<Uuid>) {
        sqlx::query(
            "INSERT INTO portal_themes (id, realm_id, name, layout_id, design_tokens, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4::uuid, '{}'::jsonb, now(), now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .bind(layout.map(|id| id.to_string()))
        .execute(pool)
        .await
        .expect("insert plain theme");
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
                "lastname": "Manager",
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
                "description": "portal themes listing fixture",
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
            .get(&format!("/realms/{realm}/portal/themes?{query}"))
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
            .map(|theme| Uuid::parse_str(theme["id"].as_str().expect("theme id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|theme| theme["name"].as_str().expect("name").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_themes() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_themes() {
        let server = make_server();
        let layouts = &ctx().layouts;
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "name=THEME-1".to_string(),
                matching(|s| s.name.contains("theme-1")),
            ),
            ("name=0".to_string(), matching(|s| s.name.contains('0'))),
            (
                format!("layout_id={}", layouts[0]),
                matching(|s| s.layout == Some(0)),
            ),
            (
                format!("layout_id={}", layouts[1]),
                matching(|s| s.layout == Some(1)),
            ),
            (
                "activatable=true".to_string(),
                matching(|s| s.pages.activatable()),
            ),
            (
                "activatable=false".to_string(),
                matching(|s| !s.pages.activatable()),
            ),
            (
                format!("layout_id={}&activatable=true", layouts[0]),
                matching(|s| s.layout == Some(0) && s.pages.activatable()),
            ),
            (
                format!("name=theme-0&layout_id={}", layouts[1]),
                matching(|s| s.name.contains("theme-0") && s.layout == Some(1)),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn activatable_mirrors_the_activation_check() {
        let server = make_server();
        rt().block_on(async {
            let complete = ctx()
                .seeds
                .iter()
                .find(|s| s.pages == Pages::Complete)
                .expect("a complete seed");
            let short = ctx()
                .seeds
                .iter()
                .find(|s| s.pages == Pages::OneBlockShort)
                .expect("a seed one block short");
            assert!(ctx().seeds.iter().any(|s| s.pages == Pages::Nested));

            let activatable = list_ok(&server, &ctx().realm, "activatable=true&limit=100").await;
            let listed: HashSet<Uuid> = ids(&activatable).into_iter().collect();
            assert!(listed.contains(&complete.id));
            assert!(!listed.contains(&short.id));

            let accepted = server
                .post(&format!(
                    "/realms/{}/portal/themes/{}/activate",
                    ctx().realm,
                    complete.id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_eq!(accepted.status_code(), 200, "{}", accepted.text());

            let refused = server
                .post(&format!(
                    "/realms/{}/portal/themes/{}/activate",
                    ctx().realm,
                    short.id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_ne!(refused.status_code(), 200, "{}", refused.text());
            assert!(
                refused.status_code().is_client_error(),
                "{}",
                refused.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn listed_themes_keep_their_full_shape() {
        let server = make_server();
        rt().block_on(async {
            let body = list_ok(&server, &ctx().realm, "activatable=true&limit=1").await;
            let theme = rows(&body).first().expect("one theme");
            for key in [
                "id",
                "realm_id",
                "name",
                "layout_id",
                "config",
                "pages",
                "created_at",
                "updated_at",
            ] {
                assert!(theme.get(key).is_some(), "{key} missing: {theme}");
            }
            let login = &theme["pages"]["login"];
            assert!(
                login.as_array().is_some_and(|nodes| !nodes.is_empty()),
                "{theme}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn themes_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=theme-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds theme-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=theme-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let foreign = format!("layout_id={}", ctx().foreign_layout);
            let witness = list_ok(&server, &ctx().other_realm, &foreign).await;
            assert_eq!(total(&witness), 2);
            let body = list_ok(&server, &ctx().realm, &foreign).await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            let seeded: HashSet<Uuid> = ctx().seeds.iter().map(|seed| seed.id).collect();
            assert_eq!(ids(&everything).into_iter().collect::<HashSet<_>>(), seeded);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
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
            assert!(!foreign.contains("theme-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxtheme"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxtheme"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%theme"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_theme"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxtheme"]);
            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\theme"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test portal_themes_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=layout_id", "order_by"),
                ("order_by=config", "order_by"),
                ("unknown=1", "unknown"),
                ("layout_id=not-a-uuid", "layout_id"),
                ("activatable=maybe", "activatable"),
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
}
