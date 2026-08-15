/// Integration test: cross-realm administration of organizations and groups (FK-006).
///
/// The five methods of `OrganizationPolicy` used to resolve the caller's rights with
/// `get_user_permissions(user)` — the raw union of *every* role the caller holds, with
/// no realm parameter at all — instead of `get_permission_for_target_realm(user, realm)`,
/// the only lookup that applies the `can_access_realm` gate. They even prefixed their
/// target parameter with an underscore (`_realm_id`, `_organization`).
///
/// There was no second line of defence: the `auth` middleware validates the token's
/// signature against `user.realm_id` and never binds it to the realm in the URL. Tenancy
/// rested entirely on the policy layer.
///
/// The consequence: any bearer of a valid token holding `ManageUsers` or `ManageRealm`
/// **in their own realm** administered the organizations, groups, members and attributes
/// of every other realm. `ViewUsers` alone was enough to read.
///
/// These tests drive the real HTTP surface: `alice` is a genuine administrator of
/// `tenant-a` and attacks the URLs of `tenant-b`. Every assertion pairs the HTTP status
/// with a side-effect check performed through the master administrator, because a denial
/// that still wrote to the database would not be a fix.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]` so it does not run in
/// regular `cargo test`. Run explicitly with:
///
///   cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored
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

    /// `create_realm` resolves the parent realm with a hard-coded `get_by_name("master")`,
    /// and `can_access_realm` grants cross-realm access on the same literal. The master
    /// realm therefore has to be named exactly this for a tenant to be creatable at all.
    /// Per-run throwaway schemas keep the fixed name from colliding.
    const MASTER_REALM: &str = "master";

    /// The client `initialize_application` seeds. Named to avoid the `admin-cli`
    /// short-circuit described in #1086, exactly as the other integration tests do.
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    /// The attacker's realm — alice is a legitimate, fully-privileged admin here.
    const TENANT_A: &str = "tenant-a";
    /// The victim realm — alice holds no role whatsoever in it.
    const TENANT_B: &str = "tenant-b";

    /// Must satisfy the default password policy: >= 12 characters, 4 character classes.
    const ALICE_PASSWORD: &str = "Alice-P4ssw0rd!";

    const VICTIM_ORG_NAME: &str = "Victim Corp";
    const VICTIM_ORG_ALIAS: &str = "victim-corp";
    const VICTIM_GROUP_NAME: &str = "Victim Payroll Group";

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
        /// Master administrator token — used to seed and to re-read state out-of-band.
        admin_token: String,
        /// Token of `alice`, admin of `tenant-a` only. The attacker's credential.
        alice_token: String,
        /// Alice's own user id, for the non-regression pass on her own realm.
        alice_user_id: String,
        /// Organization seeded in `tenant-b` by the master admin.
        victim_org_id: String,
        /// Group seeded inside that organization.
        victim_group_id: String,
        /// A `tenant-b` user, member of the victim organization.
        victim_user_id: String,
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

        let schema = format!("org_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let server = TestServer::new(app.clone()).expect("create test server");

        // ── master administrator ────────────────────────────────────────────────
        let admin_token = password_grant(&server, MASTER_REALM, "admin", "admin").await;

        // ── two sibling tenants ─────────────────────────────────────────────────
        create_realm(&server, &admin_token, TENANT_A).await;
        create_realm(&server, &admin_token, TENANT_B).await;

        // ── alice: a real administrator of `tenant-a`, and nothing more ─────────
        let alice_id = create_user(&server, &admin_token, TENANT_A, "alice").await;
        set_password(&server, &admin_token, TENANT_A, &alice_id, ALICE_PASSWORD).await;

        // `manage_realm`, `manage_users` and `view_users` are the three permissions
        // `OrganizationPolicy` accepts. Granting all three inside `tenant-a` makes the
        // denial below unambiguous: alice is denied because of *where* she is asking,
        // never because she is under-privileged.
        let role_id = create_role(
            &server,
            &admin_token,
            TENANT_A,
            "tenant-a-org-admin",
            &["manage_realm", "manage_users", "view_users"],
        )
        .await;
        assign_role(&server, &admin_token, TENANT_A, &alice_id, &role_id).await;

        let alice_token = password_grant(&server, TENANT_A, "alice", ALICE_PASSWORD).await;

        // ── the victim's data, seeded in `tenant-b` by the master admin ─────────
        let victim_org_id = create_organization(
            &server,
            &admin_token,
            TENANT_B,
            VICTIM_ORG_NAME,
            VICTIM_ORG_ALIAS,
        )
        .await;
        let victim_group_id = create_group(
            &server,
            &admin_token,
            TENANT_B,
            &victim_org_id,
            VICTIM_GROUP_NAME,
        )
        .await;

        let victim_user_id = create_user(&server, &admin_token, TENANT_B, "bob").await;
        let add_member = server
            .post(&format!(
                "/realms/{}/organizations/{}/members",
                TENANT_B, victim_org_id
            ))
            .add_header("Authorization", auth_header(&admin_token))
            .json(&json!({ "user_id": victim_user_id }))
            .await;
        assert!(
            add_member.status_code().is_success(),
            "seeding the victim membership failed: {} {}",
            add_member.status_code(),
            add_member.text()
        );

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            alice_token,
            alice_user_id: alice_id,
            victim_org_id,
            victim_group_id,
            victim_user_id,
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

    fn ctx() -> &'static SharedContext {
        shared_ctx()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("bearer header is valid")
    }

    // ── seeding helpers ─────────────────────────────────────────────────────────

    async fn password_grant(
        server: &TestServer,
        realm: &str,
        username: &str,
        password: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
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
            .unwrap_or_else(|| panic!("no access_token for {username}@{realm}: {body}"))
            .to_string()
    }

    async fn create_realm(server: &TestServer, admin_token: &str, name: &str) {
        let response = server
            .post("/realms")
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "name": name }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating realm {name} failed: {}",
            response.text()
        );
    }

    async fn create_user(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        username: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": username,
                "lastname": "Tester",
                "email": format!("{username}@{realm}.test.local"),
                "email_verified": true,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "creating user {username}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created user {username}: {body}"))
            .to_string()
    }

    async fn set_password(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = server
            .put(&format!(
                "/realms/{}/users/{}/reset-password",
                realm, user_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "setting the password of {user_id}@{realm} failed: {}",
            response.text()
        );
    }

    async fn create_role(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
        permissions: &[&str],
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/roles", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "description": "organization administration inside this realm only",
                "permissions": permissions,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating role {name}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created role {name}: {body}"))
            .to_string()
    }

    async fn assign_role(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        user_id: &str,
        role_id: &str,
    ) {
        let response = server
            .post(&format!(
                "/realms/{}/users/{}/roles/{}",
                realm, user_id, role_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({}))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "assigning role {role_id} to {user_id}@{realm} failed: {}",
            response.text()
        );
    }

    async fn create_organization(
        server: &TestServer,
        token: &str,
        realm: &str,
        name: &str,
        alias: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/organizations", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "name": name,
                "alias": alias,
                "description": "seeded by the integration test",
                "enabled": true,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating organization {alias}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created organization {alias}: {body}"))
            .to_string()
    }

    async fn create_group(
        server: &TestServer,
        token: &str,
        realm: &str,
        organization_id: &str,
        name: &str,
    ) -> String {
        let response = server
            .post(&format!(
                "/realms/{}/organizations/{}/groups",
                realm, organization_id
            ))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": name }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating group {name} in {organization_id}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created group {name}: {body}"))
            .to_string()
    }

    // ── out-of-band verification, always through the master administrator ───────

    /// Every organization currently living in `realm`, read with a token that is
    /// legitimately allowed to see them.
    async fn organizations_of(server: &TestServer, realm: &str) -> Vec<Value> {
        let response = server
            .get(&format!("/realms/{}/organizations", realm))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "master admin could not list organizations of {realm}: {}",
            response.text()
        );

        let body: Value = response.json();
        body["data"]
            .as_array()
            .unwrap_or_else(|| panic!("organizations of {realm} is not an array: {body}"))
            .clone()
    }

    /// The group tree of an organization, flattened to the raw JSON text so a nested
    /// child cannot slip past a shallow check.
    async fn groups_text_of(server: &TestServer, realm: &str, organization_id: &str) -> String {
        let response = server
            .get(&format!(
                "/realms/{}/organizations/{}/groups",
                realm, organization_id
            ))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "master admin could not list groups of {organization_id}@{realm}: {}",
            response.text()
        );

        response.text()
    }

    /// A cross-realm request must be refused, and refused as an authorization failure
    /// rather than by accident (a 404 from a missing object would prove nothing).
    fn assert_forbidden(response: &TestResponse, what: &str) {
        assert_eq!(
            response.status_code(),
            403,
            "{what}: expected 403, got {} with body {}",
            response.status_code(),
            response.text()
        );
    }

    /// The response body must not carry the victim's data, whatever the status code.
    fn assert_body_free_of(response_body: &str, needles: &[&str], what: &str) {
        for needle in needles {
            assert!(
                !response_body.contains(needle),
                "{what}: the response leaked {needle:?}; body was {response_body}"
            );
        }
    }

    // ── FK-006: the attack, from the URL of the victim realm ────────────────────

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn create_organization_is_denied_across_realms() {
        // The most exploitable of the three unguarded paths: the object does not exist
        // yet, so the `org.realm_id != realm.id` check in `get_org_for_realm` cannot
        // help. Only the policy stands between alice and a write into `tenant-b`.
        rt().block_on(async {
            let server = make_server();
            let alias = "pwned-by-tenant-a";

            let response = server
                .post(&format!("/realms/{}/organizations", TENANT_B))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .json(&json!({
                    "name": "Pwned By Tenant A",
                    "alias": alias,
                    "enabled": true,
                }))
                .await;

            assert_forbidden(&response, "alice creating an organization in tenant-b");

            // The status code is the cheap half. This is the half that matters: nothing
            // may have landed in the victim realm.
            let orgs = organizations_of(&server, TENANT_B).await;
            assert!(
                !orgs.iter().any(|o| o["alias"] == alias),
                "an organization created by a foreign admin exists in {TENANT_B}: {orgs:?}"
            );
            assert_eq!(
                orgs.len(),
                1,
                "{TENANT_B} should still hold only its own organization, got {orgs:?}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn list_organizations_is_denied_across_realms() {
        // Enumeration. `list_organizations_by_realm` is scoped to the URL's realm, so a
        // permissive policy hands the attacker the victim realm's inventory verbatim.
        rt().block_on(async {
            let server = make_server();

            let response = server
                .get(&format!("/realms/{}/organizations", TENANT_B))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;

            let body = response.text();
            assert_forbidden(&response, "alice listing the organizations of tenant-b");
            assert_body_free_of(
                &body,
                &[VICTIM_ORG_NAME, VICTIM_ORG_ALIAS, &ctx().victim_org_id],
                "list organizations of tenant-b",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn list_user_organizations_is_denied_across_realms() {
        // The third unguarded path: the memberships of an arbitrary user of the victim
        // realm, read from the victim realm's own URL.
        rt().block_on(async {
            let server = make_server();

            let response = server
                .get(&format!(
                    "/realms/{}/users/{}/organizations",
                    TENANT_B,
                    ctx().victim_user_id
                ))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;

            let body = response.text();
            assert_forbidden(
                &response,
                "alice listing the organization memberships of a tenant-b user",
            );
            assert_body_free_of(
                &body,
                &[&ctx().victim_org_id, &ctx().victim_user_id],
                "list memberships of a tenant-b user",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn create_group_is_denied_across_realms() {
        // Groups hang off an organization that genuinely belongs to `tenant-b`, so
        // `get_org_for_realm` passes and the policy is again the only gate.
        rt().block_on(async {
            let server = make_server();
            let hostile_group = "Pwned Group From Tenant A";

            let response = server
                .post(&format!(
                    "/realms/{}/organizations/{}/groups",
                    TENANT_B,
                    ctx().victim_org_id
                ))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .json(&json!({ "name": hostile_group }))
                .await;

            assert_forbidden(
                &response,
                "alice creating a group in a tenant-b organization",
            );

            let groups = groups_text_of(&server, TENANT_B, &ctx().victim_org_id).await;
            assert!(
                !groups.contains(hostile_group),
                "a group created by a foreign admin exists in {TENANT_B}: {groups}"
            );
            assert!(
                groups.contains(VICTIM_GROUP_NAME),
                "the victim's own group vanished from {TENANT_B}: {groups}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn list_groups_is_denied_across_realms() {
        // The read counterpart: `ViewUsers` alone used to be enough to walk another
        // realm's group tree.
        rt().block_on(async {
            let server = make_server();

            let response = server
                .get(&format!(
                    "/realms/{}/organizations/{}/groups",
                    TENANT_B,
                    ctx().victim_org_id
                ))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;

            let body = response.text();
            assert_forbidden(
                &response,
                "alice listing the groups of a tenant-b organization",
            );
            assert_body_free_of(
                &body,
                &[VICTIM_GROUP_NAME, &ctx().victim_group_id],
                "list groups of a tenant-b organization",
            );
        });
    }

    // ── the other half of the fix: alice keeps her own realm ────────────────────

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test organization_cross_realm_test -- --ignored"]
    fn alice_still_administers_her_own_realm() {
        // Without this, a patch that simply denied everything would look like a success.
        // Same token, same endpoints, alice's own realm: everything must work.
        rt().block_on(async {
            let server = make_server();
            let alias = format!("home-org-{}", Uuid::new_v4().simple());
            let group_name = "Home Group";

            let org_id = create_organization(
                &server,
                &ctx().alice_token,
                TENANT_A,
                "Alice's Own Organization",
                &alias,
            )
            .await;

            let list = server
                .get(&format!("/realms/{}/organizations", TENANT_A))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;
            assert_eq!(
                list.status_code(),
                200,
                "alice cannot list her own realm's organizations: {}",
                list.text()
            );
            assert!(
                list.text().contains(&alias),
                "alice's own organization is missing from her realm's listing: {}",
                list.text()
            );

            let group_id =
                create_group(&server, &ctx().alice_token, TENANT_A, &org_id, group_name).await;

            let groups = server
                .get(&format!(
                    "/realms/{}/organizations/{}/groups",
                    TENANT_A, org_id
                ))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;
            assert_eq!(
                groups.status_code(),
                200,
                "alice cannot list the groups of her own organization: {}",
                groups.text()
            );
            assert!(
                groups.text().contains(&group_id),
                "alice's own group is missing from her organization's tree: {}",
                groups.text()
            );

            // And the memberships endpoint, on a user of her own realm.
            let memberships = server
                .get(&format!(
                    "/realms/{}/users/{}/organizations",
                    TENANT_A,
                    ctx().alice_user_id
                ))
                .add_header("Authorization", auth_header(&ctx().alice_token))
                .await;
            assert_eq!(
                memberships.status_code(),
                200,
                "alice cannot read memberships in her own realm: {}",
                memberships.text()
            );
        });
    }
}
