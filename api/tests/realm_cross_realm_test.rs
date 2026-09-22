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
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const MASTER_REALM: &str = "master";
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-Adm!";
    const READER_PASSWORD: &str = "R3ader-Tenant-A!!";

    const TENANT_B_SMTP_HOST: &str = "smtp.tenant-b.test";
    const TENANT_A_SMTP_HOST: &str = "smtp.tenant-a.test";
    const ATTACKER_SMTP_HOST: &str = "smtp.attacker.test";

    const TENANT_B_TOKEN_LIFETIME: i64 = 4242;
    const TENANT_A_TOKEN_LIFETIME: i64 = 1111;
    const ATTACKER_TOKEN_LIFETIME: i64 = 9999;

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
        tenant_a: String,
        tenant_b: String,
        admin_token: String,
        alice_token: String,
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

        let schema = format!("realm_cross_realm_test_{}", Uuid::new_v4().simple());

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
        let tenant_a = format!("tenant-a-{}", &suffix[..8]);
        let tenant_b = format!("tenant-b-{}", &suffix[..8]);

        create_realm(&server, &admin_token, &tenant_a).await;
        create_realm(&server, &admin_token, &tenant_b).await;

        let alice_username = format!("alice-{}", &suffix[..8]);
        let alice_id = create_user(&server, &admin_token, &tenant_a, &alice_username).await;
        set_password(&server, &admin_token, &tenant_a, &alice_id, ALICE_PASSWORD).await;

        let role_id = create_role(
            &server,
            &admin_token,
            &tenant_a,
            &format!("tenant-a-realm-admin-{}", &suffix[..8]),
            &["manage_realm", "view_realm"],
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

    fn alice_token() -> &'static str {
        shared_ctx().alice_token.as_str()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
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
            .unwrap_or_else(|| panic!("access_token in response for {username}@{realm}: {body}"))
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
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{}@test.local", username),
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

    async fn set_password(
        server: &TestServer,
        token: &str,
        realm: &str,
        user_id: &str,
        password: &str,
    ) {
        let response = server
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
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "set password for {user_id} in {realm} failed: {}",
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
            .post(&format!("/realms/{}/roles", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "name": name,
                "description": "realm administration inside one tenant",
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

    async fn get_realm(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn rename_realm(
        server: &TestServer,
        token: &str,
        realm: &str,
        new_name: &str,
    ) -> TestResponse {
        server
            .put(&format!("/realms/{}", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "name": new_name }))
            .await
    }

    async fn delete_realm(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .delete(&format!("/realms/{}", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn get_settings(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/users/@me/realms/settings", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn set_access_token_lifetime(
        server: &TestServer,
        token: &str,
        realm: &str,
        lifetime: i64,
    ) -> TestResponse {
        server
            .put(&format!("/realms/{}/settings", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({ "access_token_lifetime": lifetime }))
            .await
    }

    async fn get_smtp_config(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/smtp-config", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn upsert_smtp_config(
        server: &TestServer,
        token: &str,
        realm: &str,
        host: &str,
    ) -> TestResponse {
        server
            .put(&format!("/realms/{}/smtp-config", realm))
            .add_header("Authorization", auth_header(token))
            .json(&json!({
                "host": host,
                "port": 587,
                "username": "mailer",
                "password": "mailer-secret",
                "from_email": "noreply@test.local",
                "from_name": "FerrisKey",
                "encryption": "starttls",
            }))
            .await
    }

    async fn delete_smtp_config(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .delete(&format!("/realms/{}/smtp-config", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn login_settings(server: &TestServer, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/login-settings", realm))
            .await
    }

    async fn my_realms(server: &TestServer, token: &str, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/users/@me/realms", realm))
            .add_header("Authorization", auth_header(token))
            .await
    }

    async fn plant_realm(server: &TestServer) -> String {
        let name = format!("victim-{}", &Uuid::new_v4().simple().to_string()[..12]);
        create_realm(server, admin_token(), &name).await;
        name
    }

    fn settings_lifetime(response: &TestResponse) -> i64 {
        let body: Value = response.json();
        body["access_token_lifetime"]
            .as_i64()
            .unwrap_or_else(|| panic!("access_token_lifetime in realm settings: {body}"))
    }

    fn smtp_host(response: &TestResponse) -> String {
        let body: Value = response.json();
        body["host"]
            .as_str()
            .unwrap_or_else(|| panic!("host in smtp config: {body}"))
            .to_string()
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_tenant_administrator_reads_her_own_realm_settings_but_not_another_tenants() {
        rt().block_on(async {
            let server = make_server();

            let own = get_settings(&server, alice_token(), tenant_a()).await;
            assert_eq!(
                own.status_code(),
                200,
                "alice must be able to read the settings of her own realm: {}",
                own.text()
            );
            let own_body: Value = own.json();
            assert!(
                own_body["id"].as_str().is_some_and(|id| !id.is_empty()),
                "the legitimate read must return a real settings row, not an empty document: {own_body}"
            );

            let foreign = get_settings(&server, alice_token(), tenant_b()).await;
            assert_eq!(
                foreign.status_code(),
                404,
                "another tenant's realm settings must read as absent, not as a refusal that confirms the realm exists: {}",
                foreign.text()
            );

            let control = get_settings(&server, admin_token(), tenant_b()).await;
            assert_eq!(
                control.status_code(),
                200,
                "the master administrator must still read the same settings, otherwise the refusal above proves nothing: {}",
                control.text()
            );
            let control_body: Value = control.json();
            assert!(
                control_body["id"]
                    .as_str()
                    .is_some_and(|id| !id.is_empty()),
                "the settings alice was refused must actually exist: {control_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_tenant_administrator_cannot_change_another_tenants_realm_settings() {
        rt().block_on(async {
            let server = make_server();

            let seeded =
                set_access_token_lifetime(&server, admin_token(), tenant_b(), TENANT_B_TOKEN_LIFETIME)
                    .await;
            assert_eq!(
                seeded.status_code(),
                200,
                "seeding tenant-b's token lifetime as master failed: {}",
                seeded.text()
            );
            let before = get_settings(&server, admin_token(), tenant_b()).await;
            assert_eq!(before.status_code(), 200, "{}", before.text());
            assert_eq!(
                settings_lifetime(&before),
                TENANT_B_TOKEN_LIFETIME,
                "the fixture must be in place before the attack"
            );

            let legitimate = set_access_token_lifetime(
                &server,
                alice_token(),
                tenant_a(),
                TENANT_A_TOKEN_LIFETIME,
            )
            .await;
            assert_eq!(
                legitimate.status_code(),
                200,
                "alice must be able to change the settings of her own realm: {}",
                legitimate.text()
            );
            let own = get_settings(&server, alice_token(), tenant_a()).await;
            assert_eq!(own.status_code(), 200, "{}", own.text());
            assert_eq!(
                settings_lifetime(&own),
                TENANT_A_TOKEN_LIFETIME,
                "the legitimate write must have landed, otherwise the refusal below is not a comparable operation"
            );

            let attack = set_access_token_lifetime(
                &server,
                alice_token(),
                tenant_b(),
                ATTACKER_TOKEN_LIFETIME,
            )
            .await;
            assert_eq!(
                attack.status_code(),
                404,
                "another tenant's realm settings must read as absent, not as a refusal that confirms the realm exists: {}",
                attack.text()
            );

            let after = get_settings(&server, admin_token(), tenant_b()).await;
            assert_eq!(after.status_code(), 200, "{}", after.text());
            assert_eq!(
                settings_lifetime(&after),
                TENANT_B_TOKEN_LIFETIME,
                "the refused write must not have reached the row"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_tenant_administrator_cannot_reach_another_tenants_smtp_configuration() {
        rt().block_on(async {
            let server = make_server();

            let seeded =
                upsert_smtp_config(&server, admin_token(), tenant_b(), TENANT_B_SMTP_HOST).await;
            assert_eq!(
                seeded.status_code(),
                200,
                "seeding tenant-b's SMTP configuration as master failed: {}",
                seeded.text()
            );
            let before = get_smtp_config(&server, admin_token(), tenant_b()).await;
            assert_eq!(before.status_code(), 200, "{}", before.text());
            assert_eq!(smtp_host(&before), TENANT_B_SMTP_HOST);

            let legitimate =
                upsert_smtp_config(&server, alice_token(), tenant_a(), TENANT_A_SMTP_HOST).await;
            assert_eq!(
                legitimate.status_code(),
                200,
                "alice must be able to configure SMTP in her own realm: {}",
                legitimate.text()
            );
            let own = get_smtp_config(&server, alice_token(), tenant_a()).await;
            assert_eq!(
                own.status_code(),
                200,
                "alice must be able to read back her own realm's SMTP configuration: {}",
                own.text()
            );
            assert_eq!(
                smtp_host(&own),
                TENANT_A_SMTP_HOST,
                "the legitimate read must return the row alice just wrote"
            );

            let read = get_smtp_config(&server, alice_token(), tenant_b()).await;
            assert_eq!(
                read.status_code(),
                404,
                "another tenant's SMTP credentials must read as absent, not as a refusal that confirms the realm exists: {}",
                read.text()
            );

            let write =
                upsert_smtp_config(&server, alice_token(), tenant_b(), ATTACKER_SMTP_HOST).await;
            assert_eq!(
                write.status_code(),
                404,
                "redirecting another tenant's mail must read as absent, not as a refusal that confirms the realm exists: {}",
                write.text()
            );

            let removal = delete_smtp_config(&server, alice_token(), tenant_b()).await;
            assert_eq!(
                removal.status_code(),
                404,
                "deleting another tenant's SMTP configuration must read as absent, not as a refusal that confirms the realm exists: {}",
                removal.text()
            );

            let after = get_smtp_config(&server, admin_token(), tenant_b()).await;
            assert_eq!(
                after.status_code(),
                200,
                "the refused deletion must have left the row in place: {}",
                after.text()
            );
            assert_eq!(
                smtp_host(&after),
                TENANT_B_SMTP_HOST,
                "the refused write must not have reached the row"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_tenant_administrator_cannot_read_or_rename_another_realm() {
        rt().block_on(async {
            let server = make_server();

            let own = get_realm(&server, alice_token(), tenant_a()).await;
            assert_eq!(
                own.status_code(),
                200,
                "alice must be able to read her own realm: {}",
                own.text()
            );
            let own_body: Value = own.json();
            assert_eq!(
                own_body["name"].as_str(),
                Some(tenant_a()),
                "the legitimate read must return alice's own realm: {own_body}"
            );

            let read = get_realm(&server, alice_token(), tenant_b()).await;
            assert_eq!(
                read.status_code(),
                404,
                "another realm must read as absent, not as a refusal that confirms it exists: {}",
                read.text()
            );

            let renamed = format!("stolen-{}", &Uuid::new_v4().simple().to_string()[..8]);
            let write = rename_realm(&server, alice_token(), tenant_b(), &renamed).await;
            assert_eq!(
                write.status_code(),
                404,
                "renaming another realm must read as absent, not as a refusal that confirms it exists: {}",
                write.text()
            );

            let after = get_realm(&server, admin_token(), tenant_b()).await;
            assert_eq!(
                after.status_code(),
                200,
                "the target realm must have survived the refused rename: {}",
                after.text()
            );
            let after_body: Value = after.json();
            assert_eq!(
                after_body["name"].as_str(),
                Some(tenant_b()),
                "the refused rename must not have reached the row: {after_body}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_tenant_administrator_cannot_delete_another_realm() {
        rt().block_on(async {
            let server = make_server();

            let victim = plant_realm(&server).await;
            let planted = get_realm(&server, admin_token(), &victim).await;
            assert_eq!(
                planted.status_code(),
                200,
                "the realm to be attacked must exist first: {}",
                planted.text()
            );

            let attack = delete_realm(&server, alice_token(), &victim).await;
            assert_eq!(
                attack.status_code(),
                404,
                "deleting another realm must read as absent, not as a refusal that confirms it exists: {}",
                attack.text()
            );

            let survivor = get_realm(&server, admin_token(), &victim).await;
            assert_eq!(
                survivor.status_code(),
                200,
                "the refused deletion must have left the realm in place: {}",
                survivor.text()
            );
            let survivor_body: Value = survivor.json();
            assert_eq!(
                survivor_body["name"].as_str(),
                Some(victim.as_str()),
                "the realm alice was refused must still be the same one: {survivor_body}"
            );

            let legitimate = delete_realm(&server, admin_token(), &victim).await;
            assert_eq!(
                legitimate.status_code(),
                200,
                "the master administrator must still be able to delete the realm: {}",
                legitimate.text()
            );
            let gone = get_realm(&server, admin_token(), &victim).await;
            assert_eq!(
                gone.status_code(),
                404,
                "a deleted realm must no longer resolve: {}",
                gone.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn login_settings_are_public_per_realm_and_an_unknown_realm_is_refused_with_404() {
        rt().block_on(async {
            let server = make_server();

            let known = login_settings(&server, tenant_a()).await;
            assert_eq!(
                known.status_code(),
                200,
                "the login settings of an existing realm are public: {}",
                known.text()
            );
            let known_body: Value = known.json();
            assert_eq!(
                known_body["name"].as_str(),
                Some(tenant_a()),
                "the public login settings must describe the realm that was asked for: {known_body}"
            );

            let unknown = format!("ghost-{}", &Uuid::new_v4().simple().to_string()[..8]);
            let refused = login_settings(&server, &unknown).await;
            assert_eq!(
                refused.status_code(),
                404,
                "an unknown realm must be refused with 404, so that the status code does not tell a stranger which tenants exist: {}",
                refused.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn a_missing_right_at_home_stays_forbidden_while_a_foreign_realm_reads_as_absent() {
        rt().block_on(async {
            let server = make_server();

            let suffix = Uuid::new_v4().simple().to_string();
            let username = format!("reader-{}", &suffix[..8]);
            let user_id = create_user(&server, admin_token(), tenant_a(), &username).await;
            set_password(
                &server,
                admin_token(),
                tenant_a(),
                &user_id,
                READER_PASSWORD,
            )
            .await;

            let role_id = create_role(
                &server,
                admin_token(),
                tenant_a(),
                &format!("tenant-a-user-reader-{}", &suffix[..8]),
                &["view_users"],
            )
            .await;
            assign_role(&server, admin_token(), tenant_a(), &user_id, &role_id).await;

            let reader_token = direct_grant(&server, tenant_a(), &username, READER_PASSWORD).await;

            let witness = get_realm(&server, alice_token(), tenant_a()).await;
            assert_eq!(
                witness.status_code(),
                200,
                "the same read must succeed for a caller who holds the right, otherwise the refusals below prove nothing: {}",
                witness.text()
            );

            let missing_right = get_realm(&server, &reader_token, tenant_a()).await;
            assert_eq!(
                missing_right.status_code(),
                403,
                "a member of the realm who merely lacks view_realm must still be told so: hiding an ordinary permission refusal behind a 404 would be a regression: {}",
                missing_right.text()
            );

            let foreign = get_realm(&server, alice_token(), tenant_b()).await;
            assert_eq!(
                foreign.status_code(),
                404,
                "a realm the caller may not reach must read as absent: {}",
                foreign.text()
            );

            let ghost = format!("ghost-{}", &Uuid::new_v4().simple().to_string()[..8]);
            let unknown = get_realm(&server, alice_token(), &ghost).await;
            assert_eq!(
                unknown.status_code(),
                404,
                "an unknown realm must answer exactly as a foreign one: {}",
                unknown.text()
            );
            assert_eq!(
                unknown.text(),
                foreign.text(),
                "the two refusals must be byte-identical, otherwise the status code closed the enumeration oracle and the body reopened it"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test realm_cross_realm_test -- --ignored"]
    fn the_master_administrators_realm_listing_names_every_tenant() {
        rt().block_on(async {
            let server = make_server();

            let response = my_realms(&server, admin_token(), MASTER_REALM).await;
            assert_eq!(
                response.status_code(),
                200,
                "the master administrator must be able to list the realms he administers: {}",
                response.text()
            );

            let body: Value = response.json();
            let realms = body["data"]
                .as_array()
                .unwrap_or_else(|| panic!("data array in realm listing: {body}"));
            assert!(
                !realms.is_empty(),
                "an empty listing would make every membership assertion below vacuous: {body}"
            );

            let names: Vec<&str> = realms
                .iter()
                .filter_map(|realm| realm["name"].as_str())
                .collect();
            assert!(
                names.contains(&tenant_a()),
                "the master administrator created tenant-a and must see it: {names:?}"
            );
            assert!(
                names.contains(&tenant_b()),
                "the master administrator created tenant-b and must see it: {names:?}"
            );
        });
    }
}
