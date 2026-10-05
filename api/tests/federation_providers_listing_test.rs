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
    const SECRET_MARK: &str = "s3cr3t-value";
    const MASK: &str = "********";
    const PROVIDER_TYPES: [&str; 3] = ["Ldap", "ActiveDirectory", "Kerberos"];
    const SYNC_MODES: [&str; 3] = ["Import", "Force", "LinkOnly"];
    const SORT_FIELDS: [&str; 6] = [
        "name",
        "priority",
        "enabled",
        "last_sync_at",
        "created_at",
        "updated_at",
    ];

    struct Seed {
        id: Uuid,
        name: String,
        provider_type: &'static str,
        enabled: bool,
        priority: i32,
        sync_enabled: bool,
        sync_mode: &'static str,
        last_sync_minute: Option<i32>,
        created_minute: i32,
        updated_minute: i32,
    }

    impl Seed {
        fn new(i: usize) -> Self {
            let index = i32::try_from(i).expect("small index");
            Self {
                id: Uuid::new_v4(),
                name: format!("fed-{i:02}"),
                provider_type: PROVIDER_TYPES[i % PROVIDER_TYPES.len()],
                enabled: !i.is_multiple_of(4),
                priority: (index % 5) * 10,
                sync_enabled: i.is_multiple_of(3),
                sync_mode: SYNC_MODES[(i / 2) % SYNC_MODES.len()],
                last_sync_minute: if i % 5 == 2 { None } else { Some(index / 4) },
                created_minute: index / 3,
                updated_minute: i32::try_from(((i * 11) % SEED_COUNT) / 2).expect("small index"),
            }
        }

        fn config(&self) -> Value {
            json!({
                "connection": {
                    "server_url": format!("{}.example", self.name),
                    "port": 389,
                    "use_tls": false,
                },
                "bind": {
                    "bind_dn": "cn=admin,dc=example,dc=org",
                    "bind_password_encrypted": format!("{SECRET_MARK}-bind-{}", self.name),
                },
                "bind_password": format!("{SECRET_MARK}-top-{}", self.name),
                "search": {
                    "base_dn": "dc=example,dc=org",
                },
            })
        }

        fn key(&self, field: &str) -> SortKey {
            match field {
                "name" => SortKey::Text(self.name.clone()),
                "priority" => SortKey::Number(self.priority),
                "enabled" => SortKey::Flag(self.enabled),
                "last_sync_at" => SortKey::Nullable(
                    self.last_sync_minute.is_none(),
                    self.last_sync_minute.unwrap_or_default(),
                ),
                "created_at" => SortKey::Number(self.created_minute),
                "updated_at" => SortKey::Number(self.updated_minute),
                other => panic!("no sort key for {other}"),
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    enum SortKey {
        Text(String),
        Number(i32),
        Nullable(bool, i32),
        Flag(bool),
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        viewer_token: String,
        local_viewer_token: String,
        no_rights_token: String,
        realm: String,
        other_realm: String,
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
            "federation_providers_listing_test_{}",
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
        let realm = format!("listing-{}", &suffix[..8]);
        let other_realm = format!("other-{}", &suffix[..8]);
        create_realm(&server, &admin_token, &realm).await;
        create_realm(&server, &admin_token, &other_realm).await;

        let realm_id = realm_id_of(&pool, &realm).await;
        let other_realm_id = realm_id_of(&pool, &other_realm).await;

        let existing: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM user_federation_providers WHERE realm_id = $1::uuid",
        )
        .bind(realm_id.to_string())
        .fetch_one(&pool)
        .await
        .expect("count federation providers");
        assert_eq!(
            existing, 0,
            "a fresh realm must start without federation providers"
        );

        let seeds: Vec<Seed> = (0..SEED_COUNT).map(Seed::new).collect();
        for seed in &seeds {
            insert_seed(&pool, realm_id, seed).await;
        }

        for name in [
            "fed-50",
            "fed-51",
            "fed-52",
            "pct%fed",
            "pctxfed",
            "under_score",
            "underxscore",
            "back\\slash",
            "backxslash",
        ] {
            insert_plain_provider(&pool, other_realm_id, name).await;
        }

        let viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &other_realm,
            &format!("viewer-{}", &suffix[..8]),
            "tenant-viewer",
            &["view_realm"],
        )
        .await;
        let local_viewer_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("local-viewer-{}", &suffix[..8]),
            "local-viewer",
            &["view_realm"],
        )
        .await;
        let no_rights_token = user_with_permissions(
            &server,
            &admin_token,
            &realm,
            &format!("no-rights-{}", &suffix[..8]),
            "user-reader",
            &["view_users"],
        )
        .await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            viewer_token,
            local_viewer_token,
            no_rights_token,
            realm,
            other_realm,
            seeds,
        }
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
        set_password(server, admin_token, realm, &user_id, VIEWER_PASSWORD).await;
        let role_id = create_role(server, admin_token, realm, role_name, permissions).await;
        assign_role(server, admin_token, realm, &user_id, &role_id).await;
        direct_grant(server, realm, username, VIEWER_PASSWORD).await
    }

    async fn insert_seed(pool: &PgPool, realm_id: Uuid, seed: &Seed) {
        sqlx::query(
            "INSERT INTO user_federation_providers (id, realm_id, name, provider_type, enabled, priority, config, sync_enabled, sync_mode, sync_interval_minutes, last_sync_at, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6, $7::jsonb, $8, $12, 60, \
             CASE WHEN $9::int IS NULL THEN NULL ELSE TIMESTAMPTZ '2026-02-01 00:00:00+00' + make_interval(mins => $9::int) END, \
             TIMESTAMPTZ '2026-01-01 00:00:00+00' + make_interval(mins => $10), \
             TIMESTAMPTZ '2026-01-01 00:00:00+00' + make_interval(mins => $11))",
        )
        .bind(seed.id.to_string())
        .bind(realm_id.to_string())
        .bind(&seed.name)
        .bind(seed.provider_type)
        .bind(seed.enabled)
        .bind(seed.priority)
        .bind(seed.config().to_string())
        .bind(seed.sync_enabled)
        .bind(seed.last_sync_minute)
        .bind(seed.created_minute)
        .bind(seed.updated_minute)
        .bind(seed.sync_mode)
        .execute(pool)
        .await
        .expect("insert federation provider");
    }

    async fn insert_plain_provider(pool: &PgPool, realm_id: Uuid, name: &str) {
        sqlx::query(
            "INSERT INTO user_federation_providers (id, realm_id, name, provider_type, enabled, priority, config, created_at, updated_at) \
             VALUES ($1::uuid, $2::uuid, $3, 'Ldap', true, 0, '{}'::jsonb, now(), now())",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(realm_id.to_string())
        .bind(name)
        .execute(pool)
        .await
        .expect("insert plain federation provider");
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
                "description": "federation providers listing fixture",
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
            .get(&format!("/realms/{realm}/federation/providers?{query}"))
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
            .map(|provider| Uuid::parse_str(provider["id"].as_str().expect("id")).expect("uuid"))
            .collect()
    }

    fn names(body: &Value) -> Vec<String> {
        rows(body)
            .iter()
            .map(|provider| provider["name"].as_str().expect("name").to_string())
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
            .expect("seeded provider")
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

    fn created_at(minute: i32) -> String {
        format!("2026-01-01T00:{minute:02}:00Z")
    }

    fn created_within(from: Option<i32>, to: Option<i32>) -> HashSet<Uuid> {
        matching(|s| {
            from.is_none_or(|from| s.created_minute >= from)
                && to.is_none_or(|to| s.created_minute < to)
        })
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn no_params_returns_the_first_twenty_newest_providers() {
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

            let first = &rows(&body)[0];
            let seed = seed_of(expected[0]);
            assert_eq!(first["name"], seed.name.as_str());
            assert_eq!(first["provider_type"], seed.provider_type);
            assert_eq!(first["enabled"], seed.enabled);
            assert_eq!(first["priority"], seed.priority);
            assert_eq!(first["sync_enabled"], seed.sync_enabled);
            assert_eq!(first["sync_mode"], seed.sync_mode);
            assert_eq!(first["sync_interval_minutes"], 60);
            assert_eq!(
                first["last_sync_at"].is_null(),
                seed.last_sync_minute.is_none()
            );
            assert!(first["created_at"].is_string(), "{first}");
            assert!(first["updated_at"].is_string(), "{first}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn never_synced_providers_sort_last_ascending_and_first_descending() {
        let server = make_server();
        rt().block_on(async {
            let never = matching(|s| s.last_sync_minute.is_none());
            assert!(
                !never.is_empty() && never.len() < SEED_COUNT,
                "the fixture mixes synced and never-synced rows"
            );

            let ascending = list_ok(
                &server,
                &ctx().realm,
                "order_by=last_sync_at&order=asc&limit=100",
            )
            .await;
            let ascending = ids(&ascending);
            let (synced, tail) = ascending.split_at(SEED_COUNT - never.len());
            assert!(
                synced.iter().all(|id| !never.contains(id)),
                "asc: synced rows first"
            );
            assert_eq!(
                tail.iter().copied().collect::<HashSet<_>>(),
                never,
                "asc: nulls last"
            );

            let descending = list_ok(
                &server,
                &ctx().realm,
                "order_by=last_sync_at&order=desc&limit=100",
            )
            .await;
            let descending = ids(&descending);
            let (head, synced) = descending.split_at(never.len());
            assert_eq!(
                head.iter().copied().collect::<HashSet<_>>(),
                never,
                "desc: nulls first"
            );
            assert!(
                synced.iter().all(|id| !never.contains(id)),
                "desc: synced rows after"
            );

            let mut tie_breaker: Vec<Uuid> = never.iter().copied().collect();
            tie_breaker.sort();
            assert_eq!(tail, tie_breaker.as_slice(), "asc: nulls ordered by id");
            tie_breaker.reverse();
            assert_eq!(head, tie_breaker.as_slice(), "desc: nulls ordered by id");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn every_filter_returns_exactly_the_matching_providers() {
        let server = make_server();
        let cases: Vec<(&str, HashSet<Uuid>)> = vec![
            ("name=FED-1", matching(|s| s.name.contains("fed-1"))),
            ("name=d-2", matching(|s| s.name.contains("d-2"))),
            (
                "provider_type=Ldap",
                matching(|s| s.provider_type == "Ldap"),
            ),
            (
                "provider_type=ActiveDirectory",
                matching(|s| s.provider_type == "ActiveDirectory"),
            ),
            (
                "provider_type=Kerberos",
                matching(|s| s.provider_type == "Kerberos"),
            ),
            ("enabled=true", matching(|s| s.enabled)),
            ("enabled=false", matching(|s| !s.enabled)),
            ("sync_enabled=true", matching(|s| s.sync_enabled)),
            ("sync_enabled=false", matching(|s| !s.sync_enabled)),
            ("synced=true", matching(|s| s.last_sync_minute.is_some())),
            ("synced=false", matching(|s| s.last_sync_minute.is_none())),
            (
                "provider_type=Ldap&enabled=true",
                matching(|s| s.provider_type == "Ldap" && s.enabled),
            ),
            (
                "sync_enabled=true&synced=false",
                matching(|s| s.sync_enabled && s.last_sync_minute.is_none()),
            ),
            (
                "name=fed-1&provider_type=Kerberos",
                matching(|s| s.name.contains("fed-1") && s.provider_type == "Kerberos"),
            ),
            (
                "provider_family=ldap",
                matching(|s| s.provider_type == "Ldap" || s.provider_type == "ActiveDirectory"),
            ),
            (
                "provider_family=kerberos",
                matching(|s| s.provider_type == "Kerberos"),
            ),
            (
                "provider_family=ldap&enabled=false",
                matching(|s| {
                    (s.provider_type == "Ldap" || s.provider_type == "ActiveDirectory")
                        && !s.enabled
                }),
            ),
            ("search=FED-1", matching(|s| s.name.contains("fed-1"))),
            ("search=d-2", matching(|s| s.name.contains("d-2"))),
            (
                "search=fed-1&enabled=false",
                matching(|s| s.name.contains("fed-1") && !s.enabled),
            ),
            ("sync_mode=Import", matching(|s| s.sync_mode == "Import")),
            ("sync_mode=Force", matching(|s| s.sync_mode == "Force")),
            (
                "sync_mode=LinkOnly",
                matching(|s| s.sync_mode == "LinkOnly"),
            ),
            (
                "sync_mode=Force&enabled=true",
                matching(|s| s.sync_mode == "Force" && s.enabled),
            ),
            (
                "sync_mode=LinkOnly&search=fed-1",
                matching(|s| s.sync_mode == "LinkOnly" && s.name.contains("fed-1")),
            ),
            ("name=", matching(|_| true)),
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

            for query in ["provider_type=Ld", "provider_type=ldap"] {
                let body = list_ok(&server, &ctx().realm, &format!("{query}&limit=100")).await;
                assert!(
                    ids(&body).is_empty(),
                    "{query}: provider_type is an exact match"
                );
                assert_eq!(total(&body), 0, "{query}");
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn bind_credentials_stay_masked_in_the_listing() {
        let server = make_server();
        rt().block_on(async {
            for seed in &ctx().seeds {
                let config = seed.config();
                assert!(
                    config["bind"]["bind_password_encrypted"]
                        .as_str()
                        .is_some_and(|secret| secret.starts_with(SECRET_MARK)),
                    "{}: the fixture stores a bind password",
                    seed.name
                );
                assert!(
                    config["bind_password"]
                        .as_str()
                        .is_some_and(|secret| secret.starts_with(SECRET_MARK)),
                    "{}: the fixture stores a top-level password",
                    seed.name
                );
            }

            for query in [
                "limit=100",
                "provider_type=Ldap&limit=100",
                "order_by=name&page=2",
                "synced=false",
            ] {
                let response = list(&server, &ctx().admin_token, &ctx().realm, query).await;
                assert_eq!(response.status_code(), 200, "{query}: {}", response.text());
                let text = response.text();
                assert!(text.contains("fed-"), "{query}: rows are listed");
                assert!(!text.contains(SECRET_MARK), "{query}: {text}");
            }

            let body = list_ok(&server, &ctx().realm, "name=fed-00").await;
            assert_eq!(names(&body), ["fed-00"]);
            let config = &rows(&body)[0]["config"];
            assert_eq!(config["bind"]["bind_password_encrypted"], MASK);
            assert_eq!(config["bind_password"], MASK);
            assert_eq!(config["bind"]["bind_dn"], "cn=admin,dc=example,dc=org");
            assert_eq!(config["connection"]["server_url"], "fed-00.example");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn providers_of_another_realm_are_never_listed() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=fed-5").await;
            assert_eq!(total(&witness), 3, "the other realm holds fed-5x rows");

            let body = list_ok(&server, &ctx().realm, "name=fed-5").await;
            assert!(ids(&body).is_empty(), "{body}");
            assert_eq!(total(&body), 0);

            let everything = list_ok(&server, &ctx().realm, "limit=100").await;
            assert_eq!(total(&everything), SEED_COUNT as u64);
            assert!(
                names(&everything)
                    .iter()
                    .all(|name| name.starts_with("fed-") && name.len() == 6)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
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
            assert!(!foreign.contains("fed-00"), "{foreign}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn a_caller_of_the_realm_without_provider_rights_is_forbidden() {
        let server = make_server();
        rt().block_on(async {
            let witness = list(
                &server,
                &ctx().local_viewer_token,
                &ctx().realm,
                "limit=100",
            )
            .await;
            assert_eq!(witness.status_code(), 200, "{}", witness.text());
            let witness: Value = witness.json();
            assert_eq!(total(&witness), SEED_COUNT as u64);
            assert_eq!(ids(&witness).len(), SEED_COUNT);

            let refused = list(&server, &ctx().no_rights_token, &ctx().realm, "").await;
            assert_eq!(refused.status_code(), 403, "{}", refused.text());
            let refused = refused.text();
            assert!(!refused.contains("fed-00"), "{refused}");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn like_wildcards_match_literally() {
        let server = make_server();
        rt().block_on(async {
            let witness = list_ok(&server, &ctx().other_realm, "name=pctx").await;
            assert_eq!(names(&witness), ["pctxfed"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=underx").await;
            assert_eq!(names(&witness), ["underxscore"]);
            let witness = list_ok(&server, &ctx().other_realm, "name=backx").await;
            assert_eq!(names(&witness), ["backxslash"]);

            let percent = list_ok(&server, &ctx().other_realm, "name=%25").await;
            assert_eq!(names(&percent), ["pct%fed"]);
            assert_eq!(total(&percent), 1);

            let underscore = list_ok(&server, &ctx().other_realm, "name=_").await;
            assert_eq!(names(&underscore), ["under_score"]);
            assert_eq!(total(&underscore), 1);

            let backslash = list_ok(&server, &ctx().other_realm, "name=%5C").await;
            assert_eq!(names(&backslash), ["back\\slash"]);
            assert_eq!(total(&backslash), 1);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn invalid_query_parameters_are_rejected() {
        let server = make_server();
        rt().block_on(async {
            for (query, param) in [
                ("limit=101", "limit"),
                ("limit=0", "limit"),
                ("order_by=config", "order_by"),
                ("order_by=sync_mode", "order_by"),
                ("order=sideways", "order"),
                ("unknown=1", "unknown"),
                ("enabled=maybe", "enabled"),
                ("sync_enabled=maybe", "sync_enabled"),
                ("synced=maybe", "synced"),
                ("provider_family=Ldap", "provider_family"),
                ("provider_family=custom", "provider_family"),
                ("name=a&name=b", "name"),
                ("search=a&search=b", "search"),
                ("sync_mode=import", "sync_mode"),
                ("sync_mode=Custom", "sync_mode"),
                ("sync_mode=Import&sync_mode=Force", "sync_mode"),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn search_matches_wildcards_literally() {
        let server = make_server();
        rt().block_on(async {
            let realm = &ctx().other_realm;
            let percent = list_ok(&server, realm, "search=%25").await;
            assert_eq!(names(&percent), ["pct%fed"]);
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
    fn sync_mode_matches_the_stored_value_exactly() {
        let server = make_server();
        rt().block_on(async {
            let mut seen = 0;
            for mode in SYNC_MODES {
                let body = list_ok(
                    &server,
                    &ctx().realm,
                    &format!("sync_mode={mode}&limit=100"),
                )
                .await;
                assert!(
                    rows(&body)
                        .iter()
                        .all(|provider| provider["sync_mode"] == mode),
                    "{mode}: {body}"
                );
                seen += total(&body);
            }
            assert_eq!(seen, SEED_COUNT as u64);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
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
                format!("created_to={}&search=fed-0", created_at(2)),
                matching(|s| s.created_minute < 2 && s.name.contains("fed-0")),
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test federation_providers_listing_test -- --ignored"]
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
