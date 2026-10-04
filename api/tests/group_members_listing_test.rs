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
    const SORT_FIELDS: [&str; 3] = ["username", "email", "created_at"];

    struct Seed {
        id: Uuid,
        user_id: Uuid,
        username: String,
        email: String,
        enabled: bool,
        created_minute: i32,
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
                user_id: Uuid::new_v4(),
                username: format!("member-{i:02}"),
                email: format!("mail{:02}@{domain}", (i * 7) % SEED_COUNT),
                enabled: !i.is_multiple_of(4),
                created_minute: i32::try_from(i / 3).expect("small index"),
            }
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "username" => SortKey::Text(self.username.clone()),
                "email" => SortKey::Text(self.email.clone()),
                "created_at" => SortKey::Minute(self.created_minute),
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
        group_id: Uuid,
        other_group_id: Uuid,
        sibling_organization_id: Uuid,
        sibling_group_id: Uuid,
        foreign_organization_id: Uuid,
        foreign_group_id: Uuid,
        outsiders: [Uuid; 3],
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

        let schema = format!("group_members_listing_test_{}", Uuid::new_v4().simple());

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
        let realm = format!("members-{}", &suffix[..8]);
        let other_realm = format!("other-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let organization_id = insert_organization(&pool, realm_id, "acme").await;
        let group_id = insert_group(&pool, organization_id, "engineering").await;
        let other_group_id = insert_group(&pool, organization_id, "sales").await;

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for (i, seed) in seeds.iter().enumerate() {
            let user_minute = i32::try_from(SEED_COUNT - i).expect("small index");
            insert_user(
                &pool,
                realm_id,
                seed.user_id,
                &seed.username,
                Some(&seed.email),
                seed.enabled,
                user_minute,
            )
            .await;
            sqlx::query(
                "INSERT INTO organization_group_members (id, group_id, user_id, created_at) \
                 VALUES ($1::uuid, $2::uuid, $3::uuid, \
                 TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $4))",
            )
            .bind(seed.id.to_string())
            .bind(group_id.to_string())
            .bind(seed.user_id.to_string())
            .bind(seed.created_minute)
            .execute(&pool)
            .await
            .expect("insert seeded membership");
        }

        let sibling_organization_id = insert_organization(&pool, realm_id, "globex").await;
        let sibling_group_id = insert_group(&pool, sibling_organization_id, "support").await;

        let loner = insert_plain_user(&pool, realm_id, "member-90").await;
        let other_member = insert_plain_user(&pool, realm_id, "member-91").await;
        add_member(&pool, other_group_id, other_member).await;
        let sibling_member = insert_plain_user(&pool, realm_id, "member-92").await;
        add_member(&pool, sibling_group_id, sibling_member).await;

        let foreign_organization_id = insert_organization(&pool, other_realm_id, "initech").await;
        let foreign_group_id = insert_group(&pool, foreign_organization_id, "engineering").await;
        for username in [
            "foreign-50",
            "pct%user",
            "pctxuser",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            let user_id = insert_plain_user(&pool, other_realm_id, username).await;
            add_member(&pool, foreign_group_id, user_id).await;
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
            group_id,
            other_group_id,
            sibling_organization_id,
            sibling_group_id,
            foreign_organization_id,
            foreign_group_id,
            outsiders: [loner, other_member, sibling_member],
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

    async fn insert_group(pool: &PgPool, organization_id: Uuid, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO organization_groups (id, organization_id, parent_group_id, name, description, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, NULL, $3, NULL, now(), now())",
        )
        .bind(id.to_string())
        .bind(organization_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert group");
        id
    }

    async fn insert_user(
        pool: &PgPool,
        realm_id: Uuid,
        id: Uuid,
        username: &str,
        email: Option<&str>,
        enabled: bool,
        created_minute: i32,
    ) {
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, email, email_verified, enabled, created_at, updated_at, failed_login_attempts) \
             VALUES ($1::uuid, $2::uuid, $3, $4, false, $5, \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $6), \
             TIMESTAMP '2026-01-01 00:00:00' + make_interval(mins => $6), 0)",
        )
        .bind(id.to_string())
        .bind(realm_id.to_string())
        .bind(username)
        .bind(email)
        .bind(enabled)
        .bind(created_minute)
        .execute(pool)
        .await
        .expect("insert user");
    }

    async fn insert_plain_user(pool: &PgPool, realm_id: Uuid, username: &str) -> Uuid {
        let id = Uuid::new_v4();
        insert_user(pool, realm_id, id, username, None, true, 0).await;
        id
    }

    async fn add_member(pool: &PgPool, group_id: Uuid, user_id: Uuid) {
        sqlx::query(
            "INSERT INTO organization_group_members (id, group_id, user_id, created_at) \
             VALUES ($1::uuid, $2::uuid, $3::uuid, now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(group_id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await
        .expect("insert membership");
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
                "description": "group members listing fixture",
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
        group_id: Uuid,
        query: &str,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{realm}/organizations/{organization_id}/groups/{group_id}/members?{query}"
            ))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn list_ok(
        server: &TestServer,
        realm: &str,
        organization_id: Uuid,
        group_id: Uuid,
        query: &str,
    ) -> Value {
        let response = list(
            server,
            &ctx().admin_token,
            realm,
            organization_id,
            group_id,
            query,
        )
        .await;
        assert_eq!(
            response.status_code(),
            200,
            "listing members of {group_id} in {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    async fn list_main(server: &TestServer, query: &str) -> Value {
        list_ok(
            server,
            &ctx().realm,
            ctx().organization_id,
            ctx().group_id,
            query,
        )
        .await
    }

    async fn list_foreign(server: &TestServer, query: &str) -> Value {
        list_ok(
            server,
            &ctx().other_realm,
            ctx().foreign_organization_id,
            ctx().foreign_group_id,
            query,
        )
        .await
    }

    async fn users_ok(server: &TestServer, realm: &str, query: &str) -> Value {
        let response = server
            .get(&format!("/realms/{realm}/users?{query}"))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "listing users of {realm} with `{query}` failed: {}",
            response.text()
        );
        response.json()
    }

    fn ids(body: &Value) -> Vec<Uuid> {
        uuids(body, "id")
    }

    fn uuids(body: &Value, field: &str) -> Vec<Uuid> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|row| Uuid::parse_str(row[field].as_str().expect("uuid field")).expect("uuid"))
            .collect()
    }

    fn usernames(body: &Value) -> Vec<String> {
        body["data"]
            .as_array()
            .expect("data array")
            .iter()
            .map(|row| row["username"].as_str().expect("username").to_string())
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_members() {
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

            let first = &body["data"][0];
            let seed = ctx()
                .seeds
                .iter()
                .find(|seed| Some(seed.id.to_string().as_str()) == first["id"].as_str())
                .expect("a seeded membership");
            assert_eq!(first["user_id"], seed.user_id.to_string());
            assert_eq!(first["group_id"], ctx().group_id.to_string());
            assert_eq!(first["username"], seed.username);
            assert_eq!(first["email"], seed.email);
            assert_eq!(first["enabled"], seed.enabled);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_members() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            (
                "username=MEMBER-1",
                matching(|s| s.username.contains("member-1")),
            ),
            ("email=CORP", matching(|s| s.email.contains("corp"))),
            ("email=mail0", matching(|s| s.email.contains("mail0"))),
            ("enabled=true", matching(|s| s.enabled)),
            ("enabled=false", matching(|s| !s.enabled)),
            (
                "username=member-1&enabled=true",
                matching(|s| s.username.contains("member-1") && s.enabled),
            ),
            (
                "username=member-2&email=example",
                matching(|s| s.username.contains("member-2") && s.email.contains("example")),
            ),
            ("username=", matching(|_| true)),
        ];

        rt().block_on(async {
            for (query, expected) in cases {
                assert!(
                    (!expected.is_empty() && expected.len() < SEED_COUNT) || query == "username=",
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn members_of_another_group_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(
                &server,
                &ctx().realm,
                ctx().organization_id,
                ctx().other_group_id,
                "",
            )
            .await;
            assert_eq!(usernames(&witness), ["member-91"]);
            assert_eq!(total(&witness), 1);

            let body = list_main(&server, "username=member-9").await;
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn a_group_outside_the_organization_or_the_realm_is_not_found() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(
                &server,
                &ctx().realm,
                ctx().sibling_organization_id,
                ctx().sibling_group_id,
                "",
            )
            .await;
            assert_eq!(usernames(&witness), ["member-92"]);

            let token = &ctx().admin_token;
            let realm = &ctx().realm;
            let response = list(
                &server,
                token,
                realm,
                ctx().organization_id,
                ctx().sibling_group_id,
                "",
            )
            .await;
            assert_eq!(response.status_code(), 404, "{}", response.text());
            assert!(
                !response.text().contains("member-92"),
                "{}",
                response.text()
            );

            let foreign_witness = list_foreign(&server, "username=foreign").await;
            assert_eq!(usernames(&foreign_witness), ["foreign-50"]);

            for (organization_id, group_id) in [
                (ctx().foreign_organization_id, ctx().foreign_group_id),
                (ctx().organization_id, ctx().foreign_group_id),
                (ctx().organization_id, Uuid::new_v4()),
            ] {
                let response = list(&server, token, realm, organization_id, group_id, "").await;
                assert_eq!(
                    response.status_code(),
                    404,
                    "{organization_id}/{group_id}: {}",
                    response.text()
                );
                assert!(
                    !response.text().contains("foreign-50"),
                    "{}",
                    response.text()
                );
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn a_caller_without_rights_on_the_realm_lists_nothing() {
        let server = make_server();
        rt().block_on(async {
            let own = list(
                &server,
                &ctx().viewer_token,
                &ctx().other_realm,
                ctx().foreign_organization_id,
                ctx().foreign_group_id,
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
                ctx().group_id,
                "",
            )
            .await;
            assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
            let foreign = foreign.text();
            assert!(!foreign.contains("member-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_foreign(&server, "username=pctx").await;
            assert_eq!(usernames(&witness), ["pctxuser"]);
            let witness = list_foreign(&server, "username=underx").await;
            assert_eq!(usernames(&witness), ["underxscore"]);

            let percent = list_foreign(&server, "username=%25").await;
            assert_eq!(usernames(&percent), ["pct%user"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_foreign(&server, "username=_").await;
            assert_eq!(usernames(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let witness = list_foreign(&server, "username=backx").await;
            assert_eq!(usernames(&witness), ["backxslash"]);
            let backslash = list_foreign(&server, "username=%5C").await;
            assert_eq!(usernames(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);

            let email_witness = list_main(&server, "email=mail&limit=100").await;
            assert_eq!(total(&email_witness), SEED_COUNT as u64);
            let email_percent = list_main(&server, "email=%25").await;
            assert_eq!(total(&email_percent), 0, "{email_percent}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn the_users_listing_offers_only_the_users_outside_the_group() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().realm;
            let [loner, other_member, sibling_member] = ctx().outsiders;
            let seeded: HashSet<Uuid> = ctx().seeds.iter().map(|seed| seed.user_id).collect();
            let candidates: HashSet<Uuid> = seeded.iter().copied().chain(ctx().outsiders).collect();

            let witness = users_ok(&server, realm, "username=member-&limit=100").await;
            assert_eq!(
                uuids(&witness, "id").into_iter().collect::<HashSet<_>>(),
                candidates
            );

            let outside = users_ok(
                &server,
                realm,
                &format!("username=member-&limit=100&not_in_group={}", ctx().group_id),
            )
            .await;
            assert_eq!(
                uuids(&outside, "id").into_iter().collect::<HashSet<_>>(),
                [loner, other_member, sibling_member]
                    .into_iter()
                    .collect::<HashSet<_>>()
            );
            assert_eq!(total(&outside), 3);

            let outside_other = users_ok(
                &server,
                realm,
                &format!(
                    "username=member-&limit=100&not_in_group={}",
                    ctx().other_group_id
                ),
            )
            .await;
            let mut expected = candidates.clone();
            expected.remove(&other_member);
            assert_eq!(
                uuids(&outside_other, "id")
                    .into_iter()
                    .collect::<HashSet<_>>(),
                expected
            );

            let narrowed = users_ok(
                &server,
                realm,
                &format!(
                    "username=member-9&enabled=true&not_in_group={}",
                    ctx().sibling_group_id
                ),
            )
            .await;
            assert_eq!(
                uuids(&narrowed, "id").into_iter().collect::<HashSet<_>>(),
                [loner, other_member].into_iter().collect::<HashSet<_>>()
            );

            let foreign_group = users_ok(
                &server,
                realm,
                &format!(
                    "username=member-&limit=100&not_in_group={}",
                    ctx().foreign_group_id
                ),
            )
            .await;
            assert_eq!(total(&foreign_group), candidates.len() as u64);

            let other_realm = &ctx().other_realm;
            let foreign_witness = users_ok(&server, other_realm, "username=foreign-50").await;
            assert_eq!(total(&foreign_witness), 1);
            let foreign_outside = users_ok(
                &server,
                other_realm,
                &format!(
                    "username=foreign-50&not_in_group={}",
                    ctx().foreign_group_id
                ),
            )
            .await;
            assert_eq!(total(&foreign_outside), 0, "{foreign_outside}");

            let response = server
                .get(&format!("/realms/{realm}/users?not_in_group=nope"))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert!(
                response.text().contains("not_in_group"),
                "{}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test group_members_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=firstname", "order_by"),
                ("unknown=1", "unknown"),
                ("search=member", "search"),
                ("offset=0", "offset"),
                ("enabled=maybe", "enabled"),
                ("username=a&username=b", "username"),
            ] {
                let response = list(
                    &server,
                    &ctx().admin_token,
                    &ctx().realm,
                    ctx().organization_id,
                    ctx().group_id,
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
