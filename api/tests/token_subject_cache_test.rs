use std::collections::BTreeSet;
use std::env;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use axum::Router;
use axum_test::{TestResponse, TestServer};
use base64::Engine as _;
use ferriskey_api::{
    application::http::server::{app_state::AppState, http_server::router},
    args::Args,
};
use ferriskey_core::{
    application::create_service_with_secrets,
    domain::common::{
        DatabaseConfig, FerriskeyConfig, PasswordHashingCost, entities::StartupConfig,
        ports::CoreService,
    },
};
use serde_json::{Value, json};
use sqlx::{Executor, PgPool};
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;

const PASSWORD: &str = "Correct-horse-battery-7";
const CACHE_TTL_SECONDS: u64 = 5;
const DELIVERY_DEADLINE: Duration = Duration::from_secs(2);
const SUBJECT_TABLES: &[&str] = &[
    "\"users\"",
    "\"user_role\"",
    "organization_groups",
    "organization_members",
    "\"organizations\"",
    "organization_attributes",
    "\"user_attributes\"",
    "user_required_actions",
];

const BACKGROUND_WORK: &[&str] = &["pg_notify", "webhook_deliveries", "webhook_outbox"];

static STATEMENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static RECORDER: OnceLock<()> = OnceLock::new();
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static CACHED: OnceLock<Deployment> = OnceLock::new();
static UNCACHED: OnceLock<Deployment> = OnceLock::new();

struct StatementRecorder;

impl<S: tracing::Subscriber> Layer<S> for StatementRecorder {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }
        let mut visitor = StatementVisitor::default();
        event.record(&mut visitor);
        if let Some(statement) = visitor.statement.or(visitor.summary) {
            STATEMENTS.lock().expect("statement log").push(statement);
        }
    }
}

#[derive(Default)]
struct StatementVisitor {
    summary: Option<String>,
    statement: Option<String>,
}

impl Visit for StatementVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "summary" => self.summary = Some(value.to_string()),
            "db.statement" if !value.trim().is_empty() => {
                self.statement = Some(value.trim().to_string())
            }
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        let rendered = rendered.trim_matches('"').trim().to_string();
        match field.name() {
            "summary" if self.summary.is_none() => self.summary = Some(rendered),
            "db.statement" if self.statement.is_none() && !rendered.is_empty() => {
                self.statement = Some(rendered)
            }
            _ => {}
        }
    }
}

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

struct Deployment {
    replicas: Vec<Router>,
    realm: String,
    realm_id: Uuid,
    scope_id: Uuid,
    pool: PgPool,
}

fn cached() -> &'static Deployment {
    CACHED.get_or_init(|| boot(CACHE_TTL_SECONDS, 2))
}

fn uncached() -> &'static Deployment {
    UNCACHED.get_or_init(|| boot(0, 1))
}

fn boot(config_cache_ttl_seconds: u64, replicas: usize) -> Deployment {
    RECORDER.get_or_init(|| {
        tracing_subscriber::registry()
            .with(StatementRecorder)
            .init();
    });
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(boot_async(config_cache_ttl_seconds, replicas))
        }),
        Err(_) => rt().block_on(boot_async(config_cache_ttl_seconds, replicas)),
    }
}

async fn boot_async(config_cache_ttl_seconds: u64, replicas: usize) -> Deployment {
    let host = env_or("DATABASE_HOST", "localhost");
    let port: u16 = env_or("DATABASE_PORT", "5432").parse().expect("port");
    let name = env_or("DATABASE_NAME", "ferriskey");
    let user = env_or("DATABASE_USER", "ferriskey");
    let password = env_or("DATABASE_PASSWORD", "ferriskey");
    let schema = format!("subject_{}", Uuid::new_v4().simple());
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

    let realm = format!("subject-{}", Uuid::new_v4().simple());
    let mut routers = Vec::with_capacity(replicas);
    for index in 0..replicas {
        let service = create_service_with_secrets(
            FerriskeyConfig {
                webhook_allow_private_endpoints: false,
                client_metadata_allow_private_endpoints: false,
                config_cache_ttl_seconds,
                password_hashing: PasswordHashingCost::default(),
                refresh_token_format: Default::default(),
                refresh_token_commit: Default::default(),
                webapp_url: "http://localhost:5555".to_string(),
                database: DatabaseConfig {
                    host: host.clone(),
                    port,
                    username: user.clone(),
                    password: password.clone(),
                    name: name.clone(),
                    schema: schema.clone(),
                    max_connections: 20,
                    min_connections: 1,
                },
            },
            "",
        )
        .await
        .expect("create service");

        if index == 0 {
            service
                .initialize_application(StartupConfig {
                    webapp_url: "http://localhost:5555".to_string(),
                    master_realm_name: realm.clone(),
                    admin_username: "admin".to_string(),
                    admin_password: "admin".to_string(),
                    admin_email: "admin@test.local".to_string(),
                    default_client_id: "ferriskey-admin".to_string(),
                })
                .await
                .expect("initialize application");
        }

        let state = AppState::new(Arc::new(Args::default()), service);
        routers.push(router(state).expect("build router"));
    }

    let (realm_id,): (Uuid,) = sqlx::query_as("SELECT id FROM realms WHERE name = $1")
        .bind(&realm)
        .fetch_one(&pool)
        .await
        .expect("fetch realm id");

    let scope_id = seed_claims_scope(&pool, realm_id).await;

    Deployment {
        replicas: routers,
        realm,
        realm_id,
        scope_id,
        pool,
    }
}

async fn seed_claims_scope(pool: &PgPool, realm_id: Uuid) -> Uuid {
    let scope_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_scopes (id, realm_id, name, description, protocol, default_scope_type, created_at, updated_at) \
         VALUES ($1, $2, 'subject-claims', 'subject-claims', 'openid-connect', 'DEFAULT', now(), now())",
    )
    .bind(scope_id)
    .bind(realm_id)
    .execute(pool)
    .await
    .expect("insert the claims scope");

    let mappers = [
        (
            "oidc-usermodel-realm-role-mapper",
            json!({"access.token.claim": "true", "id.token.claim": "false"}),
        ),
        (
            "oidc-group-membership-mapper",
            json!({"claim.name": "groups", "membership": "direct", "full.path": "true", "access.token.claim": "true", "id.token.claim": "false"}),
        ),
        (
            "oidc-organization-detail-mapper",
            json!({"claim.name": "organization", "include.attributes": "true", "access.token.claim": "true", "id.token.claim": "false"}),
        ),
        (
            "oidc-usermodel-attribute-mapper",
            json!({"user.attribute": "department", "claim.name": "department", "access.token.claim": "true", "id.token.claim": "false"}),
        ),
    ];
    for (mapper_type, config) in mappers {
        sqlx::query(
            "INSERT INTO client_scope_protocol_mappers (id, client_scope_id, name, mapper_type, config, created_at) \
             VALUES ($1, $2, $3, $3, $4, now())",
        )
        .bind(Uuid::new_v4())
        .bind(scope_id)
        .bind(mapper_type)
        .bind(config)
        .execute(pool)
        .await
        .expect("insert the mapper");
    }
    scope_id
}

fn claims(jwt: &str) -> Value {
    let payload = jwt.split('.').nth(1).expect("jwt payload segment");
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .expect("base64url payload");
    serde_json::from_slice(&bytes).expect("json payload")
}

fn names(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn recorded() -> usize {
    STATEMENTS.lock().expect("statement log").len()
}

async fn settle() {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        let before = recorded();
        tokio::time::sleep(Duration::from_millis(200)).await;
        if recorded() == before || tokio::time::Instant::now() >= deadline {
            break;
        }
    }
}

async fn capture<T>(work: impl Future<Output = T>) -> (T, Vec<String>) {
    settle().await;
    STATEMENTS.lock().expect("statement log").clear();
    let output = work.await;
    settle().await;
    let statements = STATEMENTS
        .lock()
        .expect("statement log")
        .drain(..)
        .filter(|statement| !BACKGROUND_WORK.iter().any(|work| statement.contains(work)))
        .collect();
    (output, statements)
}

fn subject_reads(statements: &[String]) -> Vec<&str> {
    statements
        .iter()
        .filter(|statement| {
            statement.trim_start().to_uppercase().starts_with("SELECT")
                || statement.trim_start().to_uppercase().starts_with("WITH")
        })
        .filter(|statement| SUBJECT_TABLES.iter().any(|table| statement.contains(table)))
        .map(|statement| statement.lines().next().unwrap_or_default())
        .collect()
}

impl Deployment {
    fn server(&self, replica: usize) -> TestServer {
        TestServer::new(self.replicas[replica].clone()).expect("create test server")
    }

    fn path(&self, suffix: &str) -> String {
        format!("/realms/{}{suffix}", self.realm)
    }

    fn token_endpoint(&self) -> String {
        self.path("/protocol/openid-connect/token")
    }

    async fn admin(&self, server: &TestServer) -> String {
        let response = server
            .post(&self.token_endpoint())
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        response.json::<Value>()["access_token"]
            .as_str()
            .expect("admin access token")
            .to_string()
    }

    async fn client(&self) -> String {
        let id = Uuid::new_v4();
        let client_id = format!("subject-client-{}", id.simple());
        sqlx::query(
            r#"INSERT INTO clients
               (id, realm_id, name, client_id, enabled, protocol, public_client,
                service_account_enabled, direct_access_grants_enabled, client_type,
                require_pkce, created_at, updated_at)
               VALUES ($1,$2,$3,$3,true,'openid-connect',true,false,true,'public',false,now(),now())"#,
        )
        .bind(id)
        .bind(self.realm_id)
        .bind(&client_id)
        .execute(&self.pool)
        .await
        .expect("insert client");
        sqlx::query(
            "INSERT INTO client_scope_mappings (client_id, client_scope_id, default_scope_type) VALUES ($1, $2, 'DEFAULT')",
        )
        .bind(id)
        .bind(self.scope_id)
        .execute(&self.pool)
        .await
        .expect("attach the claims scope");
        client_id
    }

    async fn user(&self, server: &TestServer) -> (Uuid, String) {
        let token = self.admin(server).await;
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
        assert_eq!(created.status_code(), 200, "{}", created.text());
        let id: Uuid = created.json::<Value>()["data"]["id"]
            .as_str()
            .expect("user id")
            .parse()
            .expect("uuid");
        let reset = server
            .put(&self.path(&format!("/users/{id}/reset-password")))
            .authorization_bearer(&token)
            .json(&json!({"value": PASSWORD, "temporary": false, "credential_type": "password"}))
            .await;
        assert_eq!(reset.status_code(), 200, "{}", reset.text());
        (id, username)
    }

    async fn role(&self, name: &str) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO roles (id, name, realm_id) VALUES ($1, $2, $3)")
            .bind(id)
            .bind(name)
            .bind(self.realm_id)
            .execute(&self.pool)
            .await
            .expect("insert role");
        id
    }

    async fn login(&self, server: &TestServer, client_id: &str, username: &str) -> Session {
        let response = server
            .post(&self.token_endpoint())
            .form(&[
                ("grant_type", "password"),
                ("client_id", client_id),
                ("username", username),
                ("password", PASSWORD),
                ("scope", "openid"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let body = response.json::<Value>();
        Session {
            client_id: client_id.to_string(),
            refresh_token: body["refresh_token"].as_str().expect("refresh").to_string(),
            access_token: body["access_token"].as_str().expect("access").to_string(),
        }
    }
}

struct Session {
    client_id: String,
    refresh_token: String,
    access_token: String,
}

impl Session {
    async fn try_refresh(&mut self, deployment: &Deployment, server: &TestServer) -> TestResponse {
        let response = server
            .post(&deployment.token_endpoint())
            .form(&[
                ("grant_type", "refresh_token"),
                ("client_id", self.client_id.as_str()),
                ("refresh_token", self.refresh_token.as_str()),
            ])
            .await;
        if response.status_code() == 200 {
            let body = response.json::<Value>();
            self.refresh_token = body["refresh_token"].as_str().expect("refresh").to_string();
            self.access_token = body["access_token"].as_str().expect("access").to_string();
        }
        response
    }

    async fn refresh(&mut self, deployment: &Deployment, server: &TestServer) -> Value {
        let response = self.try_refresh(deployment, server).await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        claims(&self.access_token)
    }

    async fn warm(&mut self, deployment: &Deployment, server: &TestServer) -> Vec<String> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
        loop {
            let (_, statements) = capture(self.refresh(deployment, server)).await;
            if subject_reads(&statements).is_empty() {
                println!(
                    "warm refresh: {} statements {statements:?}",
                    statements.len()
                );
                return statements;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "the subject reads never came from memory: {:?}",
                subject_reads(&statements)
            );
        }
    }

    async fn refresh_until(
        &mut self,
        deployment: &Deployment,
        server: &TestServer,
        reflects: impl Fn(&Value) -> bool,
    ) -> Value {
        let deadline = tokio::time::Instant::now() + DELIVERY_DEADLINE;
        loop {
            let token = self.refresh(deployment, server).await;
            if reflects(&token) {
                return token;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "the change did not reach a refreshed token in time: {token}"
            );
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

fn realm_roles(token: &Value) -> BTreeSet<String> {
    names(&token["realm_access"]["roles"])
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_14_role_assignment_reaches_next_refresh() {
    let deployment = cached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let server = deployment.server(0);
        let admin = deployment.admin(&server).await;
        let client = deployment.client().await;
        let (user_id, username) = deployment.user(&server).await;
        let role_name = format!("operator-{}", Uuid::new_v4().simple());
        let role_id = deployment.role(&role_name).await;
        let mut session = deployment.login(&server, &client, &username).await;
        session.warm(deployment, &server).await;
        assert!(!realm_roles(&claims(&session.access_token)).contains(&role_name));

        let assigned = server
            .post(&deployment.path(&format!("/users/{user_id}/roles/{role_id}")))
            .authorization_bearer(&admin)
            .await;
        assert_eq!(assigned.status_code(), 200, "{}", assigned.text());
        let token = session.refresh(deployment, &server).await;
        assert!(
            realm_roles(&token).contains(&role_name),
            "the assigned role is missing: {token}"
        );

        let revoked = server
            .delete(&deployment.path(&format!("/users/{user_id}/roles/{role_id}")))
            .authorization_bearer(&admin)
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
        let token = session.refresh(deployment, &server).await;
        assert!(
            !realm_roles(&token).contains(&role_name),
            "the revoked role is still present: {token}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_15_role_change_on_another_replica_reaches_the_next_refresh() {
    let deployment = cached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let (first, second) = (deployment.server(0), deployment.server(1));
        let admin = deployment.admin(&second).await;
        let client = deployment.client().await;
        let (user_id, username) = deployment.user(&first).await;
        let role_name = format!("operator-{}", Uuid::new_v4().simple());
        let role_id = deployment.role(&role_name).await;
        let mut session = deployment.login(&first, &client, &username).await;
        session.warm(deployment, &first).await;

        let assigned = second
            .post(&deployment.path(&format!("/users/{user_id}/roles/{role_id}")))
            .authorization_bearer(&admin)
            .await;
        assert_eq!(assigned.status_code(), 200, "{}", assigned.text());
        session
            .refresh_until(deployment, &first, |token| {
                realm_roles(token).contains(&role_name)
            })
            .await;

        let revoked = second
            .delete(&deployment.path(&format!("/users/{user_id}/roles/{role_id}")))
            .authorization_bearer(&admin)
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
        session
            .refresh_until(deployment, &first, |token| {
                !realm_roles(token).contains(&role_name)
            })
            .await;
    });
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_16_a_user_disabled_through_the_user_repository_cannot_refresh() {
    let deployment = cached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let server = deployment.server(0);
        let admin = deployment.admin(&server).await;
        let client = deployment.client().await;
        let (user_id, username) = deployment.user(&server).await;
        let mut session = deployment.login(&server, &client, &username).await;
        session.warm(deployment, &server).await;

        let disabled = server
            .put(&deployment.path(&format!("/users/{user_id}")))
            .authorization_bearer(&admin)
            .json(&json!({"enabled": false}))
            .await;
        assert_eq!(disabled.status_code(), 200, "{}", disabled.text());

        let refused = session.try_refresh(deployment, &server).await;
        assert!(
            refused.status_code().is_client_error(),
            "a disabled user refreshed: {} {}",
            refused.status_code(),
            refused.text()
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_17_role_rename_group_move_and_organization_attribute_reach_next_refresh() {
    let deployment = cached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let server = deployment.server(0);
        let admin = deployment.admin(&server).await;
        let client = deployment.client().await;
        let (user_id, username) = deployment.user(&server).await;
        let suffix = Uuid::new_v4().simple().to_string();

        let direct_name = format!("direct-{suffix}");
        let direct = deployment.role(&direct_name).await;
        sqlx::query("INSERT INTO user_role (user_id, role_id) VALUES ($1, $2)")
            .bind(user_id)
            .bind(direct)
            .execute(&deployment.pool)
            .await
            .expect("assign the direct role");

        let organization_id = Uuid::new_v4();
        sqlx::query("INSERT INTO organizations (id, realm_id, name, alias) VALUES ($1, $2, $3, $3)")
            .bind(organization_id)
            .bind(deployment.realm_id)
            .bind(format!("org-{suffix}"))
            .execute(&deployment.pool)
            .await
            .expect("insert organization");
        sqlx::query("INSERT INTO organization_members (id, organization_id, user_id) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(organization_id)
            .bind(user_id)
            .execute(&deployment.pool)
            .await
            .expect("add the user to the organization");

        let inherited_name = format!("inherited-{suffix}");
        let inherited = deployment.role(&inherited_name).await;
        let mut groups = Vec::new();
        for name in ["blessed", "plain"] {
            let id = Uuid::new_v4();
            sqlx::query("INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, $4)")
                .bind(id)
                .bind(organization_id)
                .bind(None::<Uuid>)
                .bind(name)
                .execute(&deployment.pool)
                .await
                .expect("insert group");
            groups.push(id);
        }
        let (blessed, plain) = (groups[0], groups[1]);
        let leaf = Uuid::new_v4();
        sqlx::query("INSERT INTO organization_groups (id, organization_id, parent_group_id, name) VALUES ($1, $2, $3, 'leaf')")
            .bind(leaf)
            .bind(organization_id)
            .bind(blessed)
            .execute(&deployment.pool)
            .await
            .expect("insert leaf group");
        sqlx::query("INSERT INTO organization_group_roles (id, group_id, role_id) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(blessed)
            .bind(inherited)
            .execute(&deployment.pool)
            .await
            .expect("attach the role to the group");
        sqlx::query("INSERT INTO organization_group_members (id, group_id, user_id) VALUES ($1, $2, $3)")
            .bind(Uuid::new_v4())
            .bind(leaf)
            .bind(user_id)
            .execute(&deployment.pool)
            .await
            .expect("add the user to the leaf group");

        let attribute = server
            .put(&deployment.path(&format!("/organizations/{organization_id}/attributes/tier")))
            .authorization_bearer(&admin)
            .json(&json!({"value": "silver"}))
            .await;
        assert_eq!(attribute.status_code(), 200, "{}", attribute.text());

        let mut session = deployment.login(&server, &client, &username).await;
        session.warm(deployment, &server).await;
        let token = session.refresh(deployment, &server).await;
        assert_eq!(
            realm_roles(&token),
            BTreeSet::from([direct_name.clone(), inherited_name.clone()]),
            "{token}"
        );
        assert_eq!(token["organization"]["attributes"]["tier"], json!("silver"), "{token}");

        let renamed_name = format!("renamed-{suffix}");
        let renamed = server
            .put(&deployment.path(&format!("/roles/{direct}")))
            .authorization_bearer(&admin)
            .json(&json!({"name": renamed_name}))
            .await;
        assert_eq!(renamed.status_code(), 200, "{}", renamed.text());
        let token = session.refresh(deployment, &server).await;
        assert_eq!(
            realm_roles(&token),
            BTreeSet::from([renamed_name, inherited_name.clone()]),
            "the rename is missing: {token}"
        );

        let moved = server
            .put(&deployment.path(&format!("/organizations/{organization_id}/groups/{leaf}")))
            .authorization_bearer(&admin)
            .json(&json!({"parent_group_id": plain}))
            .await;
        assert_eq!(moved.status_code(), 200, "{}", moved.text());
        let token = session.refresh(deployment, &server).await;
        assert!(
            !realm_roles(&token).contains(&inherited_name),
            "the group move is missing: {token}"
        );

        let changed = server
            .put(&deployment.path(&format!("/organizations/{organization_id}/attributes/tier")))
            .authorization_bearer(&admin)
            .json(&json!({"value": "gold"}))
            .await;
        assert_eq!(changed.status_code(), 200, "{}", changed.text());
        let token = session.refresh(deployment, &server).await;
        assert_eq!(
            token["organization"]["attributes"]["tier"],
            json!("gold"),
            "the attribute change is missing: {token}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_18_with_a_zero_ttl_every_subject_read_hits_the_database() {
    let deployment = uncached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let server = deployment.server(0);
        let client = deployment.client().await;
        let (_, username) = deployment.user(&server).await;
        let mut session = deployment.login(&server, &client, &username).await;
        session.refresh(deployment, &server).await;

        for _ in 0..2 {
            let (_, statements) = capture(session.refresh(deployment, &server)).await;
            let reads = subject_reads(&statements);
            for table in [
                "\"users\"",
                "\"user_role\"",
                "organization_groups",
                "organization_members",
            ] {
                assert!(
                    reads.iter().any(|read| read.contains(table))
                        || statements.iter().any(|s| s.contains(table)),
                    "no read of {table} with a zero ttl: {statements:?}"
                );
            }
        }
    });
}

#[test]
#[ignore = "requires PostgreSQL, run: cargo test -p ferriskey-api --test token_subject_cache_test -- --ignored"]
fn spec_rtp_19_warm_userinfo_runs_no_sql_besides_the_first_sight_revocation_check() {
    let deployment = cached();
    rt().block_on(async {
        let _serial = SERIAL.lock().await;
        let server = deployment.server(0);
        let client = deployment.client().await;
        let (_, username) = deployment.user(&server).await;
        let session = deployment.login(&server, &client, &username).await;
        let userinfo = || async {
            server
                .get(&deployment.path("/protocol/openid-connect/userinfo"))
                .authorization_bearer(&session.access_token)
                .await
        };

        let first = userinfo().await;
        assert_eq!(first.status_code(), 200, "{}", first.text());

        let deadline = tokio::time::Instant::now() + Duration::from_secs(4);
        loop {
            let (response, statements) = capture(userinfo()).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            println!(
                "warm userinfo: {} statements {statements:?}",
                statements.len()
            );
            if subject_reads(&statements).is_empty() {
                assert!(statements.len() <= 1, "{statements:?}");
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "userinfo kept reading the subject: {statements:?}"
            );
        }
    });
}
