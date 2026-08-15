/// Integration test: user endpoints must not reach across tenant realms (FK-004).
///
/// Seven methods of `core/src/domain/user/services.rs` evaluate the authorization
/// policy against the realm **named in the URL**, then load the target object by
/// bare UUID without ever checking that the object belongs to that realm:
/// `reset_password`, `delete_user`, `update_user`, `bulk_delete_users`, `get_user`,
/// `assign_role` and `unassign_role`. An administrator of one tenant realm can
/// therefore drive them against a user of *another* tenant realm simply by putting
/// their own realm in the path — the policy passes (they are legitimate there) and
/// the repository happily returns the foreign user.
///
/// Cross-realm access *from* `master` is deliberate: `can_access_realm` grants it on
/// `user_realm.name == "master"`. So the attacker here is an administrator of
/// `tenant-a`, never the master admin — otherwise the test would be exercising a
/// supported feature instead of the defect.
///
/// The attacker needs the victim's UUID; there is no cross-realm enumeration
/// primitive (`get_users` is correctly scoped). The setup obtains that UUID
/// legitimately as the master admin, standing in for an out-of-band leak.
///
/// The fix returns `CoreError::NotFound` — never `Forbidden` — for an out-of-realm
/// id, so that the endpoint is not an existence oracle. The assertions are on 404.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]`. Run with:
///
///   cargo test -p ferriskey-api --test user_cross_realm_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

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
    use sqlx::Executor;
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";

    /// The client `initialize_application` seeds. Named to avoid the `admin-cli`
    /// short-circuit described in #1086, exactly as the other integration tests do.
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    /// Unlike the other suites, this harness cannot use a random master realm name.
    /// `RealmService::create_realm` resolves its parent with `get_by_name("master")`
    /// and `FerriskeyPolicy::can_access_realm` grants cross-realm access on the same
    /// literal, so a differently-named seed realm can neither create tenants nor
    /// administer them. Each run owns a throwaway PostgreSQL schema, so the fixed
    /// name collides with nothing.
    const MASTER_REALM: &str = "master";

    /// The tenant admin's own password, and the victim's. Both satisfy the default
    /// CNIL policy (>= 12 chars, four classes, >= 80 bits).
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-Adm!";
    const VICTIM_PASSWORD: &str = "V1ctim-Original-Pw!";
    /// What the attacker tries to impose on the victim. If it ever authenticates,
    /// the account has been taken over.
    const ATTACKER_PASSWORD: &str = "Pwn3d-By-Al1ce-Now!";

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        /// Two sibling tenant realms, neither of them `master`.
        tenant_a: String,
        tenant_b: String,
        /// Master admin — used only to build the fixture, never to attack.
        admin_token: String,
        /// `alice`, administrator of `tenant_a` only. This is the attacker.
        alice_token: String,
    }

    // `router()` installs a *global* Prometheus recorder, so it can only be built
    // once per test binary (#1086). Every test shares this one router and runtime.
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

    fn shared_ctx() -> &'static SharedContext {
        CTX.get_or_init(|| match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| handle.block_on(init_shared_ctx())),
            Err(_) => rt().block_on(init_shared_ctx()),
        })
    }

    async fn init_shared_ctx() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("user_cross_realm_test_{}", Uuid::new_v4().simple());

        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");

        admin_pool
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{}\"",
                schema
            )))
            .await
            .expect("create schema");

        let schema_url = format!(
            "postgres://{}:{}@{}:{}/{}?options=-c search_path={}",
            db_user,
            db_password,
            db_host,
            db_port,
            db_name,
            urlencoding::encode(&schema)
        );

        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");

        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let service = create_service(FerriskeyConfig {
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

        // ---- fixture, driven through the real HTTP surface -------------------
        let server = TestServer::new(app.clone()).expect("create fixture server");

        let admin_token = direct_grant(&server, MASTER_REALM, "admin", "admin").await;

        let suffix = Uuid::new_v4().simple().to_string();
        let tenant_a = format!("tenant-a-{}", &suffix[..8]);
        let tenant_b = format!("tenant-b-{}", &suffix[..8]);

        create_realm(&server, &admin_token, &tenant_a).await;
        create_realm(&server, &admin_token, &tenant_b).await;

        // alice is a plain administrator of tenant-a: she may manage the users of
        // her own realm and nothing more. No ManageRealm, and above all no
        // membership of `master`.
        let alice_username = format!("alice-{}", &suffix[..8]);
        let alice_id = create_user(&server, &admin_token, &tenant_a, &alice_username).await;
        set_password(&server, &admin_token, &tenant_a, &alice_id, ALICE_PASSWORD).await;

        let role_id = create_role(
            &server,
            &admin_token,
            &tenant_a,
            &format!("tenant-a-user-admin-{}", &suffix[..8]),
            // `can_update_user` / `can_delete_user` accept ManageUsers;
            // `can_view_user` accepts ViewUsers (or ManageRealm) but *not*
            // ManageUsers, so both names are needed to exercise every route.
            &["manage_users", "view_users"],
        )
        .await;
        assign_role(&server, &admin_token, &tenant_a, &alice_id, &role_id).await;

        let alice_token = direct_grant(&server, &tenant_a, &alice_username, ALICE_PASSWORD).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            tenant_a,
            tenant_b,
            admin_token,
            alice_token,
        }
    }

    fn make_server() -> TestServer {
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn tenant_a() -> &'static str {
        shared_ctx().tenant_a.as_str()
    }

    fn tenant_b() -> &'static str {
        shared_ctx().tenant_b.as_str()
    }

    fn admin_token() -> &'static str {
        shared_ctx().admin_token.as_str()
    }

    /// The attacker's bearer token: alice, administrator of `tenant_a` only.
    fn alice_token() -> &'static str {
        shared_ctx().alice_token.as_str()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    // -------------------------------------------------------------------------
    // Fixture helpers — every one of them goes through the public HTTP API
    // -------------------------------------------------------------------------

    /// `POST /realms/{realm}/protocol/openid-connect/token`, `grant_type=password`.
    /// Every realm — master and tenants alike — is seeded with an `admin-cli`
    /// public client that has direct access grants enabled.
    async fn login(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> TestResponse {
        server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", username),
                ("password", password),
            ])
            .await
    }

    async fn direct_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let response = login(server, realm, username, password).await;

        assert_eq!(
            response.status_code(),
            200,
            "token request for {username}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .unwrap_or_else(|| panic!("access_token in response for {username}@{realm}: {body}"))
            .to_string()
    }

    /// `POST /realms` — only the master realm's administrator may call it.
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

    /// `POST /realms/{realm}/users` → 200, `{"data": {"id": …}}`.
    async fn create_user(server: &TestServer, token: &str, realm: &str, username: &str) -> String {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": user_email(username),
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
            .unwrap_or_else(|| panic!("created user id in response: {body}"))
            .to_string()
    }

    fn user_email(username: &str) -> String {
        format!("{}@test.local", username)
    }

    /// `PUT /realms/{realm}/users/{id}/reset-password` — note the verb: the route is
    /// a PUT, not a POST.
    async fn reset_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) -> TestResponse {
        server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await
    }

    async fn set_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = reset_password(server, token, realm, user_id, password).await;
        assert_eq!(
            response.status_code(),
            200,
            "set password for {user_id} in {realm} failed: {}",
            response.text()
        );
    }

    /// `POST /realms/{realm}/roles` → 201, `{"data": {"id": …}}`.
    async fn create_role(
        server: &TestServer,
        token: &str,
        realm: &str,
        name: &str,
        permissions: &[&str],
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/roles", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "name": name,
                "description": "tenant-scoped user administration",
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
            .unwrap_or_else(|| panic!("created role id in response: {body}"))
            .to_string()
    }

    /// `POST /realms/{realm}/users/{user_id}/roles/{role_id}` → 200.
    async fn assign_role(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        role_id: &str,
    ) {
        let response = server
            .post(&format!(
                "/realms/{}/users/{}/roles/{}",
                realm, user_id, role_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&json!({}))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "assign role {role_id} to {user_id} in {realm} failed: {}",
            response.text()
        );
    }

    async fn get_user(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
    ) -> TestResponse {
        server
            .get(&format!("/realms/{}/users/{}", realm, user_id))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn delete_user(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
    ) -> TestResponse {
        server
            .delete(&format!("/realms/{}/users/{}", realm, user_id))
            .add_header("Authorization", auth_header(token))
            .await
    }

    /// Plant a fresh victim in `tenant_b` with a known password. Returns
    /// `(user_id, username)`. Each test gets its own so a successful attack in one
    /// test cannot corrupt another.
    async fn plant_victim(server: &TestServer) -> (String, String) {
        let username = format!("victim-{}", Uuid::new_v4().simple());
        let user_id = create_user(server, admin_token(), tenant_b(), &username).await;
        set_password(server, admin_token(), tenant_b(), &user_id, VICTIM_PASSWORD).await;
        (user_id, username)
    }

    // -------------------------------------------------------------------------
    // Attacks — alice (tenant-a) aims at a tenant-b user through tenant-a's URL
    // -------------------------------------------------------------------------

    /// Worst impact: the tenant-a admin sets the password of a tenant-b user and
    /// owns the account.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_cross_realm_test -- --ignored"]
    fn reset_password_across_tenant_realms_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let (victim_id, victim_username) = plant_victim(&server).await;

            // The policy is evaluated against tenant-a, where alice is a legitimate
            // administrator; the target is then loaded by bare UUID.
            let attack = reset_password(
                &server,
                alice_token(),
                tenant_a(),
                &victim_id,
                ATTACKER_PASSWORD,
            )
            .await;

            let status = attack.status_code();
            let body = attack.text();

            // The credential is checked before the status code, so that a successful
            // attack is reported as the account takeover it is rather than as a bare
            // status mismatch.
            let takeover = login(&server, tenant_b(), &victim_username, ATTACKER_PASSWORD).await;
            assert_ne!(
                takeover.status_code(),
                200,
                "tenant-b user {victim_id} now authenticates with the password a \
                 tenant-a admin chose; reset-password returned {status}: {body} — \
                 login returned {}: {}",
                takeover.status_code(),
                takeover.text()
            );

            assert_eq!(
                status, 404,
                "a tenant-a admin reset the password of tenant-b user {victim_id}; \
                 got {status}: {body}"
            );

            let legitimate = login(&server, tenant_b(), &victim_username, VICTIM_PASSWORD).await;
            assert_eq!(
                legitimate.status_code(),
                200,
                "the victim's own password stopped working: {}",
                legitimate.text()
            );
        });
    }

    /// Disclosure: the tenant-a admin reads a tenant-b user's record.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_cross_realm_test -- --ignored"]
    fn get_user_across_tenant_realms_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let (victim_id, victim_username) = plant_victim(&server).await;

            let attack = get_user(&server, alice_token(), tenant_a(), &victim_id).await;

            let status = attack.status_code();
            let body = attack.text();
            assert_eq!(
                status, 404,
                "a tenant-a admin read tenant-b user {victim_id}; got {status}: {body}"
            );
            assert!(
                !body.contains(&user_email(&victim_username)),
                "the response disclosed the victim's email: {body}"
            );
            assert!(
                !body.contains(&victim_username),
                "the response disclosed the victim's username: {body}"
            );
        });
    }

    /// Destruction: the tenant-a admin deletes a tenant-b user.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_cross_realm_test -- --ignored"]
    fn delete_user_across_tenant_realms_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let (victim_id, _victim_username) = plant_victim(&server).await;

            let attack = delete_user(&server, alice_token(), tenant_a(), &victim_id).await;

            let status = attack.status_code();
            let body = attack.text();

            // Check survival before asserting the status, so a successful deletion is
            // reported as such rather than hidden behind the status assertion.
            let survivor = get_user(&server, admin_token(), tenant_b(), &victim_id).await;
            let survived = survivor.status_code() == 200;

            assert!(
                survived,
                "tenant-b user {victim_id} was deleted from tenant-a; \
                 delete returned {status}: {body} — lookup returned {}: {}",
                survivor.status_code(),
                survivor.text()
            );
            assert_eq!(
                status, 404,
                "a tenant-a admin was allowed to delete tenant-b user {victim_id}; \
                 got {status}: {body}"
            );
        });
    }

    // -------------------------------------------------------------------------
    // Non-regression — the same admin, on her own realm, must keep working
    // -------------------------------------------------------------------------

    /// The realm check must reject foreign ids without breaking the ordinary case:
    /// alice creates, reads, re-passwords and deletes a user of `tenant_a`.
    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test user_cross_realm_test -- --ignored"]
    fn tenant_admin_still_administers_users_of_its_own_realm() {
        rt().block_on(async {
            let server = make_server();

            let username = format!("colleague-{}", Uuid::new_v4().simple());
            let user_id = create_user(&server, alice_token(), tenant_a(), &username).await;

            let read = get_user(&server, alice_token(), tenant_a(), &user_id).await;
            assert_eq!(
                read.status_code(),
                200,
                "alice cannot read a user of her own realm: {}",
                read.text()
            );
            let read_body: Value = read.json();
            assert_eq!(
                read_body["data"]["username"].as_str(),
                Some(username.as_str()),
                "unexpected user returned: {read_body}"
            );

            let reset = reset_password(
                &server,
                alice_token(),
                tenant_a(),
                &user_id,
                VICTIM_PASSWORD,
            )
            .await;
            assert_eq!(
                reset.status_code(),
                200,
                "alice cannot reset the password of a user of her own realm: {}",
                reset.text()
            );

            let login_resp = login(&server, tenant_a(), &username, VICTIM_PASSWORD).await;
            assert_eq!(
                login_resp.status_code(),
                200,
                "the password alice set on her own realm does not authenticate: {}",
                login_resp.text()
            );

            let removed = delete_user(&server, alice_token(), tenant_a(), &user_id).await;
            assert_eq!(
                removed.status_code(),
                200,
                "alice cannot delete a user of her own realm: {}",
                removed.text()
            );

            let gone = get_user(&server, alice_token(), tenant_a(), &user_id).await;
            assert_ne!(
                gone.status_code(),
                200,
                "the deleted user is still readable: {}",
                gone.text()
            );
        });
    }
}
