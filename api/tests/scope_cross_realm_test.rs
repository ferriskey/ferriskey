#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
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
    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";

    const ADMIN_CLI: &str = "admin-cli";

    const ALICE: &str = "alice";
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-Adm!";

    const READ_SCOPE: &str = "victim-read-scope";
    const UPDATE_SCOPE: &str = "victim-update-scope";
    const UPDATE_SCOPE_RENAMED: &str = "victim-update-scope-renamed-by-its-own-realm";
    const UPDATE_SCOPE_HIJACKED: &str = "victim-update-scope-hijacked-from-tenant-a";
    const DELETE_SCOPE: &str = "victim-delete-scope";
    const MAPPER_SCOPE: &str = "victim-mapper-scope";
    const ASSIGNED_SCOPE: &str = "victim-assigned-scope";

    const LOCAL_MAPPER_SCOPE: &str = "tenant-a-mapper-scope";
    const LOCAL_SIBLING_SCOPE: &str = "tenant-a-sibling-scope";
    const LOCAL_DISPOSABLE_SCOPE: &str = "tenant-a-disposable-scope";
    const LOCAL_ASSIGNABLE_SCOPE: &str = "tenant-a-assignable-scope";
    const LOCAL_DETACHABLE_SCOPE: &str = "tenant-a-detachable-scope";

    const WITNESS_SCOPE: &str = "tenant-a-claim-witness";
    const WITNESS_CLAIM: &str = "tenant_a_witness";
    const WITNESS_VALUE: &str = "issued-by-tenant-a";

    const FORGED_SCOPE: &str = "tenant-b-forged-claim";
    const FORGED_CLAIM: &str = "forged_witness";
    const FORGED_VALUE: &str = "smuggled-from-tenant-b";

    const OWNED_MAPPER: &str = "tenant-a-owned-mapper";
    const OWNED_MAPPER_RENAMED: &str = "tenant-a-owned-mapper-renamed";
    const HIJACKED_MAPPER: &str = "mapper-forged-from-tenant-a";
    const DOOMED_MAPPER: &str = "tenant-a-doomed-mapper";

    const HARDCODED_MAPPER_TYPE: &str = "oidc-hardcoded-claim-mapper";

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        tenant_a_client: Uuid,
        tenant_b_client: Uuid,
        read_scope: Uuid,
        update_scope: Uuid,
        delete_scope: Uuid,
        mapper_scope: Uuid,
        assigned_scope: Uuid,
        local_mapper_scope: Uuid,
        local_sibling_scope: Uuid,
        local_disposable_scope: Uuid,
        local_assignable_scope: Uuid,
        local_detachable_scope: Uuid,
        owned_mapper: Uuid,
        doomed_mapper: Uuid,
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

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    fn env_u16_or(key: &str, default: u16) -> u16 {
        env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
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

        let schema = format!("scope_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let server = TestServer::new(app.clone()).expect("create test server");
        let admin_token = password_grant(&server, MASTER_REALM, "admin", "admin").await;

        create_realm(&server, &admin_token, TENANT_A).await;
        create_realm(&server, &admin_token, TENANT_B).await;

        let alice_id = create_user(&server, &admin_token, TENANT_A, ALICE).await;
        set_password(&server, &admin_token, TENANT_A, alice_id, ALICE_PASSWORD).await;

        let tenant_a_client = client_uuid(&server, &admin_token, TENANT_A, ADMIN_CLI).await;
        let tenant_b_client = client_uuid(&server, &admin_token, TENANT_B, ADMIN_CLI).await;

        let read_scope = create_scope(&server, &admin_token, TENANT_B, READ_SCOPE).await;
        let update_scope = create_scope(&server, &admin_token, TENANT_B, UPDATE_SCOPE).await;
        let delete_scope = create_scope(&server, &admin_token, TENANT_B, DELETE_SCOPE).await;
        let mapper_scope = create_scope(&server, &admin_token, TENANT_B, MAPPER_SCOPE).await;
        let assigned_scope = create_scope(&server, &admin_token, TENANT_B, ASSIGNED_SCOPE).await;

        let local_mapper_scope =
            create_scope(&server, &admin_token, TENANT_A, LOCAL_MAPPER_SCOPE).await;
        let local_sibling_scope =
            create_scope(&server, &admin_token, TENANT_A, LOCAL_SIBLING_SCOPE).await;
        let local_disposable_scope =
            create_scope(&server, &admin_token, TENANT_A, LOCAL_DISPOSABLE_SCOPE).await;
        let local_assignable_scope =
            create_scope(&server, &admin_token, TENANT_A, LOCAL_ASSIGNABLE_SCOPE).await;
        let local_detachable_scope =
            create_scope(&server, &admin_token, TENANT_A, LOCAL_DETACHABLE_SCOPE).await;

        let owned_mapper = create_mapper_ok(
            &server,
            &admin_token,
            TENANT_A,
            local_mapper_scope,
            OWNED_MAPPER,
            &claim_config(WITNESS_CLAIM, WITNESS_VALUE),
        )
        .await;
        let doomed_mapper = create_mapper_ok(
            &server,
            &admin_token,
            TENANT_A,
            local_mapper_scope,
            DOOMED_MAPPER,
            &claim_config(WITNESS_CLAIM, WITNESS_VALUE),
        )
        .await;

        assert_success(
            &assign_default_scope(
                &server,
                &admin_token,
                TENANT_B,
                tenant_b_client,
                assigned_scope,
            )
            .await,
            "attaching a tenant-b scope to the tenant-b client",
        );
        assert_success(
            &assign_default_scope(
                &server,
                &admin_token,
                TENANT_A,
                tenant_a_client,
                local_detachable_scope,
            )
            .await,
            "attaching a tenant-a scope to the tenant-a client",
        );

        let witness_scope = create_scope(&server, &admin_token, TENANT_A, WITNESS_SCOPE).await;
        create_mapper_ok(
            &server,
            &admin_token,
            TENANT_A,
            witness_scope,
            "witness-claim",
            &claim_config(WITNESS_CLAIM, WITNESS_VALUE),
        )
        .await;
        assert_success(
            &assign_default_scope(
                &server,
                &admin_token,
                TENANT_A,
                tenant_a_client,
                witness_scope,
            )
            .await,
            "attaching the witness scope to the tenant-a client",
        );

        let forged_scope = create_scope(&server, &admin_token, TENANT_B, FORGED_SCOPE).await;
        create_mapper_ok(
            &server,
            &admin_token,
            TENANT_B,
            forged_scope,
            "forged-claim",
            &claim_config(FORGED_CLAIM, FORGED_VALUE),
        )
        .await;
        plant_mapping(&pool, tenant_a_client, forged_scope).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            tenant_a_client,
            tenant_b_client,
            read_scope,
            update_scope,
            delete_scope,
            mapper_scope,
            assigned_scope,
            local_mapper_scope,
            local_sibling_scope,
            local_disposable_scope,
            local_assignable_scope,
            local_detachable_scope,
            owned_mapper,
            doomed_mapper,
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

    fn claim_config(name: &str, value: &str) -> Value {
        json!({
            "claim.name": name,
            "claim.value": value,
            "jsonType.label": "String",
            "access.token.claim": "true",
            "id.token.claim": "true",
        })
    }

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
                ("client_id", ADMIN_CLI),
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
    ) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/users", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "username": username,
                "firstname": "Seeded",
                "lastname": "User",
                "email": format!("{username}@{realm}.local"),
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
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created user {username}: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("user id {raw} is not a uuid: {e}"))
    }

    async fn set_password(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        user_id: Uuid,
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

    async fn client_uuid(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_id: &str,
    ) -> Uuid {
        let response = server
            .get(&format!("/realms/{}/clients", realm))
            .add_header("Authorization", auth_header(admin_token))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "listing the clients of {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]
            .as_array()
            .and_then(|clients| {
                clients
                    .iter()
                    .find(|client| client["client_id"] == json!(client_id))
            })
            .and_then(|client| client["id"].as_str())
            .unwrap_or_else(|| panic!("no {client_id} client in {realm}: {body}"))
            .to_string();

        Uuid::parse_str(&raw).unwrap_or_else(|e| panic!("client id {raw} is not a uuid: {e}"))
    }

    async fn create_scope(server: &TestServer, admin_token: &str, realm: &str, name: &str) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/client-scopes", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "description": format!("{name} of {realm}"),
                "protocol": "openid-connect",
                "is_default": false,
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating the scope {name} in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created scope {name}: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("scope id {raw} is not a uuid: {e}"))
    }

    async fn get_scope(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
    ) -> TestResponse {
        server
            .get(&format!("/realms/{}/client-scopes/{}", realm, scope_id))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn rename_scope(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
        name: &str,
    ) -> TestResponse {
        server
            .patch(&format!("/realms/{}/client-scopes/{}", realm, scope_id))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "name": name }))
            .await
    }

    async fn delete_scope(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
    ) -> TestResponse {
        server
            .delete(&format!("/realms/{}/client-scopes/{}", realm, scope_id))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn create_mapper(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
        name: &str,
        config: &Value,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/client-scopes/{}/protocol-mappers",
                realm, scope_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "mapper_type": HARDCODED_MAPPER_TYPE,
                "config": config,
            }))
            .await
    }

    async fn create_mapper_ok(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
        name: &str,
        config: &Value,
    ) -> Uuid {
        let response = create_mapper(server, admin_token, realm, scope_id, name, config).await;

        assert_eq!(
            response.status_code(),
            201,
            "creating the mapper {name} on {scope_id}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created mapper {name}: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("mapper id {raw} is not a uuid: {e}"))
    }

    async fn rename_mapper(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
        mapper_id: Uuid,
        name: &str,
    ) -> TestResponse {
        server
            .patch(&format!(
                "/realms/{}/client-scopes/{}/protocol-mappers/{}",
                realm, scope_id, mapper_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "name": name }))
            .await
    }

    async fn delete_mapper(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        scope_id: Uuid,
        mapper_id: Uuid,
    ) -> TestResponse {
        server
            .delete(&format!(
                "/realms/{}/client-scopes/{}/protocol-mappers/{}",
                realm, scope_id, mapper_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn assign_default_scope(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_id: Uuid,
        scope_id: Uuid,
    ) -> TestResponse {
        server
            .put(&format!(
                "/realms/{}/clients/{}/default-client-scopes/{}",
                realm, client_id, scope_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({}))
            .await
    }

    async fn unassign_default_scope(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_id: Uuid,
        scope_id: Uuid,
    ) -> TestResponse {
        server
            .delete(&format!(
                "/realms/{}/clients/{}/default-client-scopes/{}",
                realm, client_id, scope_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn client_scopes(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_id: Uuid,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{}/clients/{}/client-scopes",
                realm, client_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn plant_mapping(pool: &PgPool, client_id: Uuid, scope_id: Uuid) {
        sqlx::query(
            "INSERT INTO client_scope_mappings (client_id, client_scope_id, default_scope_type) \
             VALUES ($1, $2, 'DEFAULT')",
        )
        .bind(client_id)
        .bind(scope_id)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("plant the mapping of {scope_id} on {client_id}: {e}"));
    }

    fn access_token_payload(access_token: &str) -> Value {
        let parts: Vec<&str> = access_token.split('.').collect();
        assert_eq!(parts.len(), 3, "an access token has three JWT segments");

        let payload_bytes = URL_SAFE_NO_PAD
            .decode(parts[1])
            .expect("the access token payload is valid base64url");

        serde_json::from_slice(&payload_bytes).expect("the access token payload is valid JSON")
    }

    fn assert_success(response: &TestResponse, what: &str) {
        assert!(
            response.status_code().is_success(),
            "{what}: expected a success, got {} with body {}",
            response.status_code(),
            response.text()
        );
    }

    fn assert_not_found(response: &TestResponse, what: &str) {
        assert_eq!(
            response.status_code(),
            404,
            "{what}: expected 404, got {} with body {}",
            response.status_code(),
            response.text()
        );
    }

    fn assert_body_free_of(body: &str, needles: &[&str], what: &str) {
        for needle in needles {
            assert!(
                !body.contains(needle),
                "{what}: the response leaked {needle:?}; body was {body}"
            );
        }
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn reading_a_client_scope_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner = get_scope(&server, &ctx.admin_token, TENANT_B, ctx.read_scope).await;
            assert_success(&owner, "reading the scope from its own realm");
            assert!(
                owner.text().contains(READ_SCOPE),
                "the scope is not readable from its own realm, so the refusal below would prove \
                 nothing: {}",
                owner.text()
            );

            let response = get_scope(&server, &ctx.admin_token, TENANT_A, ctx.read_scope).await;
            let body = response.text();

            assert_not_found(&response, "reading a tenant-b scope from the tenant-a url");
            assert_body_free_of(
                &body,
                &[READ_SCOPE],
                "reading a tenant-b scope from the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn updating_a_client_scope_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner = rename_scope(
                &server,
                &ctx.admin_token,
                TENANT_B,
                ctx.update_scope,
                UPDATE_SCOPE_RENAMED,
            )
            .await;
            assert_success(&owner, "renaming the scope from its own realm");

            let response = rename_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.update_scope,
                UPDATE_SCOPE_HIJACKED,
            )
            .await;
            assert_not_found(&response, "renaming a tenant-b scope from the tenant-a url");

            let survivor = get_scope(&server, &ctx.admin_token, TENANT_B, ctx.update_scope).await;
            assert_success(&survivor, "reading the scope back from its own realm");
            let body = survivor.text();
            assert!(
                body.contains(UPDATE_SCOPE_RENAMED),
                "the refused rename must leave the tenant-b name in place: {body}"
            );
            assert_body_free_of(
                &body,
                &[UPDATE_SCOPE_HIJACKED],
                "the refused rename must not have been applied",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn deleting_a_client_scope_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owned = delete_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_disposable_scope,
            )
            .await;
            assert_success(&owned, "deleting a tenant-a scope from the tenant-a url");
            assert_not_found(
                &get_scope(
                    &server,
                    &ctx.admin_token,
                    TENANT_A,
                    ctx.local_disposable_scope,
                )
                .await,
                "reading a scope that was legitimately deleted",
            );

            let response =
                delete_scope(&server, &ctx.admin_token, TENANT_A, ctx.delete_scope).await;
            assert_not_found(&response, "deleting a tenant-b scope from the tenant-a url");

            let survivor = get_scope(&server, &ctx.admin_token, TENANT_B, ctx.delete_scope).await;
            assert_success(&survivor, "reading the scope back from its own realm");
            assert!(
                survivor.text().contains(DELETE_SCOPE),
                "the refused delete must leave the tenant-b scope in place: {}",
                survivor.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn grafting_a_protocol_mapper_onto_a_scope_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owned = create_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_sibling_scope,
                "tenant-a-legitimate-mapper",
                &claim_config(WITNESS_CLAIM, WITNESS_VALUE),
            )
            .await;
            assert_success(&owned, "creating a mapper on a scope of the url realm");

            let response = create_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.mapper_scope,
                HIJACKED_MAPPER,
                &claim_config(FORGED_CLAIM, FORGED_VALUE),
            )
            .await;
            assert_not_found(
                &response,
                "creating a mapper on a tenant-b scope from the tenant-a url",
            );

            let victim = get_scope(&server, &ctx.admin_token, TENANT_B, ctx.mapper_scope).await;
            assert_success(&victim, "reading the tenant-b scope from its own realm");
            assert_body_free_of(
                &victim.text(),
                &[HIJACKED_MAPPER, FORGED_CLAIM],
                "the refused mapper must not exist on the tenant-b scope",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn updating_a_protocol_mapper_through_a_scope_that_does_not_own_it_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner = rename_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_mapper_scope,
                ctx.owned_mapper,
                OWNED_MAPPER_RENAMED,
            )
            .await;
            assert_success(&owner, "renaming a mapper through the scope that owns it");

            let sibling = rename_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_sibling_scope,
                ctx.owned_mapper,
                HIJACKED_MAPPER,
            )
            .await;
            assert_not_found(
                &sibling,
                "renaming a mapper through another scope of the same realm",
            );

            let foreign = rename_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.mapper_scope,
                ctx.owned_mapper,
                HIJACKED_MAPPER,
            )
            .await;
            assert_not_found(
                &foreign,
                "renaming a mapper through a scope of another realm",
            );

            let survivor =
                get_scope(&server, &ctx.admin_token, TENANT_A, ctx.local_mapper_scope).await;
            assert_success(&survivor, "reading the owning scope back");
            let body = survivor.text();
            assert!(
                body.contains(OWNED_MAPPER_RENAMED),
                "the legitimate rename must still be the one in place: {body}"
            );
            assert_body_free_of(
                &body,
                &[HIJACKED_MAPPER],
                "the refused renames must not have been applied",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn deleting_a_protocol_mapper_through_a_scope_that_does_not_own_it_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let sibling = delete_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_sibling_scope,
                ctx.doomed_mapper,
            )
            .await;
            assert_not_found(
                &sibling,
                "deleting a mapper through another scope of the same realm",
            );

            let foreign = delete_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.mapper_scope,
                ctx.doomed_mapper,
            )
            .await;
            assert_not_found(
                &foreign,
                "deleting a mapper through a scope of another realm",
            );

            let survivor =
                get_scope(&server, &ctx.admin_token, TENANT_A, ctx.local_mapper_scope).await;
            assert_success(&survivor, "reading the owning scope back");
            assert!(
                survivor.text().contains(DOOMED_MAPPER),
                "the refused deletes must leave the mapper in place: {}",
                survivor.text()
            );

            let owner = delete_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.local_mapper_scope,
                ctx.doomed_mapper,
            )
            .await;
            assert_success(&owner, "deleting a mapper through the scope that owns it");

            let after =
                get_scope(&server, &ctx.admin_token, TENANT_A, ctx.local_mapper_scope).await;
            assert_success(
                &after,
                "reading the owning scope after the legitimate delete",
            );
            assert_body_free_of(
                &after.text(),
                &[DOOMED_MAPPER],
                "a mapper deleted through its own scope is gone",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn attaching_a_client_scope_of_another_realm_to_a_local_client_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owned = assign_default_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.tenant_a_client,
                ctx.local_assignable_scope,
            )
            .await;
            assert_success(&owned, "attaching a tenant-a scope to the tenant-a client");

            let listing =
                client_scopes(&server, &ctx.admin_token, TENANT_A, ctx.tenant_a_client).await;
            assert_success(&listing, "listing the scopes of the tenant-a client");
            assert!(
                listing.text().contains(LOCAL_ASSIGNABLE_SCOPE),
                "the legitimate attachment is not visible, so the refusal below would prove \
                 nothing: {}",
                listing.text()
            );

            let foreign_scope = assign_default_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.tenant_a_client,
                ctx.assigned_scope,
            )
            .await;
            assert_not_found(
                &foreign_scope,
                "attaching a tenant-b scope to the tenant-a client",
            );

            let foreign_client = assign_default_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.tenant_b_client,
                ctx.local_assignable_scope,
            )
            .await;
            assert_not_found(
                &foreign_client,
                "attaching a tenant-a scope to the tenant-b client from the tenant-a url",
            );

            let after =
                client_scopes(&server, &ctx.admin_token, TENANT_A, ctx.tenant_a_client).await;
            assert_success(&after, "listing the scopes of the tenant-a client again");
            assert_body_free_of(
                &after.text(),
                &[ASSIGNED_SCOPE],
                "no tenant-b scope may be attached to the tenant-a client",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn detaching_a_client_scope_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owned = unassign_default_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.tenant_a_client,
                ctx.local_detachable_scope,
            )
            .await;
            assert_success(
                &owned,
                "detaching a tenant-a scope from the tenant-a client",
            );

            let listing =
                client_scopes(&server, &ctx.admin_token, TENANT_A, ctx.tenant_a_client).await;
            assert_success(&listing, "listing the scopes of the tenant-a client");
            assert_body_free_of(
                &listing.text(),
                &[LOCAL_DETACHABLE_SCOPE],
                "a scope detached through its own realm is gone from the listing",
            );

            let foreign = unassign_default_scope(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.tenant_b_client,
                ctx.assigned_scope,
            )
            .await;
            assert_not_found(
                &foreign,
                "detaching a tenant-b scope from the tenant-b client from the tenant-a url",
            );

            let survivor =
                client_scopes(&server, &ctx.admin_token, TENANT_B, ctx.tenant_b_client).await;
            assert_success(&survivor, "listing the scopes of the tenant-b client");
            assert!(
                survivor.text().contains(ASSIGNED_SCOPE),
                "the refused detachment must leave the tenant-b mapping in place: {}",
                survivor.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn listing_the_client_scopes_of_a_client_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner =
                client_scopes(&server, &ctx.admin_token, TENANT_B, ctx.tenant_b_client).await;
            assert_success(&owner, "listing the scopes of the tenant-b client");
            assert!(
                owner.text().contains(ASSIGNED_SCOPE),
                "the tenant-b client has no listable scope, so the refusal below would prove \
                 nothing: {}",
                owner.text()
            );

            let response =
                client_scopes(&server, &ctx.admin_token, TENANT_A, ctx.tenant_b_client).await;
            let body = response.text();

            assert_not_found(
                &response,
                "listing the scopes of a tenant-b client from the tenant-a url",
            );
            assert_body_free_of(
                &body,
                &[ASSIGNED_SCOPE],
                "listing the scopes of a tenant-b client from the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test scope_cross_realm_test -- --ignored"]
    fn a_scope_of_another_realm_mapped_onto_a_local_client_does_not_reach_the_token() {
        rt().block_on(async {
            let server = make_server();
            let _ = ctx();

            let access_token = password_grant(&server, TENANT_A, ALICE, ALICE_PASSWORD).await;
            let payload = access_token_payload(&access_token);
            let rendered = payload.to_string();

            assert_eq!(
                payload[WITNESS_CLAIM],
                json!(WITNESS_VALUE),
                "the witness claim of a tenant-a default scope is missing, so the absence of the \
                 forged claim below would prove nothing: {rendered}"
            );

            assert!(
                payload[FORGED_CLAIM].is_null(),
                "a tenant-b scope mapped onto the tenant-a client reached the token: {rendered}"
            );
            assert_body_free_of(
                &rendered,
                &[FORGED_CLAIM, FORGED_VALUE, FORGED_SCOPE],
                "a tenant-b scope mapped onto the tenant-a client",
            );

            let listing =
                client_scopes(&server, &ctx().admin_token, TENANT_A, ctx().tenant_a_client).await;
            assert_success(&listing, "listing the scopes of the tenant-a client");
            assert!(
                listing.text().contains(WITNESS_SCOPE),
                "the witness scope is not attached, so the absence below would prove nothing: {}",
                listing.text()
            );
            assert_body_free_of(
                &listing.text(),
                &[FORGED_SCOPE],
                "a tenant-b scope mapped onto the tenant-a client",
            );
        });
    }
}
