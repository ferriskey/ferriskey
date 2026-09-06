/// Integration tests for the SAML 2.0 Identity Provider (#1269).
///
/// Proves the SAML IdP end to end through the real HTTP API: metadata
/// publication, SP-initiated SSO over both bindings — with the signed
/// response verified by `xmlsec1`, an independent verifier, never by our own
/// code — and the rejection cases (unknown service provider, mismatched ACS,
/// replayed authorization code).
///
/// Require a running PostgreSQL instance and the `xmlsec1` binary. Marked
/// `#[ignore]` so they don't block regular `cargo test` runs. Run them
/// explicitly with:
///
///   cargo test -p ferriskey-api --test saml_test -- --ignored
///
/// Environment variables (defaults shown):
///   DATABASE_HOST     = localhost
///   DATABASE_PORT     = 5432
///   DATABASE_NAME     = ferriskey
///   DATABASE_USER     = ferriskey
///   DATABASE_PASSWORD = ferriskey
#[cfg(test)]
mod tests {
    use std::{env, process::Command, sync::Arc};

    use axum::Router;
    use axum::http::HeaderValue;
    use axum_test::TestServer;
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
    use ferriskey_saml::binding::{encode_post, encode_redirect};
    use serde_json::Value;
    use sqlx::Executor;
    use uuid::Uuid;

    const PUBLIC_BASE_URL: &str = "https://auth.ferriskey.test";
    const ADMIN_PASSWORD: &str = "admin_pass_1234!";
    const ASSERTION_ID_ATTRIBUTE: &str = "--id-attr:ID";
    const ASSERTION_QUALIFIED_NAME: &str = "urn:oasis:names:tc:SAML:2.0:assertion:Assertion";

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
        realm_name: String,
        #[allow(dead_code)]
        pool: sqlx::PgPool,
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
        CTX.get_or_init(|| rt().block_on(async { setup().await }))
    }

    async fn setup() -> SharedContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port = env_u16_or("DATABASE_PORT", 5432);
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");

        let schema = format!("test_saml_{}", Uuid::new_v4().simple());
        let admin_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            db_user, db_password, db_host, db_port, db_name
        );

        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema).as_str())
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

        let svc = create_service(FerriskeyConfig {
            webapp_url: "http://localhost:5555".to_string(),
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

        let realm_name = format!("test-realm-{}", Uuid::new_v4().simple());
        svc.initialize_application(StartupConfig {
            webapp_url: "http://localhost:5555".to_string(),
            master_realm_name: realm_name.clone(),
            admin_username: "admin".to_string(),
            admin_email: "admin@ferriskey.test".to_string(),
            admin_password: ADMIN_PASSWORD.to_string(),
            default_client_id: "ferriskey-admin".to_string(),
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
        let state = AppState::new(args, svc);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            realm_name,
            pool,
        }
    }

    fn server() -> TestServer {
        let app = ctx().app.lock().expect("lock app mutex").clone();
        TestServer::new(app).expect("build test server")
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token)
            .parse()
            .expect("valid header value")
    }

    async fn admin_token(srv: &TestServer, realm: &str) -> String {
        let resp = srv
            .post(&format!("/realms/{}/protocol/openid-connect/token", realm))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", ADMIN_PASSWORD),
                ("scope", "openid profile"),
            ])
            .await;

        assert_eq!(
            resp.status_code(),
            200,
            "password grant failed: {}",
            resp.text()
        );

        resp.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string()
    }

    /// Registers a SAML service provider client and returns its `client_id`
    /// string (used by the interactive login endpoint) alongside its
    /// `sp_entity_id`/`acs_url` (used to build the AuthnRequest under test).
    async fn create_saml_service_provider(
        srv: &TestServer,
        realm: &str,
        admin_token: &str,
        acs_url: &str,
    ) -> (String, String) {
        let client_id = format!("saml-sp-{}", Uuid::new_v4().simple());
        let sp_entity_id = format!("https://{client_id}.example.com/saml/metadata");

        let create_resp = srv
            .post(&format!("/realms/{}/clients", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&serde_json::json!({
                "name": client_id,
                "client_id": client_id,
                "client_type": "public",
                "protocol": "saml",
                "enabled": true,
            }))
            .await;
        assert_eq!(
            create_resp.status_code(),
            201,
            "saml client creation failed: {}",
            create_resp.text()
        );
        let client_uuid = create_resp.json::<Value>()["id"]
            .as_str()
            .expect("client id")
            .to_string();

        let config_resp = srv
            .put(&format!(
                "/realms/{}/clients/{}/saml-config",
                realm, client_uuid
            ))
            .add_header("Authorization", auth_header(admin_token))
            .json(&serde_json::json!({
                "sp_entity_id": sp_entity_id,
                "acs_url": acs_url,
            }))
            .await;
        assert_eq!(
            config_resp.status_code(),
            201,
            "saml sp config failed: {}",
            config_resp.text()
        );

        (client_id, sp_entity_id)
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

    /// Starts SP-initiated SSO over the given binding and returns the
    /// `FERRISKEY_SESSION` cookie from the 302 redirect to the login page.
    async fn start_sso(
        srv: &TestServer,
        realm: &str,
        authn_request_xml: &str,
        via_redirect_binding: bool,
    ) -> axum_test::TestResponse {
        if via_redirect_binding {
            let encoded = encode_redirect(authn_request_xml).expect("deflate the authn request");
            srv.get(&format!("/realms/{}/protocol/saml", realm))
                .add_query_param("SAMLRequest", encoded)
                .await
        } else {
            let encoded = encode_post(authn_request_xml);
            srv.post(&format!("/realms/{}/protocol/saml", realm))
                .form(&[("SAMLRequest", encoded)])
                .await
        }
    }

    async fn complete_login(
        srv: &TestServer,
        realm: &str,
        started: &axum_test::TestResponse,
        client_id: &str,
    ) -> String {
        let login = srv
            .post(&format!("/realms/{}/login-actions/authenticate", realm))
            .add_cookie(started.cookie("FERRISKEY_SESSION"))
            .add_query_param("client_id", client_id)
            .json(&serde_json::json!({ "username": "admin", "password": ADMIN_PASSWORD }))
            .await;

        assert_eq!(
            login.status_code(),
            200,
            "interactive login failed: {}",
            login.text()
        );

        login.json::<Value>()["url"]
            .as_str()
            .expect("a completed saml login must carry a continuation url")
            .to_string()
    }

    fn continue_path(continuation_url: &str) -> String {
        continuation_url
            .strip_prefix(PUBLIC_BASE_URL)
            .expect("the continuation url must be rooted at the configured public base url")
            .to_string()
    }

    /// Extracts the `SAMLResponse` hidden field the auto-submitting HTML form
    /// carries, and base64-decodes it into the raw `<samlp:Response>` XML.
    fn saml_response_xml_from_form(html: &str) -> String {
        const MARKER: &str = r#"name="SAMLResponse" value=""#;
        let start = html
            .find(MARKER)
            .expect("the form must carry a SAMLResponse field")
            + MARKER.len();
        let end = html[start..]
            .find('"')
            .expect("the SAMLResponse value must be terminated");
        let encoded = &html[start..start + end];

        ferriskey_saml::binding::decode_post(encoded)
            .expect("the posted SAMLResponse must be valid base64")
    }

    fn descriptor_certificate_pem(descriptor_xml: &str) -> String {
        const OPEN: &str = "<ds:X509Certificate>";
        const CLOSE: &str = "</ds:X509Certificate>";
        let start = descriptor_xml
            .find(OPEN)
            .expect("descriptor carries a certificate")
            + OPEN.len();
        let end = descriptor_xml[start..]
            .find(CLOSE)
            .expect("the certificate element must be closed");
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

    fn verify_with_xmlsec(
        document: &str,
        trusted_cert_path: &std::path::Path,
    ) -> std::process::Output {
        let path = std::env::temp_dir().join(format!(
            "ferriskey-saml-api-test-{}.xml",
            Uuid::new_v4().simple()
        ));
        std::fs::write(&path, document).expect("write the document under test");

        Command::new("xmlsec1")
            .arg("--verify")
            .arg("--trusted-pem")
            .arg(trusted_cert_path)
            .arg(ASSERTION_ID_ATTRIBUTE)
            .arg(ASSERTION_QUALIFIED_NAME)
            .arg(&path)
            .output()
            .expect("run xmlsec1 — is it installed?")
    }

    async fn descriptor_xml(srv: &TestServer, realm: &str) -> String {
        let resp = srv
            .get(&format!("/realms/{}/protocol/saml/descriptor", realm))
            .await;
        assert_eq!(
            resp.status_code(),
            200,
            "fetching the descriptor failed: {}",
            resp.text()
        );
        resp.text()
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn descriptor_publishes_the_realm_metadata() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let document = descriptor_xml(&srv, &realm).await;

            assert!(
                document.contains(&format!(r#"entityID="{PUBLIC_BASE_URL}/realms/{realm}""#)),
                "the descriptor does not advertise the expected entity id: {document}"
            );
            assert!(
                document
                    .contains(r#"Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect""#),
                "the descriptor does not advertise the HTTP-Redirect binding: {document}"
            );
            assert!(
                document.contains(r#"Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST""#),
                "the descriptor does not advertise the HTTP-POST binding: {document}"
            );
            assert!(
                document.contains("<ds:X509Certificate>"),
                "the descriptor does not carry a signing certificate: {document}"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn sso_over_http_redirect_binding_produces_a_verifiable_assertion() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let acs_url = "https://sp.example.com/saml/acs/redirect";
            let (client_id, sp_entity_id) =
                create_saml_service_provider(&srv, &realm, &admin_token, acs_url).await;

            let request = authn_request(&realm, &sp_entity_id, Some(acs_url));
            let started = start_sso(&srv, &realm, &request, true).await;
            assert_eq!(
                started.status_code(),
                302,
                "starting sso over the redirect binding failed: {}",
                started.text()
            );

            let continuation_url = complete_login(&srv, &realm, &started, &client_id).await;

            let form_page = srv.get(&continue_path(&continuation_url)).await;
            assert_eq!(
                form_page.status_code(),
                200,
                "delivering the assertion failed: {}",
                form_page.text()
            );

            let response_xml = saml_response_xml_from_form(&form_page.text());
            let descriptor = descriptor_xml(&srv, &realm).await;
            let cert_path = std::env::temp_dir().join(format!("ferriskey-saml-cert-{}.pem", Uuid::new_v4().simple()));
            std::fs::write(&cert_path, descriptor_certificate_pem(&descriptor))
                .expect("write the trusted certificate");

            let outcome = verify_with_xmlsec(&response_xml, &cert_path);
            assert!(
                outcome.status.success(),
                "xmlsec1 rejected the assertion issued over the redirect binding\nstdout: {}\nstderr: {}\ndocument: {}",
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr),
                response_xml
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn sso_over_http_post_binding_produces_a_verifiable_assertion() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let acs_url = "https://sp.example.com/saml/acs/post";
            let (client_id, sp_entity_id) =
                create_saml_service_provider(&srv, &realm, &admin_token, acs_url).await;

            let request = authn_request(&realm, &sp_entity_id, Some(acs_url));
            let started = start_sso(&srv, &realm, &request, false).await;
            assert_eq!(
                started.status_code(),
                302,
                "starting sso over the post binding failed: {}",
                started.text()
            );

            let continuation_url = complete_login(&srv, &realm, &started, &client_id).await;

            let form_page = srv.get(&continue_path(&continuation_url)).await;
            assert_eq!(
                form_page.status_code(),
                200,
                "delivering the assertion failed: {}",
                form_page.text()
            );

            let response_xml = saml_response_xml_from_form(&form_page.text());
            let descriptor = descriptor_xml(&srv, &realm).await;
            let cert_path = std::env::temp_dir().join(format!("ferriskey-saml-cert-{}.pem", Uuid::new_v4().simple()));
            std::fs::write(&cert_path, descriptor_certificate_pem(&descriptor))
                .expect("write the trusted certificate");

            let outcome = verify_with_xmlsec(&response_xml, &cert_path);
            assert!(
                outcome.status.success(),
                "xmlsec1 rejected the assertion issued over the post binding\nstdout: {}\nstderr: {}\ndocument: {}",
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr),
                response_xml
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn an_unregistered_service_provider_is_refused() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let unknown_issuer = format!(
                "https://unknown-{}.example.com/saml/metadata",
                Uuid::new_v4().simple()
            );
            let request = authn_request(&realm, &unknown_issuer, None);

            let started = start_sso(&srv, &realm, &request, true).await;

            assert_eq!(
                started.status_code(),
                400,
                "an authn request from an unregistered service provider must be refused: {}",
                started.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn a_mismatched_acs_url_is_refused() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let registered_acs = "https://sp.example.com/saml/acs/registered";
            let (_client_id, sp_entity_id) =
                create_saml_service_provider(&srv, &realm, &admin_token, registered_acs).await;

            let request = authn_request(
                &realm,
                &sp_entity_id,
                Some("https://sp.example.com/saml/acs/not-registered"),
            );

            let started = start_sso(&srv, &realm, &request, true).await;

            assert_eq!(
                started.status_code(),
                400,
                "an authn request naming an unregistered acs url must be refused: {}",
                started.text()
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL and xmlsec1 — run with: cargo test -p ferriskey-api --test saml_test -- --ignored"]
    fn a_replayed_authorization_code_is_refused() {
        let srv = server();
        let realm = ctx().realm_name.clone();
        rt().block_on(async {
            let admin_token = admin_token(&srv, &realm).await;
            let acs_url = "https://sp.example.com/saml/acs/replay";
            let (client_id, sp_entity_id) =
                create_saml_service_provider(&srv, &realm, &admin_token, acs_url).await;

            let request = authn_request(&realm, &sp_entity_id, Some(acs_url));
            let started = start_sso(&srv, &realm, &request, true).await;
            let continuation_url = complete_login(&srv, &realm, &started, &client_id).await;
            let path = continue_path(&continuation_url);

            let first = srv.get(&path).await;
            assert_eq!(
                first.status_code(),
                200,
                "the first delivery of the assertion must succeed: {}",
                first.text()
            );

            let replayed = srv.get(&path).await;
            assert_eq!(
                replayed.status_code(),
                400,
                "replaying the same authorization code must not mint a second assertion: {}",
                replayed.text()
            );
        });
    }
}
