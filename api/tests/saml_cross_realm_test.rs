#[cfg(test)]
mod tests {
    use std::{env, process::Command, sync::Arc};

    use axum::{Router, http::HeaderValue};
    use axum_test::{TestResponse, TestServer};
    use chrono::{SecondsFormat, Utc};
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::{Args, ServerArgs},
    };
    use ferriskey_core::{
        application::create_service,
        domain::common::{
            DatabaseConfig, FerriskeyConfig, entities::StartupConfig, ports::CoreService,
        },
    };
    use ferriskey_saml::binding::encode_redirect;
    use serde_json::{Value, json};
    use sqlx::{Executor, PgPool};
    use uuid::Uuid;

    const PUBLIC_BASE_URL: &str = "https://auth.ferriskey.test";
    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const ADMIN_CLI: &str = "admin-cli";

    const MASTER_REALM: &str = "master";
    const MASTER_PASSWORD: &str = "admin";

    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";

    const ALICE: &str = "alice";
    const ALICE_PASSWORD: &str = "Al1ce-Tenant-A-Sso!";

    const SP_A_ENTITY_ID: &str = "https://sp-a.example.com/saml/metadata";
    const SP_A_ACS: &str = "https://sp-a.example.com/saml/acs";

    const SP_B_ENTITY_ID: &str = "https://sp-b.example.com/saml/metadata";
    const SP_B_ACS: &str = "https://sp-b.example.com/saml/acs";

    const PLANTED_ENTITY_ID: &str = "https://sp-a-planted.example.com/saml/metadata";
    const PLANTED_ACS: &str = "https://sp-a-planted.example.com/saml/acs";

    const OWNED_MAPPER: &str = "email";
    const VICTIM_MAPPER: &str = "tenant-b-only";
    const FORGED_MAPPER: &str = "forged-from-tenant-a";
    const DISPOSABLE_MAPPER: &str = "tenant-b-disposable";

    const ASSERTION_ID_ATTRIBUTE: &str = "--id-attr:ID";
    const ASSERTION_QUALIFIED_NAME: &str = "urn:oasis:names:tc:SAML:2.0:assertion:Assertion";

    const SAML_RESPONSE_FIELD: &str = r#"name="SAMLResponse" value=""#;

    struct ServiceProvider {
        uuid: Uuid,
        client_id: String,
        entity_id: String,
        acs_url: String,
    }

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        admin_token: String,
        pool: PgPool,
        sp_a: ServiceProvider,
        sp_b: ServiceProvider,
        sp_planted: ServiceProvider,
        victim_mapper: Uuid,
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
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    fn ctx() -> &'static SharedContext {
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

        let schema = format!("saml_cross_realm_test_{}", Uuid::new_v4().simple());
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
                admin_email: "admin@ferriskey.test".to_string(),
                admin_password: MASTER_PASSWORD.to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let args = Arc::new(Args {
            server: ServerArgs {
                public_url: Some(PUBLIC_BASE_URL.to_string()),
                ..ServerArgs::default()
            },
            ..Args::default()
        });
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        let server = TestServer::new(app.clone()).expect("create test server");
        let admin_token = password_grant(&server, MASTER_REALM, "admin", MASTER_PASSWORD).await;

        create_realm(&server, &admin_token, TENANT_A).await;
        create_realm(&server, &admin_token, TENANT_B).await;

        let alice = create_user(&server, &admin_token, TENANT_A, ALICE).await;
        set_password(&server, &admin_token, TENANT_A, alice, ALICE_PASSWORD).await;

        let sp_a = create_service_provider(
            &server,
            &admin_token,
            TENANT_A,
            SP_A_ENTITY_ID,
            SP_A_ACS,
            "sp-a",
        )
        .await;
        let sp_b = create_service_provider(
            &server,
            &admin_token,
            TENANT_B,
            SP_B_ENTITY_ID,
            SP_B_ACS,
            "sp-b",
        )
        .await;
        let sp_planted = create_service_provider(
            &server,
            &admin_token,
            TENANT_A,
            PLANTED_ENTITY_ID,
            PLANTED_ACS,
            "sp-a-planted",
        )
        .await;

        create_mapper_ok(&server, &admin_token, TENANT_A, sp_a.uuid, OWNED_MAPPER).await;
        let victim_mapper =
            create_mapper_ok(&server, &admin_token, TENANT_B, sp_b.uuid, VICTIM_MAPPER).await;

        SharedContext {
            app: std::sync::Mutex::new(app),
            admin_token,
            pool,
            sp_a,
            sp_b,
            sp_planted,
            victim_mapper,
        }
    }

    fn make_server() -> TestServer {
        let app = ctx().app.lock().expect("router mutex poisoned").clone();
        TestServer::new(app).expect("create test server")
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

        response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
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

        Uuid::parse_str(raw).unwrap_or_else(|error| panic!("user id {raw} is not a uuid: {error}"))
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

    async fn create_service_provider(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        entity_id: &str,
        acs_url: &str,
        label: &str,
    ) -> ServiceProvider {
        let client_id = format!("{label}-{}", Uuid::new_v4().simple());

        let created = server
            .post(&format!("/realms/{}/clients", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": client_id,
                "client_id": client_id,
                "client_type": "public",
                "protocol": "saml",
                "enabled": true,
            }))
            .await;

        assert_eq!(
            created.status_code(),
            201,
            "creating the saml client {client_id} in {realm} failed: {}",
            created.text()
        );

        let raw = created.json::<Value>()["id"]
            .as_str()
            .expect("the created client carries an id")
            .to_string();
        let uuid = Uuid::parse_str(&raw)
            .unwrap_or_else(|error| panic!("client id {raw} is not a uuid: {error}"));

        let configured = server
            .put(&format!("/realms/{}/clients/{}/saml-config", realm, uuid))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "sp_entity_id": entity_id,
                "acs_url": acs_url,
            }))
            .await;

        assert_eq!(
            configured.status_code(),
            201,
            "configuring the saml client {client_id} in {realm} failed: {}",
            configured.text()
        );

        ServiceProvider {
            uuid,
            client_id,
            entity_id: entity_id.to_string(),
            acs_url: acs_url.to_string(),
        }
    }

    async fn create_mapper(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_uuid: Uuid,
        name: &str,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/clients/{}/saml-attribute-mappers",
                realm, client_uuid
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "source": "user:email",
            }))
            .await
    }

    async fn create_mapper_ok(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_uuid: Uuid,
        name: &str,
    ) -> Uuid {
        let response = create_mapper(server, admin_token, realm, client_uuid, name).await;

        assert_eq!(
            response.status_code(),
            201,
            "creating the mapper {name} on {client_uuid} in {realm} failed: {}",
            response.text()
        );

        let raw = response.json::<Value>()["id"]
            .as_str()
            .expect("the created mapper carries an id")
            .to_string();

        Uuid::parse_str(&raw)
            .unwrap_or_else(|error| panic!("mapper id {raw} is not a uuid: {error}"))
    }

    async fn list_mappers(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_uuid: Uuid,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{}/clients/{}/saml-attribute-mappers",
                realm, client_uuid
            ))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    async fn delete_mapper(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        client_uuid: Uuid,
        mapper_id: Uuid,
    ) -> TestResponse {
        server
            .delete(&format!(
                "/realms/{}/clients/{}/saml-attribute-mappers/{}",
                realm, client_uuid, mapper_id
            ))
            .add_header("Authorization", auth_header(admin_token))
            .await
    }

    fn authn_request(realm: &str, issuer: &str, acs: Option<&str>) -> String {
        let id = format!("_{}", Uuid::new_v4().simple());
        let issue_instant = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let acs_attribute = acs
            .map(|value| format!(r#" AssertionConsumerServiceURL="{value}""#))
            .unwrap_or_default();

        format!(
            r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="{id}" Version="2.0" IssueInstant="{issue_instant}" Destination="{PUBLIC_BASE_URL}/realms/{realm}/protocol/saml"{acs_attribute}><saml:Issuer>{issuer}</saml:Issuer></samlp:AuthnRequest>"#
        )
    }

    async fn start_sso(
        server: &TestServer,
        realm: &str,
        issuer: &str,
        acs: Option<&str>,
    ) -> TestResponse {
        let document = authn_request(realm, issuer, acs);
        let encoded = encode_redirect(&document).expect("deflate the authn request");

        server
            .get(&format!("/realms/{}/protocol/saml", realm))
            .add_query_param("SAMLRequest", encoded)
            .await
    }

    async fn mint_authorization_code(
        server: &TestServer,
        realm: &str,
        provider: &ServiceProvider,
        username: &str,
        password: &str,
    ) -> String {
        let started = start_sso(server, realm, &provider.entity_id, Some(&provider.acs_url)).await;
        assert_eq!(
            started.status_code(),
            302,
            "starting an sso in {realm} for {} failed: {}",
            provider.entity_id,
            started.text()
        );

        let login = server
            .post(&format!("/realms/{}/login-actions/authenticate", realm))
            .add_cookie(started.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", provider.client_id.clone())
            .json(&json!({ "username": username, "password": password }))
            .await;

        assert_eq!(
            login.status_code(),
            200,
            "the interactive login of {username}@{realm} failed: {}",
            login.text()
        );

        let continuation = login.json::<Value>()["url"]
            .as_str()
            .expect("a completed saml login carries a continuation url")
            .to_string();

        continuation
            .strip_prefix(PUBLIC_BASE_URL)
            .expect("the continuation url is rooted at the configured public base url")
            .split("code=")
            .nth(1)
            .expect("the continuation url carries the authorization code")
            .to_string()
    }

    async fn deliver(server: &TestServer, realm: &str, code: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/protocol/saml/continue", realm))
            .add_query_param("code", code)
            .await
    }

    async fn descriptor(server: &TestServer, realm: &str) -> String {
        let response = server
            .get(&format!("/realms/{}/protocol/saml/descriptor", realm))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "fetching the descriptor of {realm} failed: {}",
            response.text()
        );

        response.text()
    }

    fn certificate_pem(descriptor_xml: &str) -> String {
        const OPEN: &str = "<ds:X509Certificate>";
        const CLOSE: &str = "</ds:X509Certificate>";

        let start = descriptor_xml
            .find(OPEN)
            .expect("the descriptor carries a certificate")
            + OPEN.len();
        let end = descriptor_xml[start..]
            .find(CLOSE)
            .expect("the certificate element is closed");
        let base64_der = &descriptor_xml[start..start + end];

        let wrapped: Vec<&str> = base64_der
            .as_bytes()
            .chunks(64)
            .map(|chunk| std::str::from_utf8(chunk).expect("base64 is ascii"))
            .collect();

        format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            wrapped.join("\n")
        )
    }

    fn certificate_body(descriptor_xml: &str) -> String {
        const OPEN: &str = "<ds:X509Certificate>";
        const CLOSE: &str = "</ds:X509Certificate>";

        let start = descriptor_xml
            .find(OPEN)
            .expect("the descriptor carries a certificate")
            + OPEN.len();
        let end = descriptor_xml[start..]
            .find(CLOSE)
            .expect("the certificate element is closed");

        descriptor_xml[start..start + end].to_string()
    }

    fn assertion_of(form_page: &str) -> String {
        let start = form_page
            .find(SAML_RESPONSE_FIELD)
            .expect("the delivered form carries a SAMLResponse field")
            + SAML_RESPONSE_FIELD.len();
        let end = form_page[start..]
            .find('"')
            .expect("the SAMLResponse value is terminated");

        ferriskey_saml::binding::decode_post(&form_page[start..start + end])
            .expect("the posted SAMLResponse is valid base64")
    }

    fn verify_with_xmlsec(document: &str, certificate: &str) -> bool {
        let document_path = std::env::temp_dir().join(format!(
            "ferriskey-saml-cross-realm-{}.xml",
            Uuid::new_v4().simple()
        ));
        let certificate_path = std::env::temp_dir().join(format!(
            "ferriskey-saml-cross-realm-{}.pem",
            Uuid::new_v4().simple()
        ));

        std::fs::write(&document_path, document).expect("write the document under test");
        std::fs::write(&certificate_path, certificate).expect("write the trusted certificate");

        Command::new("xmlsec1")
            .arg("--verify")
            .arg("--trusted-pem")
            .arg(&certificate_path)
            .arg(ASSERTION_ID_ATTRIBUTE)
            .arg(ASSERTION_QUALIFIED_NAME)
            .arg(&document_path)
            .output()
            .expect("run xmlsec1 — is it installed?")
            .status
            .success()
    }

    async fn realm_id(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query_scalar("SELECT id FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| panic!("read the id of realm {name}: {error}"))
    }

    async fn plant_foreign_realm_on_config(
        pool: &PgPool,
        client_uuid: Uuid,
        foreign_realm: Uuid,
        foreign_acs: &str,
    ) {
        sqlx::query(
            "UPDATE client_saml_configs SET realm_id = $2, acs_url = $3 WHERE client_id = $1",
        )
        .bind(client_uuid)
        .bind(foreign_realm)
        .bind(foreign_acs)
        .execute(pool)
        .await
        .unwrap_or_else(|error| {
            panic!("plant a foreign realm on the config of {client_uuid}: {error}")
        });
    }

    async fn stored_realm_of_config(pool: &PgPool, client_uuid: Uuid) -> Uuid {
        sqlx::query_scalar("SELECT realm_id FROM client_saml_configs WHERE client_id = $1")
            .bind(client_uuid)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|error| {
                panic!("read the realm of the config of {client_uuid}: {error}")
            })
    }

    fn assert_not_found(response: &TestResponse, what: &str) {
        assert_eq!(
            response.status_code(),
            404,
            "{what}: expected a 404, got {} with body {}",
            response.status_code(),
            response.text()
        );
    }

    fn assert_no_assertion(response: &TestResponse, what: &str) {
        assert!(
            !response.status_code().is_success(),
            "{what}: expected a refusal, got {} with body {}",
            response.status_code(),
            response.text()
        );

        assert!(
            !response.text().contains(SAML_RESPONSE_FIELD),
            "{what}: the refusal still delivered a signed response: {}",
            response.text()
        );
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn an_authn_request_for_a_service_provider_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner = start_sso(&server, TENANT_B, &ctx.sp_b.entity_id, Some(SP_B_ACS)).await;
            assert_eq!(
                owner.status_code(),
                302,
                "the tenant-b service provider is not servable from its own realm, so the \
                 refusals below would prove nothing: {}",
                owner.text()
            );

            let borrowed = start_sso(&server, TENANT_A, &ctx.sp_b.entity_id, Some(SP_B_ACS)).await;
            assert_eq!(
                borrowed.status_code(),
                400,
                "tenant a must not start a login for a service provider registered in tenant b: {}",
                borrowed.text()
            );

            let mirrored = start_sso(&server, TENANT_B, &ctx.sp_a.entity_id, Some(SP_A_ACS)).await;
            assert_eq!(
                mirrored.status_code(),
                400,
                "tenant b must not start a login for a service provider registered in tenant a: {}",
                mirrored.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn an_authorization_code_cannot_be_redeemed_at_another_realms_continue_endpoint() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let code =
                mint_authorization_code(&server, TENANT_A, &ctx.sp_a, ALICE, ALICE_PASSWORD).await;

            let borrowed = deliver(&server, TENANT_B, &code).await;
            assert_no_assertion(
                &borrowed,
                "redeeming a tenant-a authorization code at the tenant-b continue endpoint",
            );

            let owner = deliver(&server, TENANT_A, &code).await;
            assert_eq!(
                owner.status_code(),
                200,
                "the refused redemption must not have spent the code of its own realm: {}",
                owner.text()
            );
            assert!(
                owner.text().contains(SAML_RESPONSE_FIELD),
                "the tenant-a redemption delivered no signed response: {}",
                owner.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn an_assertion_is_signed_by_its_own_realm_and_by_no_other() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let tenant_a_descriptor = descriptor(&server, TENANT_A).await;
            let tenant_b_descriptor = descriptor(&server, TENANT_B).await;

            assert_ne!(
                certificate_body(&tenant_a_descriptor),
                certificate_body(&tenant_b_descriptor),
                "two realms publishing the same certificate would make the comparison below empty"
            );

            let code =
                mint_authorization_code(&server, TENANT_A, &ctx.sp_a, ALICE, ALICE_PASSWORD).await;
            let delivered = deliver(&server, TENANT_A, &code).await;
            assert_eq!(
                delivered.status_code(),
                200,
                "delivering the tenant-a assertion failed: {}",
                delivered.text()
            );

            let assertion = assertion_of(&delivered.text());

            assert!(
                verify_with_xmlsec(&assertion, &certificate_pem(&tenant_a_descriptor)),
                "xmlsec1 rejected a tenant-a assertion against the tenant-a certificate: {assertion}"
            );
            assert!(
                !verify_with_xmlsec(&assertion, &certificate_pem(&tenant_b_descriptor)),
                "xmlsec1 accepted a tenant-a assertion against the tenant-b certificate, so a \
                 service provider trusting tenant b would take it: {assertion}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn a_service_provider_configuration_that_claims_another_realm_yields_no_assertion() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let honest =
                mint_authorization_code(&server, TENANT_A, &ctx.sp_planted, ALICE, ALICE_PASSWORD)
                    .await;
            let delivered = deliver(&server, TENANT_A, &honest).await;
            assert_eq!(
                delivered.status_code(),
                200,
                "the planted service provider is not servable before the divergence, so the \
                 refusal below would prove nothing: {}",
                delivered.text()
            );
            assert!(
                delivered.text().contains(PLANTED_ACS),
                "the honest delivery did not target its own assertion consumer service: {}",
                delivered.text()
            );

            let smuggled =
                mint_authorization_code(&server, TENANT_A, &ctx.sp_planted, ALICE, ALICE_PASSWORD)
                    .await;

            let tenant_b = realm_id(&ctx.pool, TENANT_B).await;
            plant_foreign_realm_on_config(&ctx.pool, ctx.sp_planted.uuid, tenant_b, SP_B_ACS).await;
            assert_eq!(
                stored_realm_of_config(&ctx.pool, ctx.sp_planted.uuid).await,
                tenant_b,
                "the divergence was not planted, so the refusal below would prove nothing"
            );

            let response = deliver(&server, TENANT_A, &smuggled).await;
            assert_no_assertion(
                &response,
                "delivering an assertion for a configuration that claims another realm",
            );
            assert!(
                !response.text().contains(SP_B_ACS),
                "the refusal still named the assertion consumer service of the other realm: {}",
                response.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn an_assertion_carries_only_the_attribute_mappers_of_its_own_client() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let code =
                mint_authorization_code(&server, TENANT_A, &ctx.sp_a, ALICE, ALICE_PASSWORD).await;
            let delivered = deliver(&server, TENANT_A, &code).await;
            assert_eq!(
                delivered.status_code(),
                200,
                "delivering the tenant-a assertion failed: {}",
                delivered.text()
            );

            let assertion = assertion_of(&delivered.text());

            assert!(
                assertion.contains(&format!(r#"Name="{OWNED_MAPPER}""#)),
                "the assertion carries no attribute at all, so the absence below would prove \
                 nothing: {assertion}"
            );
            assert!(
                !assertion.contains(VICTIM_MAPPER),
                "the assertion carries an attribute mapper registered in the other realm: \
                 {assertion}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn creating_a_saml_attribute_mapper_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owned = create_mapper(
                &server,
                &ctx.admin_token,
                TENANT_B,
                ctx.sp_b.uuid,
                DISPOSABLE_MAPPER,
            )
            .await;
            assert_eq!(
                owned.status_code(),
                201,
                "the tenant-b client does not accept a mapper from its own realm, so the refusal \
                 below would prove nothing: {}",
                owned.text()
            );

            let forged = create_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.sp_b.uuid,
                FORGED_MAPPER,
            )
            .await;
            assert_not_found(
                &forged,
                "mapping an attribute onto a tenant-b client through the tenant-a url",
            );

            let listed = list_mappers(&server, &ctx.admin_token, TENANT_B, ctx.sp_b.uuid).await;
            assert_eq!(
                listed.status_code(),
                200,
                "listing the tenant-b mappers failed: {}",
                listed.text()
            );
            assert!(
                !listed.text().contains(FORGED_MAPPER),
                "the refused creation still reached the tenant-b client: {}",
                listed.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn listing_another_realms_saml_attribute_mappers_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let owner = list_mappers(&server, &ctx.admin_token, TENANT_B, ctx.sp_b.uuid).await;
            assert_eq!(
                owner.status_code(),
                200,
                "listing the tenant-b mappers from its own realm failed: {}",
                owner.text()
            );
            assert!(
                owner.text().contains(VICTIM_MAPPER),
                "the tenant-b client carries no mapper, so the refusal below would prove nothing: \
                 {}",
                owner.text()
            );

            let borrowed = list_mappers(&server, &ctx.admin_token, TENANT_A, ctx.sp_b.uuid).await;
            assert_not_found(
                &borrowed,
                "listing the mappers of a tenant-b client through the tenant-a url",
            );
            assert!(
                !borrowed.text().contains(VICTIM_MAPPER),
                "the refusal leaked the attribute the other realm sends: {}",
                borrowed.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_cross_realm_test -- --ignored"]
    fn deleting_another_realms_saml_attribute_mapper_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let ctx = ctx();

            let doomed = create_mapper_ok(
                &server,
                &ctx.admin_token,
                TENANT_B,
                ctx.sp_b.uuid,
                "tenant-b-doomed",
            )
            .await;
            let owned =
                delete_mapper(&server, &ctx.admin_token, TENANT_B, ctx.sp_b.uuid, doomed).await;
            assert_eq!(
                owned.status_code(),
                200,
                "the tenant-b realm cannot delete its own mapper, so the refusal below would \
                 prove nothing: {}",
                owned.text()
            );

            let borrowed = delete_mapper(
                &server,
                &ctx.admin_token,
                TENANT_A,
                ctx.sp_b.uuid,
                ctx.victim_mapper,
            )
            .await;
            assert_not_found(
                &borrowed,
                "deleting a tenant-b mapper through the tenant-a url",
            );

            let listed = list_mappers(&server, &ctx.admin_token, TENANT_B, ctx.sp_b.uuid).await;
            assert_eq!(
                listed.status_code(),
                200,
                "listing the tenant-b mappers failed: {}",
                listed.text()
            );
            assert!(
                listed.text().contains(VICTIM_MAPPER),
                "the refused deletion still removed the mapper: {}",
                listed.text()
            );
        });
    }
}
