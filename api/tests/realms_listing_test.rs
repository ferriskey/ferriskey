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
    const CALLER_PASSWORD: &str = "L1ster-Realm-Pw!";
    const SEED_COUNT: usize = 25;
    const MANAGE_REALM: i64 = 1 << 5;
    const QUERY_REALMS: i64 = 1 << 10;
    const VIEW_REALM: i64 = 1 << 16;
    const VIEW_USERS: i64 = 1 << 17;
    const SORT_FIELDS: [&str; 3] = ["name", "created_at", "updated_at"];

    struct Seed {
        id: Uuid,
        name: String,
        display_name: Option<String>,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(prefix: &str, i: usize) -> Self {
            let display_name = match i % 3 {
                0 => None,
                1 => Some(format!("Tenant Alpha {i:02}")),
                _ => Some(format!("Tenant Beta {i:02}")),
            };
            Self {
                id: Uuid::new_v4(),
                name: format!("{prefix}-{i:02}"),
                display_name,
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

        fn permission(i: usize) -> i64 {
            match i % 3 {
                0 => VIEW_REALM,
                1 => QUERY_REALMS,
                _ => MANAGE_REALM,
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
        lister_token: String,
        witness_token: String,
        prefix: String,
        seeds: Vec<Seed>,
        hidden: Vec<Uuid>,
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

        let schema = format!("realms_listing_test_{}", Uuid::new_v4().simple());

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
        let master_id = realm_id_of(&pool, MASTER_REALM).await;

        let suffix = Uuid::new_v4().simple().to_string();
        let prefix = format!("lst-{}", &suffix[..8]);

        let lister =
            create_caller(&server, &admin_token, &format!("lister-{}", &suffix[..8])).await;
        let witness =
            create_caller(&server, &admin_token, &format!("witness-{}", &suffix[..8])).await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(|i| Seed::new(&prefix, i)).collect();
        for (i, seed) in seeds.iter().enumerate() {
            let client = insert_realm(&pool, master_id, seed).await;
            grant(&pool, master_id, client, &lister, Seed::permission(i)).await;
        }

        let mut hidden = Vec::new();
        for name in ["h1", "h2"] {
            let seed = plain_seed(&format!("{prefix}-{name}"), None);
            let client = insert_realm(&pool, master_id, &seed).await;
            grant(&pool, master_id, client, &witness, VIEW_REALM).await;
            hidden.push(seed.id);
        }
        let no_rights = plain_seed(&format!("{prefix}-n1"), None);
        let client = insert_realm(&pool, master_id, &no_rights).await;
        grant(&pool, master_id, client, &lister, VIEW_USERS).await;
        grant(&pool, master_id, client, &witness, VIEW_REALM).await;
        hidden.push(no_rights.id);

        for (name, display_name) in [
            ("pct-a", "50% off"),
            ("pct-b", "50x off"),
            ("und-a", "a_b"),
            ("und-b", "axb"),
            ("bsl-a", "back\\slash"),
            ("bsl-b", "backxslash"),
        ] {
            let seed = plain_seed(&format!("{prefix}-{name}"), Some(display_name));
            let client = insert_realm(&pool, master_id, &seed).await;
            grant(&pool, master_id, client, &witness, QUERY_REALMS).await;
        }

        let lister_token = direct_grant(&server, MASTER_REALM, &lister.1, CALLER_PASSWORD).await;
        let witness_token = direct_grant(&server, MASTER_REALM, &witness.1, CALLER_PASSWORD).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            lister_token,
            witness_token,
            prefix,
            seeds,
            hidden,
        }
    }

    fn plain_seed(name: &str, display_name: Option<&str>) -> Seed {
        Seed {
            id: Uuid::new_v4(),
            name: name.to_string(),
            display_name: display_name.map(str::to_string),
            created_minute: 0,
            updated_minute: 0,
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

    async fn insert_realm(pool: &PgPool, master_id: Uuid, seed: &Seed) -> Uuid {
        sqlx::query(
            "INSERT INTO realms (id, name, display_name, created_at, updated_at) \
             VALUES ($1::uuid, $2, $3, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $4), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $5))",
        )
        .bind(seed.id.to_string())
        .bind(&seed.name)
        .bind(seed.display_name.as_deref())
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert realm");

        let client_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $3, true, 'openid-connect', false, false, 'confidential', now(), now())",
        )
        .bind(client_id.to_string())
        .bind(master_id.to_string())
        .bind(format!("{}-realm", seed.name))
        .execute(pool)
        .await
        .expect("insert realm client");
        client_id
    }

    async fn grant(
        pool: &PgPool,
        master_id: Uuid,
        client_id: Uuid,
        caller: &(String, String),
        permissions: i64,
    ) {
        let role_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO roles (id, name, permissions, realm_id, client_id, created_at, updated_at) \
             VALUES ($1::uuid, $2, $3, $4::uuid, $5::uuid, now(), now())",
        )
        .bind(role_id.to_string())
        .bind(format!("access-{}", role_id.simple()))
        .bind(permissions)
        .bind(master_id.to_string())
        .bind(client_id.to_string())
        .execute(pool)
        .await
        .expect("insert role");
        sqlx::query(
            "INSERT INTO user_role (user_id, role_id, created_at, updated_at) VALUES ($1::uuid, $2::uuid, now(), now())",
        )
        .bind(&caller.0)
        .bind(role_id.to_string())
        .execute(pool)
        .await
        .expect("insert user role");
    }

    async fn create_caller(server: &TestServer, token: &str, username: &str) -> (String, String) {
        let id = create_user(server, token, MASTER_REALM, username).await;
        set_password(server, token, MASTER_REALM, &id, CALLER_PASSWORD).await;
        (id, username.to_string())
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

    async fn list(server: &TestServer, token: &str, query: &str) -> TestResponse {
        server
            .get(&format!("/realms/{MASTER_REALM}/users/@me/realms?{query}"))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok(server: &TestServer, token: &str, query: &str) -> Value {
        let response = list(server, token, query).await;
        assert_eq!(
            response.status_code(),
            200,
            "listing realms with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn lister_list(server: &TestServer, query: &str) -> Value {
        list_ok(server, &ctx().lister_token, query).await
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|realm| Uuid::parse_str(realm["id"].as_str().expect("realm id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|realm| realm["name"].as_str().expect("realm name").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_realms() {
        let server = make_server();
        rt().block_on(async {
            let body = lister_list(&server, "").await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn every_sort_field_orders_both_ways_without_gaps_or_duplicates() {
        let server = make_server();
        rt().block_on(async {
            for field in SORT_FIELDS {
                for (order, ascending) in [("asc", true), ("desc", false)] {
                    let query = format!("order_by={field}&order={order}");
                    let first = lister_list(&server, &query).await;
                    let second = lister_list(&server, &format!("{query}&page=2")).await;

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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_realms() {
        let server = make_server();
        let prefix = ctx().prefix.to_uppercase();
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                format!("name={prefix}-1"),
                matching(|s| s.name.contains(&format!("{}-1", ctx().prefix))),
            ),
            (
                "display_name=alpha".to_string(),
                matching(|s| {
                    s.display_name
                        .as_deref()
                        .is_some_and(|d| d.contains("Alpha"))
                }),
            ),
            (
                "display_name=tenant".to_string(),
                matching(|s| s.display_name.is_some()),
            ),
            (
                format!("name={prefix}-1&display_name=beta"),
                matching(|s| {
                    s.name.contains(&format!("{}-1", ctx().prefix))
                        && s.display_name
                            .as_deref()
                            .is_some_and(|d| d.contains("Beta"))
                }),
            ),
            ("name=".to_string(), matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "name=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = lister_list(&server, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn search_matches_the_name_or_the_display_name() {
        let server = make_server();
        let prefix = &ctx().prefix;
        let searched = |s: &Seed, needle: &str| {
            s.name.to_lowercase().contains(needle)
                || s.display_name
                    .as_deref()
                    .is_some_and(|d| d.to_lowercase().contains(needle))
        };
        let cases: Vec<(String, String, HashSet<Uuid>)> = vec![
            (
                format!("{}-1", prefix.to_uppercase()),
                format!("{prefix}-1"),
                matching(|s| s.name.contains(&format!("{prefix}-1"))),
            ),
            (
                "alpha".to_string(),
                "alpha".to_string(),
                matching(|s| {
                    s.display_name
                        .as_deref()
                        .is_some_and(|d| d.contains("Alpha"))
                }),
            ),
        ];

        rt().block_on(async {
            for (value, needle, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{value}: the fixture must make this search discriminating"
                );
                assert_eq!(
                    matching(|s| searched(s, &needle)),
                    expected,
                    "{value}: only one field may carry the needle"
                );
                let body = lister_list(&server, &format!("search={value}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{value}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{value}: total");
            }

            let combined = lister_list(
                &server,
                &format!("search={prefix}-1&display_name=beta&limit=100"),
            )
            .await;
            let expected = matching(|s| {
                s.name.contains(&format!("{prefix}-1"))
                    && s.display_name
                        .as_deref()
                        .is_some_and(|d| d.contains("Beta"))
            });
            let found: HashSet<Uuid> = ids(&combined).into_iter().collect();
            assert_eq!(found, expected);
            assert_eq!(total(&combined), expected.len() as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn search_matches_like_wildcards_literally() {
        let server = make_server();
        let token = &ctx().witness_token;
        rt().block_on(async {
            for (value, name) in [("%25", "pct-a"), ("_", "und-a"), ("%5C", "bsl-a")] {
                let body = list_ok(&server, token, &format!("search={value}")).await;
                assert_eq!(
                    names(&body),
                    [format!("{}-{name}", ctx().prefix)],
                    "{value}"
                );
                assert_eq!(total(&body), 1, "{value}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn realms_outside_the_callers_rights_are_never_listed() {
        let server = make_server();
        let prefix = &ctx().prefix;
        rt().block_on(async {
            for name in ["h1", "h2", "n1"] {
                let witness = list_ok(
                    &server,
                    &ctx().witness_token,
                    &format!("name={prefix}-{name}"),
                )
                .await;
                assert_eq!(
                    names(&witness),
                    [format!("{prefix}-{name}")],
                    "witness {name}"
                );
                assert_eq!(total(&witness), 1);

                let body = lister_list(&server, &format!("name={prefix}-{name}")).await;
                assert!(ids(&body).is_empty(), "{name}: {body}");
                assert_eq!(total(&body), 0, "{name}");
            }

            let everything = lister_list(&server, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            let listed: HashSet<Uuid> = ids(&everything).into_iter().collect();
            assert_eq!(listed, matching(|_| true));
            assert!(ctx().hidden.iter().all(|id| !listed.contains(id)));
            assert!(!names(&everything).iter().any(|name| name == MASTER_REALM));

            let witness_everything = list_ok(&server, &ctx().witness_token, "limit=100").await;
            assert_eq!(total(&witness_everything), 9);
            assert!(
                ids(&witness_everything)
                    .iter()
                    .all(|id| !matching(|_| true).contains(id))
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        let token = &ctx().witness_token;
        rt().block_on(async {
            let witness = list_ok(&server, token, "display_name=50x").await;
            assert_eq!(names(&witness), [format!("{}-pct-b", ctx().prefix)]);
            let percent = list_ok(&server, token, "display_name=%25").await;
            assert_eq!(names(&percent), [format!("{}-pct-a", ctx().prefix)]);
            assert_eq!(total(&percent), 1);

            let witness = list_ok(&server, token, "display_name=axb").await;
            assert_eq!(names(&witness), [format!("{}-und-b", ctx().prefix)]);
            let underscore = list_ok(&server, token, "display_name=_").await;
            assert_eq!(names(&underscore), [format!("{}-und-a", ctx().prefix)]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, token, "display_name=backx").await;
            assert_eq!(names(&witness), [format!("{}-bsl-b", ctx().prefix)]);
            let backslash = list_ok(&server, token, "display_name=%5C").await;
            assert_eq!(names(&backslash), [format!("{}-bsl-a", ctx().prefix)]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realms_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=display_name", "order_by"),
                ("unknown=1", "unknown"),
                ("name=a&name=b", "name"),
                ("search=a&search=b", "search"),
                ("created_from=2026-01-01T00:00:00Z", "created_from"),
            ] {
                let response = list(&server, &ctx().lister_token, query).await;
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
