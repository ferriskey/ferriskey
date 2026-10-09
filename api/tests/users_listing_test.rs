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
    const FIRSTNAMES: [&str; 5] = ["alice", "bob", "carol", "dave", "erin"];
    const SORT_FIELDS: [&str; 7] = [
        "username",
        "email",
        "firstname",
        "lastname",
        "enabled",
        "created_at",
        "updated_at",
    ];

    struct Seed {
        id: Uuid,
        username: String,
        email: String,
        firstname: &'static str,
        lastname: String,
        enabled: bool,
        email_verified: bool,
        service_account: bool,
        has_role: bool,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let domain = if i.is_multiple_of(2) {
                "example.org"
            } else {
                "corp.test"
            };
            Self {
                id: Uuid::new_v4(),
                username: format!("user-{i:02}"),
                email: format!("mail{:02}@{domain}", (i * 7) % SEED_COUNT),
                firstname: FIRSTNAMES[i % FIRSTNAMES.len()],
                lastname: format!("last{:02}", SEED_COUNT - 1 - i),
                enabled: !i.is_multiple_of(4),
                email_verified: i.is_multiple_of(2),
                service_account: i == 3 || i == 8,
                has_role: i.is_multiple_of(5),
                created_minute: i32::try_from(i / 3).expect("small index"),
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "username" => SortKey::Text(self.username.clone()),
                "email" => SortKey::Text(self.email.clone()),
                "firstname" => SortKey::Text(self.firstname.to_string()),
                "lastname" => SortKey::Text(self.lastname.clone()),
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
        role_id: String,
        seeds: Vec<Seed>,
        foreign_user_id: Uuid,
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

        let schema = format!("users_listing_test_{}", Uuid::new_v4().simple());

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

        let existing: i64 =
            sqlx::query_scalar("SELECT count(*) FROM users WHERE realm_id = $1::uuid")
                .bind(realm_id.to_string())
                .fetch_one(&pool)
                .await
                .expect("count users");
        assert_eq!(existing, 0, "a fresh realm must start without users");

        let role_id = create_role(
            &server,
            &admin_token,
            &realm,
            "listed-role",
            &["view_users"],
        )
        .await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, seed).await;
            if seed.has_role {
                link_role(&pool, seed.id, &role_id).await;
            }
        }

        let foreign_user_id = insert_plain_user(&pool, other_realm_id, "user-50").await;
        for username in [
            "user-51",
            "user-52",
            "pct%user",
            "pctxuser",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_plain_user(&pool, other_realm_id, username).await;
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
            &["view_users"],
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
            role_id,
            seeds,
            foreign_user_id,
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

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        let client_id = if seed.service_account {
            let client_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO clients (id, realm_id, name, client_id, enabled, protocol, public_client, service_account_enabled, client_type, created_at, updated_at) \
                 VALUES ($1::uuid, $2::uuid, $3, $3, true, 'openid-connect', false, true, 'confidential', now(), now())",
            )
            .bind(client_id.to_string())
            .bind(realm_id.to_string())
            .bind(format!("client-{}", seed.username))
            .execute(pool)
            .await
            .expect("insert client");
            Some(client_id.to_string())
        } else {
            None
        };

        sqlx::query(
            "INSERT INTO users (id, realm_id, client_id, username, firstname, lastname, email, email_verified, enabled, created_at, updated_at, failed_login_attempts) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, $4, $5, $6, $7, $8, $9, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $10), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $11), 0)",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(client_id)
        .bind(&seed.username)
        .bind(seed.firstname)
        .bind(&seed.lastname)
        .bind(&seed.email)
        .bind(seed.email_verified)
        .bind(seed.enabled)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .execute(pool)
        .await
        .expect("insert user");
    }

    async fn insert_plain_user(pool: &PgPool, realm_id: Uuid, username: &str) -> Uuid {
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
        .expect("insert plain user");
        id
    }

    async fn link_role(pool: &PgPool, user_id: Uuid, role_id: &str) {
        sqlx::query(
            "INSERT INTO user_role (user_id, role_id, created_at, updated_at) VALUES ($1::uuid, $2::uuid, now(), now())",
        )
        .bind(user_id.to_string())
        .bind(role_id)
        .execute(pool)
        .await
        .expect("insert user role");
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
                "description": "users listing fixture",
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
            .get(&format!("/realms/{realm}/users?{query}"))
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
            .map(|user| Uuid::parse_str(user["id"].as_str().expect("user id")).expect("uuid"))
            .collect()
    }

    fn usernames(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|user| user["username"].as_str().expect("username").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_users() {
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_users() {
        let server = make_server();
        let role_id = ctx().role_id.clone();
        let cases: Vec<(String, HashSet<Uuid>)> = vec![
            (
                "username=USER-1".to_string(),
                matching(|s| s.username.contains("user-1")),
            ),
            (
                "email=corp".to_string(),
                matching(|s| s.email.contains("corp")),
            ),
            (
                "firstname=AR".to_string(),
                matching(|s| s.firstname.contains("ar")),
            ),
            (
                "lastname=last0".to_string(),
                matching(|s| s.lastname.contains("last0")),
            ),
            ("enabled=true".to_string(), matching(|s| s.enabled)),
            ("enabled=false".to_string(), matching(|s| !s.enabled)),
            (
                "email_verified=true".to_string(),
                matching(|s| s.email_verified),
            ),
            (
                "email_verified=false".to_string(),
                matching(|s| !s.email_verified),
            ),
            (
                "service_account=true".to_string(),
                matching(|s| s.service_account),
            ),
            (
                "service_account=false".to_string(),
                matching(|s| !s.service_account),
            ),
            (format!("role_id={role_id}"), matching(|s| s.has_role)),
            (
                "username=user-1&enabled=true".to_string(),
                matching(|s| s.username.contains("user-1") && s.enabled),
            ),
            ("username=".to_string(), matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "username=",
                    "{query}: the fixture must make this filter discriminating"
                );
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{query}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{query}: total");
            }
        });
    }

    fn searched(seed: &Seed, needle: &str) -> bool {
        [
            seed.username.as_str(),
            seed.email.as_str(),
            seed.firstname,
            seed.lastname.as_str(),
        ]
        .iter()
        .any(|value| value.to_lowercase().contains(needle))
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn search_matches_any_of_the_text_fields() {
        let server = make_server();
        let cases: [(&str, &str, HashSet<Uuid>); 4] = [
            (
                "USER-1",
                "user-1",
                matching(|s| s.username.contains("user-1")),
            ),
            ("corp", "corp", matching(|s| s.email.contains("corp"))),
            ("AR", "ar", matching(|s| s.firstname.contains("ar"))),
            ("last0", "last0", matching(|s| s.lastname.contains("last0"))),
        ];

        rt().block_on(async {
            for (value, needle, expected) in cases {
                assert!(
                    !expected.is_empty() && expected.len() < SEED_COUNT,
                    "{value}: the fixture must make this search discriminating"
                );
                assert_eq!(
                    matching(|s| searched(s, needle)),
                    expected,
                    "{value}: only one field may carry the needle"
                );
                let body =
                    list_ok(&server, &ctx().realm, &format!("search={value}&limit=100")).await;
                let found: HashSet<Uuid> = ids(&body).into_iter().collect();
                assert_eq!(found, expected, "{value}: rows");
                assert_eq!(total(&body), expected.len() as u64, "{value}: total");
            }

            let combined = list_ok(
                &server,
                &ctx().realm,
                "search=user-1&enabled=true&limit=100",
            )
            .await;
            let found: HashSet<Uuid> = ids(&combined).into_iter().collect();
            let expected = matching(|s| s.username.contains("user-1") && s.enabled);
            assert_eq!(found, expected);
            assert_eq!(total(&combined), expected.len() as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn search_matches_like_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            for (value, expected) in [
                ("%25", "pct%user"),
                ("_", "under_score"),
                ("%5C", "back\\slash"),
            ] {
                let body = list_ok(&server, &ctx().other_realm, &format!("search={value}")).await;
                assert_eq!(usernames(&body), [expected], "{value}");
                assert_eq!(total(&body), 1, "{value}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
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
                "created_from=2026-01-01T00:02:00Z&created_to=2026-01-01T00:05:00Z&search=alice",
                matching(|s| (2..5).contains(&s.created_minute) && s.firstname == "alice"),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn users_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "username=user-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds user-5x rows");

            let body = list_ok(&server, &ctx().realm, "username=user-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                usernames(&everything)
                    .iter()
                    .all(|name| name.starts_with("user-") && name.len() == 7)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
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
            assert!(!foreign.contains("user-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "username=pctx").await;
            assert_eq!(usernames(&witness), ["pctxuser"]);
            let witness = list_ok(&server, &ctx().other_realm, "username=underx").await;
            assert_eq!(usernames(&witness), ["underxscore"]);

            let percent = list_ok(&server, &ctx().other_realm, "username=%25").await;
            assert_eq!(usernames(&percent), ["pct%user"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "username=_").await;
            assert_eq!(usernames(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_ok(&server, &ctx().other_realm, "username=backx").await;
            assert_eq!(usernames(&witness), ["backxslash"]);
            let backslash = list_ok(&server, &ctx().other_realm, "username=%5C").await;
            assert_eq!(usernames(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn ids_select_exactly_the_given_users_of_the_realm() {
        let server = make_server();
        rt().block_on(async {
            let seeds = &ctx().seeds;
            let witness = list_ok(&server, &ctx().realm, "limit=100").await;
            let listed: HashSet<Uuid> = ids(&witness).into_iter().collect();
            let wanted = [seeds[1].id, seeds[7].id, seeds[22].id];
            assert!(wanted.iter().all(|id| listed.contains(id)));
            assert_eq!(total(&witness), SEED_COUNT as u64);

            let foreign = ctx().foreign_user_id;
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test users_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=password", "order_by"),
                ("unknown=1", "unknown"),
                ("enabled=maybe", "enabled"),
                ("role_id=not-a-uuid", "role_id"),
                ("ids=not-a-uuid", "ids"),
                ("username=a&username=b", "username"),
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
