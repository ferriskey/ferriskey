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

    const MASTER_REALM: &str = "master";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";

    const READ_IDP_ALIAS: &str = "victim-read-idp";
    const READ_IDP_NAME: &str = "Tenant B Read Provider";

    const UPDATE_IDP_ALIAS: &str = "victim-update-idp";
    const UPDATE_IDP_NAME: &str = "Tenant B Update Provider";
    const UPDATE_IDP_RENAMED: &str = "Renamed By Its Own Realm";
    const UPDATE_IDP_HIJACKED: &str = "Hijacked From Tenant A";

    const DELETE_IDP_ALIAS: &str = "victim-delete-idp";
    const DELETE_IDP_NAME: &str = "Tenant B Delete Provider";

    const SHARED_IDP_ALIAS: &str = "shared-idp";
    const SHARED_IDP_NAME_A: &str = "Tenant A Shared Provider";
    const SHARED_IDP_NAME_B: &str = "Tenant B Shared Provider";
    const SHARED_IDP_RENAMED_A: &str = "Tenant A Shared Provider Renamed";

    const BROKER_IDP_ALIAS: &str = "broker-idp";

    const TENANT_A_CLIENT_ID: &str = "tenant-a-broker-client";
    const TENANT_B_CLIENT_ID: &str = "tenant-b-broker-client";
    const TENANT_A_REDIRECT_URI: &str = "https://tenant-a.example/callback";
    const TENANT_B_REDIRECT_URI: &str = "https://tenant-b.example/callback";

    const READ_DIRECTORY_NAME: &str = "Tenant B Read Directory";
    const UPDATE_DIRECTORY_NAME: &str = "Tenant B Update Directory";
    const UPDATE_DIRECTORY_RENAMED: &str = "Tenant B Update Directory Renamed";
    const UPDATE_DIRECTORY_HIJACKED: &str = "Tenant B Update Directory Pwned";
    const DELETE_DIRECTORY_NAME: &str = "Tenant B Delete Directory";
    const TENANT_A_DIRECTORY_NAME: &str = "Tenant A Directory";

    const SESSION_ABSENT_MESSAGE: &str = "Invalid or expired SSO session";

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
        admin_token: String,
        read_provider_id: String,
        update_provider_id: String,
        delete_provider_id: String,
        tenant_a_provider_id: String,
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

        let schema = format!("abyss_cross_realm_test_{}", Uuid::new_v4().simple());

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

        create_identity_provider(
            &server,
            &admin_token,
            TENANT_B,
            READ_IDP_ALIAS,
            READ_IDP_NAME,
        )
        .await;
        create_identity_provider(
            &server,
            &admin_token,
            TENANT_B,
            UPDATE_IDP_ALIAS,
            UPDATE_IDP_NAME,
        )
        .await;
        create_identity_provider(
            &server,
            &admin_token,
            TENANT_B,
            DELETE_IDP_ALIAS,
            DELETE_IDP_NAME,
        )
        .await;
        create_identity_provider(
            &server,
            &admin_token,
            TENANT_B,
            SHARED_IDP_ALIAS,
            SHARED_IDP_NAME_B,
        )
        .await;
        create_identity_provider(
            &server,
            &admin_token,
            TENANT_A,
            SHARED_IDP_ALIAS,
            SHARED_IDP_NAME_A,
        )
        .await;

        create_broker_identity_provider(&server, &admin_token, TENANT_A).await;
        create_broker_identity_provider(&server, &admin_token, TENANT_B).await;

        create_broker_client(
            &server,
            &admin_token,
            TENANT_A,
            TENANT_A_CLIENT_ID,
            TENANT_A_REDIRECT_URI,
        )
        .await;
        create_broker_client(
            &server,
            &admin_token,
            TENANT_B,
            TENANT_B_CLIENT_ID,
            TENANT_B_REDIRECT_URI,
        )
        .await;

        let read_provider_id =
            create_federation_provider(&server, &admin_token, TENANT_B, READ_DIRECTORY_NAME).await;
        let update_provider_id =
            create_federation_provider(&server, &admin_token, TENANT_B, UPDATE_DIRECTORY_NAME)
                .await;
        let delete_provider_id =
            create_federation_provider(&server, &admin_token, TENANT_B, DELETE_DIRECTORY_NAME)
                .await;
        let tenant_a_provider_id =
            create_federation_provider(&server, &admin_token, TENANT_A, TENANT_A_DIRECTORY_NAME)
                .await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            read_provider_id,
            update_provider_id,
            delete_provider_id,
            tenant_a_provider_id,
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

    async fn create_identity_provider(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        alias: &str,
        display_name: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/identity-providers", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "alias": alias,
                "provider_id": "oidc",
                "enabled": true,
                "display_name": display_name,
                "store_token": false,
                "add_read_token_role_on_create": false,
                "trust_email": false,
                "link_only": false,
                "config": {
                    "client_id": format!("{}-client", alias),
                    "client_secret": format!("{}-secret", alias),
                    "authorization_url": format!("https://idp.{}.example/authorize", realm),
                    "token_url": format!("https://idp.{}.example/token", realm),
                    "scopes": ["openid", "email"],
                    "use_pkce": false,
                },
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating identity provider {alias}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["internal_id"]
            .as_str()
            .unwrap_or_else(|| panic!("no internal_id for created provider {alias}: {body}"))
            .to_string()
    }

    async fn create_broker_identity_provider(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
    ) -> String {
        create_identity_provider(
            server,
            admin_token,
            realm,
            BROKER_IDP_ALIAS,
            &format!("{} Broker Provider", realm),
        )
        .await
    }

    async fn create_broker_client(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_id: &str,
        redirect_uri: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/clients", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "client_id": client_id,
                "name": format!("{} broker client", realm),
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
            "creating client {client_id}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let uuid = body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created client {client_id}: {body}"))
            .to_string();

        let response = server
            .post(&format!("/realms/{}/clients/{}/redirects", realm, uuid))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({ "value": redirect_uri, "enabled": true }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "registering the redirect uri of {client_id}@{realm} failed: {}",
            response.text()
        );

        uuid
    }

    async fn create_federation_provider(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
    ) -> String {
        let response = server
            .post(&format!("/realms/{}/federation/providers", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "provider_type": "Ldap",
                "enabled": true,
                "priority": 0,
                "config": { "note": "deliberately incomplete, no directory is ever reached" },
                "sync_enabled": false,
                "sync_mode": "LinkOnly",
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating federation provider {name}@{realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        body["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for created federation provider {name}: {body}"))
            .to_string()
    }

    async fn get_identity_provider(server: &TestServer, realm: &str, alias: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/identity-providers/{}", realm, alias))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn identity_provider_text(server: &TestServer, realm: &str, alias: &str) -> String {
        let response = get_identity_provider(server, realm, alias).await;

        assert_eq!(
            response.status_code(),
            200,
            "the master admin could not read {alias}@{realm}: {}",
            response.text()
        );

        response.text()
    }

    async fn get_federation_provider(server: &TestServer, realm: &str, id: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/federation/providers/{}", realm, id))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn federation_provider_text(server: &TestServer, realm: &str, id: &str) -> String {
        let response = get_federation_provider(server, realm, id).await;

        assert_eq!(
            response.status_code(),
            200,
            "the master admin could not read federation provider {id}@{realm}: {}",
            response.text()
        );

        response.text()
    }

    async fn broker_login_location(
        server: &TestServer,
        realm: &str,
        alias: &str,
        client_id: &str,
        redirect_uri: &str,
    ) -> TestResponse {
        server
            .get(&format!("/realms/{}/broker/{}/login", realm, alias))
            .add_query_param("client_id", client_id)
            .add_query_param("redirect_uri", redirect_uri)
            .add_query_param("response_type", "code")
            .add_query_param("scope", "openid")
            .await
    }

    async fn issue_broker_state(server: &TestServer) -> String {
        let response = broker_login_location(
            server,
            TENANT_B,
            BROKER_IDP_ALIAS,
            TENANT_B_CLIENT_ID,
            TENANT_B_REDIRECT_URI,
        )
        .await;

        assert_eq!(
            response.status_code(),
            302,
            "initiating a broker login in {TENANT_B} failed: {}",
            response.text()
        );

        let location = response
            .headers()
            .get("location")
            .expect("a broker login redirects to the identity provider")
            .to_str()
            .expect("Location is valid UTF-8")
            .to_string();

        broker_state_of(&location)
    }

    fn broker_state_of(location: &str) -> String {
        let marker = "&state=";
        let start = location
            .find(marker)
            .unwrap_or_else(|| panic!("the authorization url carries no state: {location}"))
            + marker.len();
        let rest = &location[start..];
        let end = rest.find('&').unwrap_or(rest.len());
        let state = urlencoding::decode(&rest[..end])
            .expect("the state parameter is valid percent encoding")
            .into_owned();

        assert!(
            !state.is_empty(),
            "the authorization url carries an empty state: {location}"
        );

        state
    }

    async fn broker_callback(
        server: &TestServer,
        realm: &str,
        state: &str,
        outcome: &[(&str, &str)],
    ) -> TestResponse {
        let mut request = server
            .get(&format!(
                "/realms/{}/broker/{}/endpoint",
                realm, BROKER_IDP_ALIAS
            ))
            .add_query_param("state", state);

        for (key, value) in outcome {
            request = request.add_query_param(key, value);
        }

        request.await
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

    fn assert_body_free_of(response_body: &str, needles: &[&str], what: &str) {
        for needle in needles {
            assert!(
                !response_body.contains(needle),
                "{what}: the response leaked {needle:?}; body was {response_body}"
            );
        }
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn reading_an_identity_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let witness = identity_provider_text(&server, TENANT_B, READ_IDP_ALIAS).await;
            assert!(
                witness.contains(READ_IDP_NAME),
                "the provider is not readable from its own realm, so the refusal below would \
                 prove nothing: {witness}"
            );

            let response = get_identity_provider(&server, TENANT_A, READ_IDP_ALIAS).await;
            let body = response.text();

            assert_not_found(
                &response,
                "reading a tenant-b identity provider from the tenant-a url",
            );
            assert_body_free_of(
                &body,
                &[READ_IDP_NAME],
                "read of a tenant-b identity provider from the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn updating_an_identity_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let before = identity_provider_text(&server, TENANT_B, UPDATE_IDP_ALIAS).await;
            assert!(
                before.contains(UPDATE_IDP_NAME),
                "the provider is not readable from its own realm, so the assertions below would \
                 prove nothing: {before}"
            );

            let refused = server
                .put(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_A, UPDATE_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .json(&json!({ "display_name": UPDATE_IDP_HIJACKED }))
                .await;

            assert_not_found(
                &refused,
                "updating a tenant-b identity provider from the tenant-a url",
            );

            let after = identity_provider_text(&server, TENANT_B, UPDATE_IDP_ALIAS).await;
            assert!(
                after.contains(UPDATE_IDP_NAME),
                "the tenant-b provider lost its display name: {after}"
            );
            assert_body_free_of(
                &after,
                &[UPDATE_IDP_HIJACKED],
                "the tenant-b provider after the refused foreign update",
            );

            let accepted = server
                .put(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_B, UPDATE_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .json(&json!({ "display_name": UPDATE_IDP_RENAMED }))
                .await;

            assert_eq!(
                accepted.status_code(),
                200,
                "a tenant-b provider must be updatable from the tenant-b url: {} {}",
                accepted.status_code(),
                accepted.text()
            );

            let renamed = identity_provider_text(&server, TENANT_B, UPDATE_IDP_ALIAS).await;
            assert!(
                renamed.contains(UPDATE_IDP_RENAMED),
                "the accepted update did not land, so the refusal above is not evidence of \
                 scoping: {renamed}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn deleting_an_identity_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let before = identity_provider_text(&server, TENANT_B, DELETE_IDP_ALIAS).await;
            assert!(
                before.contains(DELETE_IDP_NAME),
                "the provider is not readable from its own realm, so the assertions below would \
                 prove nothing: {before}"
            );

            let refused = server
                .delete(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_A, DELETE_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;

            assert_not_found(
                &refused,
                "deleting a tenant-b identity provider from the tenant-a url",
            );

            let survived = identity_provider_text(&server, TENANT_B, DELETE_IDP_ALIAS).await;
            assert!(
                survived.contains(DELETE_IDP_NAME),
                "the tenant-b provider was deleted through the tenant-a url: {survived}"
            );

            let accepted = server
                .delete(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_B, DELETE_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;

            assert_eq!(
                accepted.status_code(),
                200,
                "a tenant-b provider must be deletable from the tenant-b url: {} {}",
                accepted.status_code(),
                accepted.text()
            );

            let gone = get_identity_provider(&server, TENANT_B, DELETE_IDP_ALIAS).await;
            assert_not_found(
                &gone,
                "the accepted deletion did not land, so the refusal above is not evidence of \
                 scoping",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn an_identity_provider_sharing_an_alias_with_another_realm_is_untouched() {
        rt().block_on(async {
            let server = make_server();

            let tenant_b_before = identity_provider_text(&server, TENANT_B, SHARED_IDP_ALIAS).await;
            assert!(
                tenant_b_before.contains(SHARED_IDP_NAME_B),
                "both realms must hold a provider under this alias for the test to mean \
                 anything: {tenant_b_before}"
            );
            let tenant_a_before = identity_provider_text(&server, TENANT_A, SHARED_IDP_ALIAS).await;
            assert!(
                tenant_a_before.contains(SHARED_IDP_NAME_A),
                "both realms must hold a provider under this alias for the test to mean \
                 anything: {tenant_a_before}"
            );

            let updated = server
                .put(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_A, SHARED_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .json(&json!({ "display_name": SHARED_IDP_RENAMED_A }))
                .await;

            assert_eq!(
                updated.status_code(),
                200,
                "tenant-a must be able to update its own namesake provider: {} {}",
                updated.status_code(),
                updated.text()
            );

            let tenant_a_after = identity_provider_text(&server, TENANT_A, SHARED_IDP_ALIAS).await;
            assert!(
                tenant_a_after.contains(SHARED_IDP_RENAMED_A),
                "the update did not reach tenant-a's own provider: {tenant_a_after}"
            );

            let tenant_b_after = identity_provider_text(&server, TENANT_B, SHARED_IDP_ALIAS).await;
            assert!(
                tenant_b_after.contains(SHARED_IDP_NAME_B),
                "tenant-b's namesake provider lost its display name: {tenant_b_after}"
            );
            assert_body_free_of(
                &tenant_b_after,
                &[SHARED_IDP_RENAMED_A, SHARED_IDP_NAME_A],
                "tenant-b's namesake provider after tenant-a updated its own",
            );

            let deleted = server
                .delete(&format!(
                    "/realms/{}/identity-providers/{}",
                    TENANT_A, SHARED_IDP_ALIAS
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;

            assert_eq!(
                deleted.status_code(),
                200,
                "tenant-a must be able to delete its own namesake provider: {} {}",
                deleted.status_code(),
                deleted.text()
            );

            let tenant_a_gone = get_identity_provider(&server, TENANT_A, SHARED_IDP_ALIAS).await;
            assert_not_found(
                &tenant_a_gone,
                "tenant-a's own namesake provider survived its own deletion",
            );

            let tenant_b_survived =
                identity_provider_text(&server, TENANT_B, SHARED_IDP_ALIAS).await;
            assert!(
                tenant_b_survived.contains(SHARED_IDP_NAME_B),
                "deleting tenant-a's provider took tenant-b's namesake with it: \
                 {tenant_b_survived}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn reading_a_federation_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let witness =
                federation_provider_text(&server, TENANT_B, &ctx().read_provider_id).await;
            assert!(
                witness.contains(READ_DIRECTORY_NAME),
                "the provider is not readable from its own realm, so the refusal below would \
                 prove nothing: {witness}"
            );

            let response =
                get_federation_provider(&server, TENANT_A, &ctx().read_provider_id).await;
            let body = response.text();

            assert_not_found(
                &response,
                "reading a tenant-b federation provider from the tenant-a url",
            );
            assert_body_free_of(
                &body,
                &[READ_DIRECTORY_NAME, &ctx().read_provider_id],
                "read of a tenant-b federation provider from the tenant-a url",
            );

            let own =
                federation_provider_text(&server, TENANT_A, &ctx().tenant_a_provider_id).await;
            assert!(
                own.contains(TENANT_A_DIRECTORY_NAME),
                "tenant-a cannot read its own federation provider, so the 404 above may be the \
                 route rather than the realm: {own}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn updating_a_federation_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let before =
                federation_provider_text(&server, TENANT_B, &ctx().update_provider_id).await;
            assert!(
                before.contains(UPDATE_DIRECTORY_NAME),
                "the provider is not readable from its own realm, so the assertions below would \
                 prove nothing: {before}"
            );

            let refused = server
                .put(&format!(
                    "/realms/{}/federation/providers/{}",
                    TENANT_A,
                    ctx().update_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .json(&json!({ "name": UPDATE_DIRECTORY_HIJACKED }))
                .await;

            assert_not_found(
                &refused,
                "updating a tenant-b federation provider from the tenant-a url",
            );

            let after =
                federation_provider_text(&server, TENANT_B, &ctx().update_provider_id).await;
            assert!(
                after.contains(UPDATE_DIRECTORY_NAME),
                "the tenant-b federation provider lost its name: {after}"
            );
            assert_body_free_of(
                &after,
                &[UPDATE_DIRECTORY_HIJACKED],
                "the tenant-b federation provider after the refused foreign update",
            );

            let accepted = server
                .put(&format!(
                    "/realms/{}/federation/providers/{}",
                    TENANT_B,
                    ctx().update_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .json(&json!({ "name": UPDATE_DIRECTORY_RENAMED }))
                .await;

            assert_eq!(
                accepted.status_code(),
                200,
                "a tenant-b federation provider must be updatable from the tenant-b url: {} {}",
                accepted.status_code(),
                accepted.text()
            );

            let renamed =
                federation_provider_text(&server, TENANT_B, &ctx().update_provider_id).await;
            assert!(
                renamed.contains(UPDATE_DIRECTORY_RENAMED),
                "the accepted update did not land, so the refusal above is not evidence of \
                 scoping: {renamed}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn deleting_a_federation_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let before =
                federation_provider_text(&server, TENANT_B, &ctx().delete_provider_id).await;
            assert!(
                before.contains(DELETE_DIRECTORY_NAME),
                "the provider is not readable from its own realm, so the assertions below would \
                 prove nothing: {before}"
            );

            let refused = server
                .delete(&format!(
                    "/realms/{}/federation/providers/{}",
                    TENANT_A,
                    ctx().delete_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;

            assert_not_found(
                &refused,
                "deleting a tenant-b federation provider from the tenant-a url",
            );

            let survived =
                federation_provider_text(&server, TENANT_B, &ctx().delete_provider_id).await;
            assert!(
                survived.contains(DELETE_DIRECTORY_NAME),
                "the tenant-b federation provider was deleted through the tenant-a url: {survived}"
            );

            let accepted = server
                .delete(&format!(
                    "/realms/{}/federation/providers/{}",
                    TENANT_B,
                    ctx().delete_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;

            assert!(
                accepted.status_code().is_success(),
                "a tenant-b federation provider must be deletable from the tenant-b url: {} {}",
                accepted.status_code(),
                accepted.text()
            );

            let gone = get_federation_provider(&server, TENANT_B, &ctx().delete_provider_id).await;
            assert_not_found(
                &gone,
                "the accepted deletion did not land, so the refusal above is not evidence of \
                 scoping",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn operating_a_federation_provider_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let own_test = server
                .post(&format!(
                    "/realms/{}/federation/providers/{}/test-connection",
                    TENANT_B,
                    ctx().read_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_ne!(
                own_test.status_code(),
                404,
                "the test-connection route does not reach a provider of its own realm, so the \
                 refusal below would prove nothing: {}",
                own_test.text()
            );

            let own_sync = server
                .post(&format!(
                    "/realms/{}/federation/providers/{}/sync-users",
                    TENANT_B,
                    ctx().read_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_ne!(
                own_sync.status_code(),
                404,
                "the sync-users route does not reach a provider of its own realm, so the refusal \
                 below would prove nothing: {}",
                own_sync.text()
            );

            let foreign_test = server
                .post(&format!(
                    "/realms/{}/federation/providers/{}/test-connection",
                    TENANT_A,
                    ctx().read_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_not_found(
                &foreign_test,
                "testing the connection of a tenant-b federation provider from the tenant-a url",
            );

            let foreign_sync = server
                .post(&format!(
                    "/realms/{}/federation/providers/{}/sync-users",
                    TENANT_A,
                    ctx().read_provider_id
                ))
                .add_header("Authorization", auth_header(&ctx().admin_token))
                .await;
            assert_not_found(
                &foreign_sync,
                "syncing a tenant-b federation provider from the tenant-a url",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn a_broker_state_issued_in_one_realm_is_not_consumable_from_another() {
        rt().block_on(async {
            let server = make_server();

            let state = issue_broker_state(&server).await;

            let foreign = broker_callback(
                &server,
                TENANT_A,
                &state,
                &[("error", "access_denied"), ("error_description", "nope")],
            )
            .await;
            let foreign_body = foreign.text();

            assert_eq!(
                foreign.status_code(),
                400,
                "consuming a tenant-b broker state from the tenant-a callback: expected 400, got \
                 {} with body {foreign_body}",
                foreign.status_code()
            );
            assert!(
                foreign_body.contains(SESSION_ABSENT_MESSAGE),
                "the tenant-a callback answered something other than an absent session, so the \
                 state may have been consumed: {foreign_body}"
            );
            assert_body_free_of(
                &foreign_body,
                &[TENANT_B_REDIRECT_URI],
                "the tenant-a callback for a tenant-b broker state",
            );

            let own = broker_callback(
                &server,
                TENANT_B,
                &state,
                &[("error", "access_denied"), ("error_description", "nope")],
            )
            .await;

            assert_eq!(
                own.status_code(),
                401,
                "the tenant-b callback did not find the state it issued, so the refusal above is \
                 not evidence of scoping and may just be a consumed session: {} {}",
                own.status_code(),
                own.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn a_foreign_callback_does_not_destroy_the_broker_session_it_cannot_read() {
        rt().block_on(async {
            let server = make_server();

            let state = issue_broker_state(&server).await;

            let foreign = broker_callback(
                &server,
                TENANT_A,
                &state,
                &[("code", "an-authorization-code-tenant-a-never-received")],
            )
            .await;
            let foreign_body = foreign.text();

            assert_eq!(
                foreign.status_code(),
                400,
                "redeeming a tenant-b broker state at the tenant-a callback: expected 400, got {} \
                 with body {foreign_body}",
                foreign.status_code()
            );
            assert!(
                foreign_body.contains(SESSION_ABSENT_MESSAGE),
                "the tenant-a callback got past the session lookup: {foreign_body}"
            );

            let own = broker_callback(
                &server,
                TENANT_B,
                &state,
                &[("error", "access_denied"), ("error_description", "nope")],
            )
            .await;

            assert_eq!(
                own.status_code(),
                401,
                "the tenant-b session did not survive the foreign callback, so a neighbouring \
                 realm can still cancel a login in flight: {} {}",
                own.status_code(),
                own.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test abyss_cross_realm_test -- --ignored"]
    fn a_broker_login_names_an_identity_provider_of_the_url_realm_only() {
        rt().block_on(async {
            let server = make_server();

            let own = broker_login_location(
                &server,
                TENANT_A,
                BROKER_IDP_ALIAS,
                TENANT_A_CLIENT_ID,
                TENANT_A_REDIRECT_URI,
            )
            .await;
            assert_eq!(
                own.status_code(),
                302,
                "tenant-a cannot start a broker login with its own provider, so the refusal \
                 below would prove nothing: {}",
                own.text()
            );

            let foreign = broker_login_location(
                &server,
                TENANT_A,
                READ_IDP_ALIAS,
                TENANT_A_CLIENT_ID,
                TENANT_A_REDIRECT_URI,
            )
            .await;

            assert_not_found(
                &foreign,
                "starting a broker login in tenant-a with a tenant-b identity provider alias",
            );
        });
    }
}
