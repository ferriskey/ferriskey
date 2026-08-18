/// Integration test: cross-realm IDOR on OAuth clients (FK-005).
///
/// `ClientService` decided authorization against the realm named in the URL, then
/// loaded the client by **bare UUID**. Thirteen of its fifteen methods never checked
/// that the client actually belonged to that realm, so an administrator of one
/// tenant could reach into another tenant's clients:
///
///   * **Secret theft.** `Client.secret` is stored unhashed and serialized in full by
///     `GET /realms/{realm}/clients/{id}` — no masking. A stolen secret is directly
///     replayable against the victim tenant's token endpoint.
///   * **Weakening.** `PATCH` could flip `require_pkce` to `false`, turn on
///     `direct_access_grants_enabled`, or stretch token lifetimes on a third party.
///   * **Destruction.** `DELETE` cascades onto roles, the service account, scope
///     mappings and post-logout redirect URIs. Deleting a realm's console client
///     takes its administration UI offline.
///   * **Redirect URI injection**, which combined with the neighbouring findings
///     hijacks the victim tenant's authorization codes.
///
/// Cross-realm access **from `master` is intentional** (`can_access_realm` grants it
/// on the literal name `master`). The finding is strictly the tenant → other tenant
/// escalation, so the attacker below is an administrator of a *tenant* realm, never
/// the master admin — otherwise the test would be exercising designed behaviour.
///
/// Every attack asserts a side effect as well as the status code: a 404 obtained for
/// the wrong reason proves nothing. The load-bearing assertion is that the response
/// body of the attacker's `GET` does not contain the victim client's real secret,
/// read back through the master admin.
///
/// Requires a running PostgreSQL instance. Marked `#[ignore]` so it does not run in
/// regular `cargo test`. Run explicitly with:
///
///   cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored
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
    use axum_test::TestServer;
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

    /// `create_realm` resolves the parent realm through `get_by_name("master")` and
    /// `can_access_realm` grants cross-realm on the same literal, so the master realm
    /// must be seeded under exactly this name or no tenant can be created at all.
    const MASTER_REALM: &str = "master";

    /// Naming the seeded client `admin-cli` short-circuits its seeding (#1086).
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    const MASTER_ADMIN_USERNAME: &str = "admin";
    const MASTER_ADMIN_PASSWORD: &str = "admin";

    /// The attacker's realm.
    const TENANT_A: &str = "tenant-a";
    /// The victim's realm.
    const TENANT_B: &str = "tenant-b";

    /// Administrator of `tenant-a`. Holds `manage_clients` and `view_clients` inside
    /// her own realm and nothing at all in `tenant-b`.
    const ALICE_USERNAME: &str = "alice";
    /// Default CNIL policy: at least 12 characters over 4 character classes.
    const ALICE_PASSWORD: &str = "Att4cker-Passw0rd!";

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
    }

    // `router()` installs a *global* Prometheus recorder, so it can only be built
    // once per test binary (#1086). Every test shares this one router and runtime,
    // hence `#[test]` + `rt().block_on(..)` rather than `#[tokio::test]`.
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

        // A throwaway schema per run: the realms below carry fixed names, so runs
        // would otherwise collide.
        let schema = format!("client_cross_realm_test_{}", Uuid::new_v4().simple());

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
                admin_username: MASTER_ADMIN_USERNAME.to_string(),
                admin_password: MASTER_ADMIN_PASSWORD.to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        let ctx = SharedContext {
            app: std::sync::Mutex::new(app),
        };

        seed_tenants(&ctx).await;

        ctx
    }

    /// Build the two tenant realms and the attacker account. Runs once, inside
    /// `init_shared_ctx`, before any test observes the context.
    async fn seed_tenants(ctx: &SharedContext) {
        let app = ctx.app.lock().expect("router mutex poisoned").clone();
        let server = TestServer::new(app).expect("create test server");

        let master = get_token(
            &server,
            MASTER_REALM,
            MASTER_ADMIN_USERNAME,
            MASTER_ADMIN_PASSWORD,
        )
        .await;

        for realm in [TENANT_A, TENANT_B] {
            let response = server
                .post("/realms")
                .add_header("Authorization", auth_header(&master))
                .json(&json!({ "name": realm, "display_name": realm }))
                .await;
            assert_eq!(
                response.status_code(),
                201,
                "creating realm {realm} failed: {}",
                response.text()
            );
        }

        // Alice: administrator of tenant-a only.
        let response = server
            .post(&format!("/realms/{TENANT_A}/users"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "username": ALICE_USERNAME,
                "firstname": "Alice",
                "lastname": "Tenant A",
                "email": format!("{ALICE_USERNAME}@tenant-a.local"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "creating alice failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let alice_id = body["data"]["id"].as_str().expect("alice id").to_string();

        let response = server
            .put(&format!(
                "/realms/{TENANT_A}/users/{alice_id}/reset-password"
            ))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "value": ALICE_PASSWORD,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "setting alice's password failed: {}",
            response.text()
        );

        // `can_view_client` accepts ManageRealm or ViewClients; the create / update /
        // delete policies accept ManageRealm or ManageClients. Alice therefore needs
        // both client permissions, and deliberately not ManageRealm.
        let response = server
            .post(&format!("/realms/{TENANT_A}/roles"))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({
                "name": "tenant-a-client-admin",
                "description": "manage clients inside tenant-a",
                "permissions": ["manage_clients", "view_clients"],
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "creating alice's role failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let role_id = body["data"]["id"].as_str().expect("role id").to_string();

        let response = server
            .post(&format!(
                "/realms/{TENANT_A}/users/{alice_id}/roles/{role_id}"
            ))
            .add_header("Authorization", auth_header(&master))
            .json(&json!({}))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "assigning alice's role failed: {}",
            response.text()
        );

        // Mint and discard one token per realm while seeding is still single
        // threaded. `KeystoreRepository::get_or_generate_key` is a read-then-insert
        // with no unique index on `jwt_keys.realm_id`, so the first *concurrent*
        // logins to a cold realm each insert a key pair; `find().one()` then returns
        // an arbitrary one and tokens signed with the other fail verification with a
        // bare 401. Warming the key here keeps the tests from tripping over that
        // unrelated race.
        get_token(&server, TENANT_A, ALICE_USERNAME, ALICE_PASSWORD).await;
    }

    fn make_server() -> TestServer {
        let app = shared_ctx()
            .app
            .lock()
            .expect("router mutex poisoned")
            .clone();
        TestServer::new(app).expect("create test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("bearer header is valid ASCII")
    }

    /// Direct-grant login through the realm's own `admin-cli`, which `create_realm`
    /// seeds with `direct_access_grants_enabled`.
    async fn get_token(server: &TestServer, realm: &str, username: &str, password: &str) -> String {
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

    async fn master_token(server: &TestServer) -> String {
        get_token(
            server,
            MASTER_REALM,
            MASTER_ADMIN_USERNAME,
            MASTER_ADMIN_PASSWORD,
        )
        .await
    }

    async fn alice_token(server: &TestServer) -> String {
        get_token(server, TENANT_A, ALICE_USERNAME, ALICE_PASSWORD).await
    }

    struct VictimClient {
        uuid: String,
        secret: String,
    }

    /// Create a confidential client in `tenant-b` — confidential so the repository
    /// generates a secret — harden it with `require_pkce`, and register one redirect
    /// URI so the injection test has a baseline to compare against.
    ///
    /// Each test gets its own victim: the deletion test would otherwise destroy the
    /// fixture the others depend on.
    async fn create_victim_client(server: &TestServer, master: &str) -> VictimClient {
        let client_id = format!("victim-{}", Uuid::new_v4().simple());

        let response = server
            .post(&format!("/realms/{TENANT_B}/clients"))
            .add_header("Authorization", auth_header(master))
            .json(&json!({
                "client_id": client_id,
                "name": "Tenant B confidential client",
                "client_type": "confidential",
                "protocol": "openid-connect",
                "public_client": false,
                "service_account_enabled": false,
                "direct_access_grants_enabled": false,
                "enabled": true,
                "oauth_device_code_grant_enabled": false,
            }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "creating the victim client failed: {}",
            response.text()
        );
        let body: Value = response.json();
        let uuid = body["id"].as_str().expect("victim client id").to_string();
        let secret = body["client_secret"]
            .as_str()
            .expect("creation is the one place that hands back the plaintext secret")
            .to_string();
        assert!(
            !secret.is_empty() && secret != "***",
            "the leak assertions below are meaningless unless this is the real secret: {secret}"
        );

        // `create_client` always persists `require_pkce: false`; the update attack
        // needs it on so that flipping it off is an observable weakening.
        let response = server
            .patch(&format!("/realms/{TENANT_B}/clients/{uuid}"))
            .add_header("Authorization", auth_header(master))
            .json(&json!({ "require_pkce": true }))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "hardening the victim client failed: {}",
            response.text()
        );

        let response = server
            .post(&format!("/realms/{TENANT_B}/clients/{uuid}/redirects"))
            .add_header("Authorization", auth_header(master))
            .json(&json!({ "value": "https://tenant-b.example/callback", "enabled": true }))
            .await;
        assert_eq!(
            response.status_code(),
            201,
            "registering the victim's redirect URI failed: {}",
            response.text()
        );

        VictimClient { uuid, secret }
    }

    /// Read a `tenant-b` client back through the master admin, whose cross-realm
    /// access is intentional. This is the oracle every side-effect assertion uses.
    async fn read_victim_client(server: &TestServer, master: &str, uuid: &str) -> Value {
        let response = server
            .get(&format!("/realms/{TENANT_B}/clients/{uuid}"))
            .add_header("Authorization", auth_header(master))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "master admin could not read the victim client {uuid}: {}",
            response.text()
        );
        let body: Value = response.json();
        body["data"].clone()
    }

    async fn read_victim_redirect_uris(
        server: &TestServer,
        master: &str,
        uuid: &str,
    ) -> Vec<String> {
        let response = server
            .get(&format!("/realms/{TENANT_B}/clients/{uuid}/redirects"))
            .add_header("Authorization", auth_header(master))
            .await;
        assert_eq!(
            response.status_code(),
            200,
            "master admin could not list the victim's redirect URIs: {}",
            response.text()
        );
        let body: Value = response.json();
        body.as_array()
            .expect("redirect URI list")
            .iter()
            .map(|uri| uri["value"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn reading_a_client_never_yields_its_plaintext_secret() {
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let victim = create_victim_client(&server, &master).await;

            let client = read_victim_client(&server, &master, &victim.uuid).await;
            let serialized = client.to_string();
            assert!(
                !serialized.contains(&victim.secret),
                "the client read endpoint must not carry the plaintext secret: {serialized}"
            );
            assert_eq!(
                client["secret"], "***",
                "the field must be present but redacted, so the console can still tell a confidential client apart: {client}"
            );

            let list = server
                .get(&format!("/realms/{TENANT_B}/clients"))
                .add_header("Authorization", auth_header(&master))
                .await;
            assert!(
                !list.text().contains(&victim.secret),
                "the client list must not carry the plaintext secret"
            );

            let reveal = server
                .get(&format!(
                    "/realms/{TENANT_B}/clients/{}/client-secret",
                    victim.uuid
                ))
                .add_header("Authorization", auth_header(&master))
                .await;
            assert_eq!(
                reveal.status_code(),
                200,
                "an operator holding ManageClients must still be able to retrieve it: {}",
                reveal.text()
            );
            let body: Value = reveal.json();
            assert_eq!(
                body["client_secret"], victim.secret,
                "the dedicated endpoint must return the real secret: {body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn tenant_admin_cannot_read_another_tenants_client_secret() {
        // The headline exploit. `Client.secret` is unhashed and serialized in the
        // clear, so a successful read hands the attacker credentials replayable on
        // tenant-b's own token endpoint.
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let victim = create_victim_client(&server, &master).await;
            let alice = alice_token(&server).await;

            let response = server
                .get(&format!("/realms/{TENANT_A}/clients/{}", victim.uuid))
                .add_header("Authorization", auth_header(&alice))
                .await;

            let status = response.status_code();
            let body = response.text();

            // Asserted before the status code: on the vulnerable build this is the
            // failure that carries the stolen secret into the test output.
            assert!(
                !body.contains(&victim.secret),
                "FK-005: the tenant-a administrator read tenant-b's client secret \
                 ({secret}). HTTP {status}, body: {body}",
                secret = victim.secret,
            );
            assert!(
                !body.contains(&victim.uuid),
                "FK-005: tenant-a received tenant-b's client representation. \
                 HTTP {status}, body: {body}",
            );
            assert_eq!(
                status, 404,
                "expected 404 for a client outside the URL realm, body: {body}",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn tenant_admin_cannot_weaken_another_tenants_client() {
        // Turning `require_pkce` off on someone else's confidential client removes
        // the defence that makes a stolen authorization code useless.
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let victim = create_victim_client(&server, &master).await;
            let alice = alice_token(&server).await;

            let response = server
                .patch(&format!("/realms/{TENANT_A}/clients/{}", victim.uuid))
                .add_header("Authorization", auth_header(&alice))
                .json(&json!({
                    "require_pkce": false,
                    "direct_access_grants_enabled": true,
                }))
                .await;

            let status = response.status_code();
            let body = response.text();
            assert_eq!(
                status, 404,
                "expected 404 when updating a client outside the URL realm, body: {body}",
            );

            let client = read_victim_client(&server, &master, &victim.uuid).await;
            assert_eq!(
                client["require_pkce"].as_bool(),
                Some(true),
                "FK-005: tenant-a turned require_pkce off on tenant-b's client. \
                 Attack returned HTTP {status}; client is now {client}",
            );
            assert_eq!(
                client["direct_access_grants_enabled"].as_bool(),
                Some(false),
                "FK-005: tenant-a enabled direct access grants on tenant-b's client. \
                 Attack returned HTTP {status}; client is now {client}",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn tenant_admin_cannot_delete_another_tenants_client() {
        // The delete cascades onto roles, the service account, scope mappings and
        // post-logout redirect URIs — deleting a realm's console client takes its
        // administration UI offline.
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let victim = create_victim_client(&server, &master).await;
            let alice = alice_token(&server).await;

            let response = server
                .delete(&format!("/realms/{TENANT_A}/clients/{}", victim.uuid))
                .add_header("Authorization", auth_header(&alice))
                .await;

            let status = response.status_code();
            let body = response.text();
            assert_eq!(
                status, 404,
                "expected 404 when deleting a client outside the URL realm, body: {body}",
            );

            // The oracle doubles as the survival check: it asserts a 200 itself.
            let client = read_victim_client(&server, &master, &victim.uuid).await;
            assert_eq!(
                client["id"].as_str(),
                Some(victim.uuid.as_str()),
                "FK-005: tenant-b's client did not survive the cross-realm delete. \
                 Attack returned HTTP {status}",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn tenant_admin_cannot_inject_a_redirect_uri_into_another_tenants_client() {
        // Registering a redirect URI on a victim client is the delivery half of an
        // authorization-code hijack.
        rt().block_on(async {
            let server = make_server();
            let master = master_token(&server).await;
            let victim = create_victim_client(&server, &master).await;
            let alice = alice_token(&server).await;

            let before = read_victim_redirect_uris(&server, &master, &victim.uuid).await;
            let hostile = "https://attacker.tenant-a.example/steal";

            let response = server
                .post(&format!(
                    "/realms/{TENANT_A}/clients/{}/redirects",
                    victim.uuid
                ))
                .add_header("Authorization", auth_header(&alice))
                .json(&json!({ "value": hostile, "enabled": true }))
                .await;

            let status = response.status_code();
            let body = response.text();
            assert_eq!(
                status, 404,
                "expected 404 when adding a redirect URI to a client outside the URL \
                 realm, body: {body}",
            );

            let after = read_victim_redirect_uris(&server, &master, &victim.uuid).await;
            assert!(
                !after.iter().any(|uri| uri == hostile),
                "FK-005: tenant-a injected {hostile} into tenant-b's client. \
                 Attack returned HTTP {status}; redirect URIs are now {after:?}",
            );
            assert_eq!(
                before, after,
                "the victim's redirect URI list changed. Attack returned HTTP {status}",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test client_cross_realm_test -- --ignored"]
    fn tenant_admin_retains_full_control_of_their_own_clients() {
        // Non-regression. Without this, a fix that simply denied every client
        // operation would look like a success above.
        rt().block_on(async {
            let server = make_server();
            let alice = alice_token(&server).await;
            let client_id = format!("own-{}", Uuid::new_v4().simple());

            let response = server
                .post(&format!("/realms/{TENANT_A}/clients"))
                .add_header("Authorization", auth_header(&alice))
                .json(&json!({
                    "client_id": client_id,
                    "name": "Tenant A own client",
                    "client_type": "confidential",
                    "protocol": "openid-connect",
                    "public_client": false,
                    "service_account_enabled": false,
                    "direct_access_grants_enabled": false,
                    "enabled": true,
                    "oauth_device_code_grant_enabled": false,
                }))
                .await;
            assert_eq!(
                response.status_code(),
                201,
                "alice could not create a client in her own realm: {}",
                response.text()
            );
            let body: Value = response.json();
            let uuid = body["id"].as_str().expect("own client id").to_string();

            // Read: she still sees her own client, secret included.
            let response = server
                .get(&format!("/realms/{TENANT_A}/clients/{uuid}"))
                .add_header("Authorization", auth_header(&alice))
                .await;
            assert_eq!(
                response.status_code(),
                200,
                "alice could not read her own client: {}",
                response.text()
            );
            let body: Value = response.json();
            assert_eq!(
                body["data"]["secret"], "***",
                "alice sees her own confidential client, with the secret redacted like everyone else's: {body}"
            );

            // Update.
            let response = server
                .patch(&format!("/realms/{TENANT_A}/clients/{uuid}"))
                .add_header("Authorization", auth_header(&alice))
                .json(&json!({ "require_pkce": true }))
                .await;
            assert_eq!(
                response.status_code(),
                200,
                "alice could not update her own client: {}",
                response.text()
            );
            let body: Value = response.json();
            assert_eq!(
                body["data"]["require_pkce"].as_bool(),
                Some(true),
                "alice's update did not take effect: {body}"
            );

            // Sub-resource: the realm binding must not break child writes.
            let response = server
                .post(&format!("/realms/{TENANT_A}/clients/{uuid}/redirects"))
                .add_header("Authorization", auth_header(&alice))
                .json(&json!({ "value": "https://tenant-a.example/callback", "enabled": true }))
                .await;
            assert_eq!(
                response.status_code(),
                201,
                "alice could not register a redirect URI on her own client: {}",
                response.text()
            );

            // Delete, then confirm it is really gone.
            let response = server
                .delete(&format!("/realms/{TENANT_A}/clients/{uuid}"))
                .add_header("Authorization", auth_header(&alice))
                .await;
            assert_eq!(
                response.status_code(),
                200,
                "alice could not delete her own client: {}",
                response.text()
            );

            let response = server
                .get(&format!("/realms/{TENANT_A}/clients/{uuid}"))
                .add_header("Authorization", auth_header(&alice))
                .await;
            assert_eq!(
                response.status_code(),
                404,
                "the deleted client is still readable: {}",
                response.text()
            );
        });
    }
}
