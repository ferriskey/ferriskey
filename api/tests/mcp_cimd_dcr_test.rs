/// Integration tests for Client ID Metadata Documents and Dynamic Client Registration.
///
///   DATABASE_HOST=127.0.0.1 DATABASE_PORT=55434 \
///     cargo test -p ferriskey-api --test mcp_cimd_dcr_test -- --ignored
#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        env,
        sync::{
            Arc, Mutex, MutexGuard, OnceLock,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use axum::{
        Router,
        extract::State,
        http::{HeaderMap, StatusCode, Uri, header},
        response::{IntoResponse, Response},
    };
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

    const ADMIN_PASSWORD: &str = "admin_pass_1234!";
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    const CALLBACK: &str = "https://app.example/callback";
    const EXCHANGE: &str = "urn:ietf:params:oauth:grant-type:token-exchange";
    const ACCESS_TOKEN_TYPE: &str = "urn:ietf:params:oauth:token-type:access_token";

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    struct Ctx {
        router: Mutex<Router>,
        realm: String,
        realm_id: Uuid,
        pool: PgPool,
    }

    impl Ctx {
        fn server(&self) -> TestServer {
            let router = self.router.lock().expect("router mutex").clone();
            TestServer::new(router).expect("test server")
        }

        fn path(&self, suffix: &str) -> String {
            format!("/realms/{}{suffix}", self.realm)
        }

        fn oidc(&self, endpoint: &str) -> String {
            self.path(&format!("/protocol/openid-connect/{endpoint}"))
        }

        async fn toggles(&self, cimd: bool, dcr: bool, hosts: &[&str]) {
            let hosts: Vec<String> = hosts.iter().map(|h| h.to_string()).collect();
            sqlx::query(
                "UPDATE realm_settings SET cimd_enabled = $1, dcr_enabled = $2, cimd_allowed_hosts = $3 WHERE realm_id = $4",
            )
            .bind(cimd)
            .bind(dcr)
            .bind(hosts)
            .bind(self.realm_id)
            .execute(&self.pool)
            .await
            .expect("set toggles");
        }

        async fn admin_token(&self, server: &TestServer) -> String {
            let response = server
                .post(&self.oidc("token"))
                .form(&[
                    ("grant_type", "password"),
                    ("client_id", "admin-cli"),
                    ("username", "admin"),
                    ("password", ADMIN_PASSWORD),
                    ("scope", "openid"),
                ])
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            response.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token")
                .to_string()
        }

        async fn client_row(&self, client_id: &str) -> Option<(String, bool, bool, bool, bool)> {
            sqlx::query_as(
                "SELECT registration_source, consent_required, public_client, COALESCE(require_pkce, false), token_exchange_enabled FROM clients WHERE client_id = $1 AND realm_id = $2",
            )
            .bind(client_id)
            .bind(self.realm_id)
            .fetch_optional(&self.pool)
            .await
            .expect("client row")
        }
    }

    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    static PRIVATE_ON: OnceLock<Ctx> = OnceLock::new();
    static PRIVATE_OFF: OnceLock<Ctx> = OnceLock::new();
    static LOCAL: OnceLock<LocalServer> = OnceLock::new();
    static SERIAL: Mutex<()> = Mutex::new(());

    fn rt() -> &'static tokio::runtime::Runtime {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime")
        })
    }

    fn serial() -> MutexGuard<'static, ()> {
        SERIAL
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn on() -> &'static Ctx {
        PRIVATE_ON.get_or_init(|| rt().block_on(boot(true)))
    }

    fn off() -> &'static Ctx {
        PRIVATE_OFF.get_or_init(|| rt().block_on(boot(false)))
    }

    async fn boot(allow_private: bool) -> Ctx {
        let host = env_or("DATABASE_HOST", "localhost");
        let port: u16 = env_or("DATABASE_PORT", "5432").parse().unwrap_or(5432);
        let name = env_or("DATABASE_NAME", "ferriskey");
        let user = env_or("DATABASE_USER", "ferriskey");
        let password = env_or("DATABASE_PASSWORD", "ferriskey");
        let schema = format!("cimd_dcr_{}", Uuid::new_v4().simple());
        let base = format!("postgres://{user}:{password}@{host}:{port}/{name}");

        PgPool::connect(&base)
            .await
            .expect("admin pool")
            .execute(format!("CREATE SCHEMA IF NOT EXISTS \"{schema}\"").as_str())
            .await
            .expect("create schema");

        let pool = PgPool::connect(&format!(
            "{base}?options=-c search_path={}",
            urlencoding::encode(&schema)
        ))
        .await
        .expect("schema pool");
        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("migrations");

        let service = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: allow_private,
            webapp_url: "http://localhost:5555".to_string(),
            database: DatabaseConfig {
                host,
                port,
                username: user,
                password,
                name,
                schema,
            },
        })
        .await
        .expect("service");

        let realm = format!("mcp-{}", Uuid::new_v4().simple());
        service
            .initialize_application(StartupConfig {
                webapp_url: "http://localhost:5555".to_string(),
                master_realm_name: realm.clone(),
                admin_username: "admin".to_string(),
                admin_email: "admin@ferriskey.test".to_string(),
                admin_password: ADMIN_PASSWORD.to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize");

        let (realm_id,): (Uuid,) = sqlx::query_as("SELECT id FROM realms WHERE name = $1")
            .bind(&realm)
            .fetch_one(&pool)
            .await
            .expect("realm id");

        let router = router(AppState::new(Arc::new(Args::default()), service)).expect("router");

        Ctx {
            router: Mutex::new(router),
            realm,
            realm_id,
            pool,
        }
    }

    struct Doc {
        body: String,
        content_type: &'static str,
        delay: Duration,
    }

    #[derive(Default)]
    struct Docs {
        docs: Mutex<HashMap<String, Doc>>,
        hits: AtomicUsize,
    }

    struct LocalServer {
        base: String,
        docs: Arc<Docs>,
    }

    async fn serve_doc(State(docs): State<Arc<Docs>>, uri: Uri) -> Response {
        docs.hits.fetch_add(1, Ordering::SeqCst);
        let found = {
            let guard = docs.docs.lock().expect("docs");
            guard
                .get(uri.path())
                .map(|doc| (doc.body.clone(), doc.content_type, doc.delay))
        };
        match found {
            Some((body, content_type, delay)) => {
                tokio::time::sleep(delay).await;
                let mut headers = HeaderMap::new();
                headers.insert(header::CONTENT_TYPE, content_type.parse().expect("ct"));
                headers.insert(header::CACHE_CONTROL, "max-age=60".parse().expect("cc"));
                (StatusCode::OK, headers, body).into_response()
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    fn local() -> &'static LocalServer {
        LOCAL.get_or_init(|| {
            let docs = Arc::new(Docs::default());
            let served = Arc::clone(&docs);
            let (sender, receiver) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("local server runtime");
                runtime.block_on(async move {
                    let app = Router::new().fallback(serve_doc).with_state(served);
                    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                        .await
                        .expect("bind");
                    let _ = sender.send(listener.local_addr().expect("addr"));
                    let _ = axum::serve(listener, app).await;
                });
            });
            let addr = receiver.recv().expect("local server address");
            LocalServer {
                base: format!("http://{addr}"),
                docs,
            }
        })
    }

    fn publish(body: String, content_type: &'static str, delay: Duration) -> String {
        let server = local();
        let path = format!("/doc/{}", Uuid::new_v4().simple());
        server.docs.docs.lock().expect("docs").insert(
            path.clone(),
            Doc {
                body,
                content_type,
                delay,
            },
        );
        format!("{}{path}", server.base)
    }

    fn document_for(url: &str, redirect_uris: &[&str]) -> Value {
        json!({
            "client_id": url,
            "client_name": "Test MCP Client",
            "redirect_uris": redirect_uris,
            "token_endpoint_auth_method": "none",
        })
    }

    fn publish_json(build: impl FnOnce(&str) -> Value) -> String {
        let server = local();
        let path = format!("/doc/{}", Uuid::new_v4().simple());
        let url = format!("{}{path}", server.base);
        server.docs.docs.lock().expect("docs").insert(
            path,
            Doc {
                body: build(&url).to_string(),
                content_type: "application/json",
                delay: Duration::ZERO,
            },
        );
        url
    }

    fn query_param(url: &str, key: &str) -> Option<String> {
        let query = url.split_once('?')?.1;
        query.split('&').find_map(|pair| {
            let (k, v) = pair.split_once('=')?;
            (k == key).then(|| {
                urlencoding::decode(v)
                    .map(|s| s.into_owned())
                    .unwrap_or_default()
            })
        })
    }

    fn assert_refused(response: &TestResponse, redirect_uri: &str) {
        assert!(
            response.maybe_cookie("FERRISKEY_SESSION").is_none(),
            "a refused client must not get an auth session"
        );
        let status = response.status_code().as_u16();
        let location = response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();
        assert!(
            status >= 400 || location.contains("login_error"),
            "expected a refusal, got {status} {location}"
        );
        assert!(
            !location.starts_with(redirect_uri),
            "the browser must not be sent to an unvalidated redirect uri"
        );
    }

    async fn authorize(
        server: &TestServer,
        ctx: &Ctx,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
    ) -> TestResponse {
        server
            .get(&ctx.oidc("auth"))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", client_id)
            .add_query_param("redirect_uri", redirect_uri)
            .add_query_param("scope", scope)
            .add_query_param("state", "st-123")
            .add_query_param("code_challenge", CHALLENGE)
            .add_query_param("code_challenge_method", "S256")
            .await
    }

    async fn sign_in(
        server: &TestServer,
        ctx: &Ctx,
        client_id: &str,
        redirect_uri: &str,
        scope: &str,
    ) -> (String, axum_extra::extract::cookie::Cookie<'static>) {
        let authorized = authorize(server, ctx, client_id, redirect_uri, scope).await;
        assert_eq!(
            authorized.status_code(),
            302,
            "authorize: {}",
            authorized.text()
        );
        let session = authorized.cookie("FERRISKEY_SESSION");

        let login = server
            .post(&ctx.path("/login-actions/authenticate"))
            .add_cookie(session.clone())
            .add_query_param("client_id", client_id)
            .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
            .await;
        assert_eq!(login.status_code(), 200, "authenticate: {}", login.text());

        let url = login.json::<Value>()["url"]
            .as_str()
            .expect("redirect url")
            .to_string();
        (url, session)
    }

    async fn complete_flow(
        server: &TestServer,
        ctx: &Ctx,
        client_id: &str,
        redirect_uri: &str,
    ) -> String {
        let (mut url, session) = sign_in(server, ctx, client_id, redirect_uri, "openid").await;

        if let Some(consent_token) = query_param(&url, "consent_token") {
            let decision = server
                .post(&ctx.path("/auth/consent"))
                .add_cookie(session)
                .json(&json!({ "consent_token": consent_token, "approved_scopes": [] }))
                .await;
            assert_eq!(decision.status_code(), 200, "{}", decision.text());
            url = decision.json::<Value>()["redirect_url"]
                .as_str()
                .expect("redirect_url")
                .to_string();
        }

        assert!(url.starts_with(redirect_uri), "redirect went to {url}");
        assert_eq!(query_param(&url, "state").as_deref(), Some("st-123"));
        query_param(&url, "code").expect("authorization code in the redirect")
    }

    async fn exchange(
        server: &TestServer,
        ctx: &Ctx,
        client_id: &str,
        secret: Option<&str>,
        redirect_uri: &str,
        code: &str,
    ) -> TestResponse {
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("client_id", client_id),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("code_verifier", VERIFIER),
        ];
        if let Some(secret) = secret {
            form.push(("client_secret", secret));
        }
        server.post(&ctx.oidc("token")).form(&form).await
    }

    async fn register(server: &TestServer, ctx: &Ctx, body: Value) -> TestResponse {
        server
            .post(&ctx.path("/clients/register"))
            .json(&body)
            .await
    }

    // ---------------------------------------------------------------- CIMD

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_metadata_document_client_completes_code_and_pkce() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|url| document_for(url, &[CALLBACK]));

            let code = complete_flow(&server, ctx, &url, CALLBACK).await;
            let token = exchange(&server, ctx, &url, None, CALLBACK, &code).await;
            assert_eq!(token.status_code(), 200, "{}", token.text());
            assert!(token.json::<Value>()["access_token"].is_string());

            let row = ctx.client_row(&url).await.expect("client row");
            assert_eq!(
                row,
                ("metadata_document".to_string(), true, true, true, false)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_consent_screen_names_the_metadata_document_host() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|url| document_for(url, &[CALLBACK]));
            let admin = ctx.admin_token(&server).await;

            let first = authorize(&server, ctx, &url, CALLBACK, "openid").await;
            assert_eq!(first.status_code(), 302, "{}", first.text());

            let (client_uuid,): (Uuid,) =
                sqlx::query_as("SELECT id FROM clients WHERE client_id = $1")
                    .bind(&url)
                    .fetch_one(&ctx.pool)
                    .await
                    .expect("client uuid");

            let scope = server
                .post(&ctx.path("/client-scopes"))
                .authorization_bearer(&admin)
                .json(&json!({
                    "name": format!("contacts-{}", Uuid::new_v4().simple()),
                    "description": "contacts",
                    "protocol": "openid-connect",
                    "is_default": false,
                }))
                .await;
            assert_eq!(scope.status_code(), 201, "{}", scope.text());
            let scope: Value = scope.json();
            let scope_id = scope["id"].as_str().expect("scope id");
            let scope_name = scope["name"].as_str().expect("scope name");

            let assign = server
                .put(&ctx.path(&format!(
                    "/clients/{client_uuid}/optional-client-scopes/{scope_id}"
                )))
                .authorization_bearer(&admin)
                .await;
            assert_eq!(assign.status_code(), 200, "{}", assign.text());

            let (consent_url, session) = sign_in(
                &server,
                ctx,
                &url,
                CALLBACK,
                &format!("openid {scope_name}"),
            )
            .await;
            let consent_token = query_param(&consent_url, "consent_token")
                .expect("a metadata document client always reaches consent");

            let view = server
                .get(&ctx.path("/auth/consent"))
                .add_cookie(session)
                .add_query_param("consent_token", &consent_token)
                .await;
            assert_eq!(view.status_code(), 200, "{}", view.text());
            let view: Value = view.json();
            assert_eq!(view["client_name"], "Test MCP Client");
            assert_eq!(view["client_uri_host"], "127.0.0.1");
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_document_client_id_that_differs_from_its_url_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|_| document_for("https://other.example/c.json", &[CALLBACK]));

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert!(ctx.client_row(&url).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_redirect_uri_missing_from_the_document_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|url| document_for(url, &[CALLBACK]));

            let response = authorize(
                &server,
                ctx,
                &url,
                "https://evil.example/callback",
                "openid",
            )
            .await;

            assert_refused(&response, "https://evil.example/callback");
            assert!(ctx.client_row(&url).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_oversized_document_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|url| {
                let mut doc = document_for(url, &[CALLBACK]);
                doc["padding"] = Value::String("a".repeat(6000));
                doc
            });

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert!(ctx.client_row(&url).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_document_that_answers_too_slowly_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let path_url = publish(String::new(), "application/json", Duration::from_secs(8));
            let body = document_for(&path_url, &[CALLBACK]).to_string();
            let path = path_url.trim_start_matches(&local().base).to_string();
            if let Some(doc) = local().docs.docs.lock().expect("docs").get_mut(&path) {
                doc.body = body;
            }

            let response = authorize(&server, ctx, &path_url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert!(ctx.client_row(&path_url).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_document_that_is_not_json_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish("<html></html>".to_string(), "text/html", Duration::ZERO);

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn private_key_jwt_documents_are_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = publish_json(|url| {
                let mut doc = document_for(url, &[CALLBACK]);
                doc["token_endpoint_auth_method"] = json!("private_key_jwt");
                doc
            });

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert!(ctx.client_row(&url).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_host_allowlist_is_enforced_before_any_fetch() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &["trusted.example"]).await;
            let url = publish_json(|url| document_for(url, &[CALLBACK]));
            let before = local().docs.hits.load(Ordering::SeqCst);

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert_eq!(local().docs.hits.load(Ordering::SeqCst), before);
            ctx.toggles(true, false, &[]).await;
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_client_id_longer_than_the_column_is_refused() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let url = format!("{}/doc/{}", local().base, "a".repeat(300));

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn without_the_test_flag_http_and_loopback_documents_are_refused() {
        let _guard = serial();
        let ctx = off();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(true, false, &[]).await;
            let http_url = publish_json(|url| document_for(url, &[CALLBACK]));
            let https_loopback = http_url.replacen("http://", "https://", 1);
            let before = local().docs.hits.load(Ordering::SeqCst);

            let http = authorize(&server, ctx, &http_url, CALLBACK, "openid").await;
            let loopback = authorize(&server, ctx, &https_loopback, CALLBACK, "openid").await;

            assert_refused(&http, CALLBACK);
            assert_refused(&loopback, CALLBACK);
            assert_eq!(local().docs.hits.load(Ordering::SeqCst), before);
            assert!(ctx.client_row(&http_url).await.is_none());
            assert!(ctx.client_row(&https_loopback).await.is_none());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn with_cimd_off_a_url_client_id_is_an_unknown_client() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(false, false, &[]).await;
            let url = publish_json(|url| document_for(url, &[CALLBACK]));
            let before = local().docs.hits.load(Ordering::SeqCst);

            let response = authorize(&server, ctx, &url, CALLBACK, "openid").await;

            assert_refused(&response, CALLBACK);
            assert_eq!(local().docs.hits.load(Ordering::SeqCst), before);
            assert!(ctx.client_row(&url).await.is_none());
        });
    }

    // ----------------------------------------------------------------- DCR

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_public_dynamic_client_registers_then_completes_code_and_pkce() {
        let _guard = serial();
        let ctx = off();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(false, true, &[]).await;

            let response = register(
                &server,
                ctx,
                json!({
                    "client_name": "Chat Client",
                    "redirect_uris": [CALLBACK],
                    "token_endpoint_auth_method": "none",
                    "grant_types": ["authorization_code", "refresh_token"],
                    "response_types": ["code"],
                }),
            )
            .await;
            assert_eq!(response.status_code(), 201, "{}", response.text());
            let body: Value = response.json();
            let client_id = body["client_id"].as_str().expect("client_id").to_string();
            assert!(Uuid::parse_str(&client_id).is_ok());
            assert!(body["client_id_issued_at"].as_i64().unwrap_or_default() > 0);
            assert!(body.get("client_secret").is_none());
            assert_eq!(body["token_endpoint_auth_method"], "none");
            assert_eq!(body["redirect_uris"], json!([CALLBACK]));
            assert_eq!(body["client_name"], "Chat Client");

            let code = complete_flow(&server, ctx, &client_id, CALLBACK).await;
            let token = exchange(&server, ctx, &client_id, None, CALLBACK, &code).await;
            assert_eq!(token.status_code(), 200, "{}", token.text());

            let row = ctx.client_row(&client_id).await.expect("client row");
            assert_eq!(row, ("dynamic".to_string(), true, true, true, false));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn registration_refuses_http_redirect_uris_off_loopback_and_bad_metadata() {
        let _guard = serial();
        let ctx = off();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(false, true, &[]).await;

            let http = register(
                &server,
                ctx,
                json!({ "redirect_uris": ["http://app.example/cb"], "token_endpoint_auth_method": "none" }),
            )
            .await;
            assert_eq!(http.status_code(), 400, "{}", http.text());
            assert_eq!(http.json::<Value>()["error"], "invalid_redirect_uri");

            let grant = register(
                &server,
                ctx,
                json!({ "redirect_uris": [CALLBACK], "grant_types": ["password"] }),
            )
            .await;
            assert_eq!(grant.status_code(), 400, "{}", grant.text());
            assert_eq!(grant.json::<Value>()["error"], "invalid_client_metadata");

            let loopback = register(
                &server,
                ctx,
                json!({ "redirect_uris": ["http://127.0.0.1:8123/cb"], "token_endpoint_auth_method": "none" }),
            )
            .await;
            assert_eq!(loopback.status_code(), 201, "{}", loopback.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_dynamic_client_cannot_use_token_exchange_nor_be_granted_it() {
        let _guard = serial();
        let ctx = off();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(false, true, &[]).await;

            let response = register(
                &server,
                ctx,
                json!({
                    "client_name": "Confidential Chat",
                    "redirect_uris": [CALLBACK],
                    "token_endpoint_auth_method": "client_secret_post",
                }),
            )
            .await;
            assert_eq!(response.status_code(), 201, "{}", response.text());
            let body: Value = response.json();
            let client_id = body["client_id"].as_str().expect("client_id").to_string();
            let secret = body["client_secret"].as_str().expect("secret").to_string();
            assert_eq!(body["client_secret_expires_at"], 0);

            let code = complete_flow(&server, ctx, &client_id, CALLBACK).await;
            let token = exchange(&server, ctx, &client_id, Some(&secret), CALLBACK, &code).await;
            assert_eq!(token.status_code(), 200, "{}", token.text());
            let access = token.json::<Value>()["access_token"]
                .as_str()
                .expect("access token")
                .to_string();

            sqlx::query("UPDATE clients SET token_exchange_enabled = true WHERE client_id = $1")
                .bind(&client_id)
                .execute(&ctx.pool)
                .await
                .expect("force the flag");

            let exchanged = server
                .post(&ctx.oidc("token"))
                .form(&[
                    ("grant_type", EXCHANGE),
                    ("client_id", client_id.as_str()),
                    ("client_secret", secret.as_str()),
                    ("subject_token", access.as_str()),
                    ("subject_token_type", ACCESS_TOKEN_TYPE),
                ])
                .await;
            assert_ne!(exchanged.status_code(), 200, "{}", exchanged.text());
            assert_eq!(
                exchanged.json::<Value>()["error"],
                "unauthorized_client",
                "{}",
                exchanged.text()
            );

            let admin = ctx.admin_token(&server).await;
            let (uuid,): (Uuid,) = sqlx::query_as("SELECT id FROM clients WHERE client_id = $1")
                .bind(&client_id)
                .fetch_one(&ctx.pool)
                .await
                .expect("uuid");
            let update = server
                .patch(&ctx.path(&format!("/clients/{uuid}")))
                .authorization_bearer(&admin)
                .json(&json!({ "token_exchange_enabled": true }))
                .await;
            assert_eq!(update.status_code(), 403, "{}", update.text());

            let listing = server
                .get(&ctx.path("/clients"))
                .authorization_bearer(&admin)
                .await;
            assert_eq!(listing.status_code(), 200, "{}", listing.text());
            assert!(
                listing
                    .text()
                    .contains("\"registration_source\":\"dynamic\"")
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn with_dcr_off_the_registration_endpoint_is_not_found() {
        let _guard = serial();
        let ctx = off();
        let server = ctx.server();
        rt().block_on(async {
            ctx.toggles(false, false, &[]).await;

            let response = register(
                &server,
                ctx,
                json!({ "redirect_uris": [CALLBACK], "token_endpoint_auth_method": "none" }),
            )
            .await;

            assert_eq!(response.status_code(), 404, "{}", response.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_admin_enables_the_toggles_through_the_settings_api() {
        let _guard = serial();
        let ctx = on();
        let server = ctx.server();
        rt().block_on(async {
            let admin = ctx.admin_token(&server).await;

            let response = server
                .put(&ctx.path("/settings"))
                .authorization_bearer(&admin)
                .json(&json!({
                    "cimd_enabled": true,
                    "dcr_enabled": true,
                    "cimd_allowed_hosts": ["app.example"],
                }))
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());

            let (cimd, dcr, hosts): (bool, bool, Vec<String>) = sqlx::query_as(
                "SELECT cimd_enabled, dcr_enabled, cimd_allowed_hosts FROM realm_settings WHERE realm_id = $1",
            )
            .bind(ctx.realm_id)
            .fetch_one(&ctx.pool)
            .await
            .expect("settings");
            assert!(cimd && dcr);
            assert_eq!(hosts, vec!["app.example".to_string()]);

            ctx.toggles(false, false, &[]).await;
        });
    }
}
