#![allow(dead_code)]

use std::{
    env,
    sync::{Arc, Mutex, OnceLock},
};

use axum::Router;
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
use sqlx::{Executor, PgPool};
use uuid::Uuid;

pub const ADMIN_USERNAME: &str = "admin";
pub const ADMIN_PASSWORD: &str = "admin";
pub const ADMIN_CLIENT_ID: &str = "admin-cli";
pub const CALLBACK: &str = "http://localhost/callback";

pub struct TestApp {
    router: Mutex<Router>,
    pub realm: String,
    pub realm_id: Uuid,
    pub pool: PgPool,
}

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static APP: OnceLock<TestApp> = OnceLock::new();

pub fn rt() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build shared runtime")
    })
}

pub fn app() -> &'static TestApp {
    APP.get_or_init(|| match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(TestApp::boot())),
        Err(_) => rt().block_on(TestApp::boot()),
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

impl TestApp {
    async fn boot() -> TestApp {
        let host = env_or("DATABASE_HOST", "localhost");
        let port = env_u16_or("DATABASE_PORT", 5432);
        let name = env_or("DATABASE_NAME", "ferriskey");
        let user = env_or("DATABASE_USER", "ferriskey");
        let password = env_or("DATABASE_PASSWORD", "ferriskey");
        let schema = format!("oidc_{}", Uuid::new_v4().simple());
        let base = format!("postgres://{user}:{password}@{host}:{port}/{name}");

        PgPool::connect(&base)
            .await
            .expect("connect admin pool")
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{schema}\""
            )))
            .await
            .expect("create schema");

        let pool = PgPool::connect(&format!(
            "{base}?options=-c search_path={}",
            urlencoding::encode(&schema)
        ))
        .await
        .expect("connect schema pool");

        sqlx::migrate!("../core/migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let service = create_service(FerriskeyConfig {
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: false,
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
        .expect("create service");

        let realm = format!("oidc-{}", Uuid::new_v4().simple());
        service
            .initialize_application(StartupConfig {
                webapp_url: "http://localhost:5555".to_string(),
                master_realm_name: realm.clone(),
                admin_username: ADMIN_USERNAME.to_string(),
                admin_password: ADMIN_PASSWORD.to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let (realm_id,): (Uuid,) = sqlx::query_as("SELECT id FROM realms WHERE name = $1")
            .bind(&realm)
            .fetch_one(&pool)
            .await
            .expect("fetch realm id");

        let state = AppState::new(Arc::new(Args::default()), service);
        let router = router(state).expect("build router");

        TestApp {
            router: Mutex::new(router),
            realm,
            realm_id,
            pool,
        }
    }

    pub fn server(&self) -> TestServer {
        let router = self.router.lock().expect("router mutex poisoned").clone();
        TestServer::new(router).expect("create test server")
    }

    pub fn path(&self, suffix: &str) -> String {
        format!("/realms/{}{suffix}", self.realm)
    }

    pub fn oidc(&self, endpoint: &str) -> String {
        self.path(&format!("/protocol/openid-connect/{endpoint}"))
    }

    pub async fn client(&self, spec: ClientSpec) -> SeededClient {
        let id = Uuid::new_v4();
        let client_id = format!("{}-{}", spec.prefix, Uuid::new_v4().simple());
        let now = chrono::Utc::now().naive_utc();
        let client_type = if spec.public {
            "public"
        } else {
            "confidential"
        };

        sqlx::query(
            r#"INSERT INTO clients
               (id, realm_id, name, client_id, secret, enabled, protocol, public_client,
                service_account_enabled, direct_access_grants_enabled, client_type,
                require_pkce, created_at, updated_at)
               VALUES ($1,$2,$3,$3,$4,$5,'openid-connect',$6,$7,$8,$9,$10,$11,$11)"#,
        )
        .bind(id)
        .bind(self.realm_id)
        .bind(&client_id)
        .bind(spec.secret.as_deref())
        .bind(spec.enabled)
        .bind(spec.public)
        .bind(spec.service_account)
        .bind(spec.direct_access)
        .bind(client_type)
        .bind(spec.require_pkce)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("insert client");

        for uri in &spec.redirect_uris {
            sqlx::query(
                "INSERT INTO redirect_uris (id, client_id, value, enabled) VALUES ($1,$2,$3,true)",
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(uri)
            .execute(&self.pool)
            .await
            .expect("insert redirect uri");
        }

        SeededClient {
            id,
            client_id,
            secret: spec.secret,
        }
    }

    pub async fn set_client_enabled(&self, client: &SeededClient, enabled: bool) {
        sqlx::query("UPDATE clients SET enabled = $1 WHERE id = $2")
            .bind(enabled)
            .bind(client.id)
            .execute(&self.pool)
            .await
            .expect("toggle client");
    }

    pub async fn add_post_logout_redirect_uri(&self, client: &SeededClient, uri: &str) {
        let now = chrono::Utc::now().naive_utc();
        sqlx::query(
            "INSERT INTO post_logout_redirect_uris (id, client_id, value, enabled, created_at, updated_at) VALUES ($1,$2,$3,true,$4,$4)",
        )
        .bind(Uuid::new_v4())
        .bind(client.id)
        .bind(uri)
        .bind(now)
        .execute(&self.pool)
        .await
        .expect("insert post logout redirect uri");
    }

    pub async fn set_id_token_lifetime(&self, client: &SeededClient, seconds: i32) {
        sqlx::query("UPDATE clients SET id_token_lifetime_secs = $1 WHERE id = $2")
            .bind(seconds)
            .bind(client.id)
            .execute(&self.pool)
            .await
            .expect("set id token lifetime");
    }

    pub async fn bind_service_account(&self, server: &TestServer, client: &SeededClient) {
        let user = self.user(server, "Service-Account-Passw0rd!").await;
        sqlx::query("UPDATE users SET client_id = $1 WHERE id = $2::uuid")
            .bind(client.id)
            .bind(&user.id)
            .execute(&self.pool)
            .await
            .expect("bind service account user");
    }

    pub async fn admin_token(&self, server: &TestServer) -> String {
        password_grant(
            server,
            self,
            ADMIN_CLIENT_ID,
            None,
            ADMIN_USERNAME,
            ADMIN_PASSWORD,
        )
        .await
        .json::<Value>()["access_token"]
            .as_str()
            .expect("admin access_token")
            .to_string()
    }

    pub async fn user(&self, server: &TestServer, password: &str) -> SeededUser {
        let token = self.admin_token(server).await;
        let username = format!("u-{}", Uuid::new_v4().simple());

        let created = server
            .post(&self.path("/users"))
            .authorization_bearer(&token)
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(
            created.status_code(),
            200,
            "create user: {}",
            created.text()
        );
        let id = created.json::<Value>()["data"]["id"]
            .as_str()
            .expect("created user id")
            .to_string();

        let reset = server
            .put(&self.path(&format!("/users/{id}/reset-password")))
            .authorization_bearer(&token)
            .json(&json!({
                "value": password,
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(reset.status_code(), 200, "reset password: {}", reset.text());

        SeededUser {
            id,
            username,
            password: password.to_string(),
        }
    }
}

pub struct ClientSpec {
    pub prefix: &'static str,
    pub public: bool,
    pub secret: Option<String>,
    pub enabled: bool,
    pub service_account: bool,
    pub direct_access: bool,
    pub require_pkce: bool,
    pub redirect_uris: Vec<String>,
}

impl ClientSpec {
    pub fn public() -> Self {
        Self {
            prefix: "public",
            public: true,
            secret: None,
            enabled: true,
            service_account: false,
            direct_access: false,
            require_pkce: false,
            redirect_uris: vec![CALLBACK.to_string()],
        }
    }

    pub fn confidential() -> Self {
        Self {
            prefix: "confidential",
            public: false,
            secret: Some(format!("secret-{}", Uuid::new_v4().simple())),
            ..Self::public()
        }
    }

    pub fn with_direct_access(mut self) -> Self {
        self.direct_access = true;
        self
    }

    pub fn with_service_account(mut self) -> Self {
        self.service_account = true;
        self
    }

    pub fn with_required_pkce(mut self) -> Self {
        self.require_pkce = true;
        self
    }
}

pub struct SeededClient {
    pub id: Uuid,
    pub client_id: String,
    pub secret: Option<String>,
}

pub struct SeededUser {
    pub id: String,
    pub username: String,
    pub password: String,
}

pub async fn authorize(
    server: &TestServer,
    app: &TestApp,
    client_id: &str,
    extra: &[(&str, &str)],
) -> axum_test::TestResponse {
    let mut request = server
        .get(&app.oidc("auth"))
        .add_query_param("response_type", "code")
        .add_query_param("client_id", client_id)
        .add_query_param("redirect_uri", CALLBACK)
        .add_query_param("scope", "openid");
    for (key, value) in extra {
        request = request.add_query_param(key, value);
    }
    request.await
}

pub async fn sign_in_for_code(
    server: &TestServer,
    app: &TestApp,
    client_id: &str,
    username: &str,
    password: &str,
    extra: &[(&str, &str)],
) -> String {
    let authorized = authorize(server, app, client_id, extra).await;
    assert_eq!(
        authorized.status_code(),
        302,
        "authorize: {}",
        authorized.text()
    );
    let session = authorized.cookie("FERRISKEY_SESSION");

    let authenticated = server
        .post(&app.path("/login-actions/authenticate"))
        .add_cookie(session)
        .add_query_param("client_id", client_id)
        .json(&json!({ "username": username, "password": password }))
        .await;
    assert_eq!(
        authenticated.status_code(),
        200,
        "authenticate: {}",
        authenticated.text()
    );

    let url = authenticated.json::<Value>()["url"]
        .as_str()
        .expect("redirect url")
        .to_string();
    query_param(&url, "code").expect("authorization code in redirect")
}

pub fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    let query = query.split('#').next().unwrap_or(query);
    query.split('&').find_map(|pair| {
        let (k, v) = pair.split_once('=')?;
        (k == key).then(|| {
            urlencoding::decode(v)
                .map(|s| s.into_owned())
                .unwrap_or_default()
        })
    })
}

pub async fn exchange_code(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    code: &str,
    extra: &[(&str, &str)],
) -> axum_test::TestResponse {
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "authorization_code"),
        ("client_id", client.client_id.as_str()),
        ("code", code),
        ("redirect_uri", CALLBACK),
    ];
    if let Some(secret) = client.secret.as_deref() {
        form.push(("client_secret", secret));
    }
    form.extend_from_slice(extra);
    server.post(&app.oidc("token")).form(&form).await
}

pub async fn password_grant(
    server: &TestServer,
    app: &TestApp,
    client_id: &str,
    client_secret: Option<&str>,
    username: &str,
    password: &str,
) -> axum_test::TestResponse {
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "password"),
        ("client_id", client_id),
        ("username", username),
        ("password", password),
        ("scope", "openid"),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }
    server.post(&app.oidc("token")).form(&form).await
}

pub async fn refresh(
    server: &TestServer,
    app: &TestApp,
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> axum_test::TestResponse {
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }
    server.post(&app.oidc("token")).form(&form).await
}

pub async fn introspect(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    token: &str,
) -> Value {
    let response = server
        .post(&app.oidc("token/introspect"))
        .form(&[
            ("client_id", client.client_id.as_str()),
            (
                "client_secret",
                client.secret.as_deref().unwrap_or_default(),
            ),
            ("token", token),
        ])
        .await;
    assert_eq!(
        response.status_code(),
        200,
        "introspect: {}",
        response.text()
    );
    response.json()
}

pub fn decode_unverified(jwt: &str) -> Value {
    use base64::Engine;
    let payload = jwt.split('.').nth(1).expect("jwt payload segment");
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .expect("base64url payload");
    serde_json::from_slice(&bytes).expect("json payload")
}

pub async fn client_credentials(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
) -> axum_test::TestResponse {
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "client_credentials"),
        ("client_id", client.client_id.as_str()),
    ];
    if let Some(secret) = client.secret.as_deref() {
        form.push(("client_secret", secret));
    }
    server.post(&app.oidc("token")).form(&form).await
}
