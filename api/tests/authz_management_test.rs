/// Integration tests for the authorization management routes.
///
///   DATABASE_HOST=127.0.0.1 DATABASE_PORT=5432 \
///     cargo test -p ferriskey-api --test authz_management_test -- --ignored
#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::PasswordHashingCost;
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

    const MASTER: &str = "master";
    const ADMIN_PASSWORD: &str = "admin";

    const DOCUMENT_SCHEMA: &str = r#"namespace App {
    entity Document { classification: String };
    action "document:read" appliesTo { principal: [FerrisKey::User], resource: [Document] };
}"#;

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    struct Ctx {
        enabled: Mutex<Router>,
        disabled: Mutex<Router>,
        pool: PgPool,
        admin_token: String,
    }

    impl Ctx {
        fn server(&self) -> TestServer {
            let router = self.enabled.lock().expect("router mutex").clone();
            TestServer::new(router).expect("test server")
        }

        fn disabled_server(&self) -> TestServer {
            let router = self.disabled.lock().expect("router mutex").clone();
            TestServer::new(router).expect("test server")
        }
    }

    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    static CTX: OnceLock<Ctx> = OnceLock::new();

    fn rt() -> &'static tokio::runtime::Runtime {
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("runtime")
        })
    }

    fn ctx() -> &'static Ctx {
        CTX.get_or_init(|| rt().block_on(boot()))
    }

    async fn boot() -> Ctx {
        let host = env_or("DATABASE_HOST", "localhost");
        let port: u16 = env_or("DATABASE_PORT", "5432").parse().unwrap_or(5432);
        let name = env_or("DATABASE_NAME", "ferriskey");
        let user = env_or("DATABASE_USER", "ferriskey");
        let password = env_or("DATABASE_PASSWORD", "ferriskey");
        let schema = format!("authz_management_{}", Uuid::new_v4().simple());
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
            client_metadata_allow_private_endpoints: false,
            config_cache_ttl_seconds: 0,
            password_hashing: PasswordHashingCost::default(),
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

        service
            .initialize_application(StartupConfig {
                webapp_url: "http://localhost:5555".to_string(),
                master_realm_name: MASTER.to_string(),
                admin_username: "admin".to_string(),
                admin_password: ADMIN_PASSWORD.to_string(),
                admin_email: "admin@ferriskey.test".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize");

        let enabled = router(AppState::new(
            Arc::new(Args {
                authz_enabled: true,
                ..Args::default()
            }),
            service.clone(),
        ))
        .expect("enabled router");
        let disabled =
            router(AppState::new(Arc::new(Args::default()), service)).expect("disabled router");

        let server = TestServer::new(enabled.clone()).expect("test server");
        let response = server
            .post(&format!("/realms/{MASTER}/protocol/openid-connect/token"))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", ADMIN_PASSWORD),
                ("scope", "openid"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let admin_token = response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string();

        Ctx {
            enabled: Mutex::new(enabled),
            disabled: Mutex::new(disabled),
            pool,
            admin_token,
        }
    }

    struct Pep {
        token: String,
    }

    struct Realm {
        name: String,
        id: Uuid,
    }

    async fn create_realm(ctx: &Ctx, server: &TestServer) -> Realm {
        let name = format!("t-{}", Uuid::new_v4().simple());
        let response = server
            .post("/realms")
            .authorization_bearer(&ctx.admin_token)
            .json(&json!({ "name": name }))
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let (id,): (Uuid,) = sqlx::query_as("SELECT id FROM realms WHERE name = $1")
            .bind(&name)
            .fetch_one(&ctx.pool)
            .await
            .expect("realm id");
        Realm { name, id }
    }

    async fn create_role(
        ctx: &Ctx,
        server: &TestServer,
        realm: &Realm,
        permissions: &[&str],
    ) -> Uuid {
        let name = format!("r-{}", Uuid::new_v4().simple());
        let created = server
            .post(&format!("/realms/{}/roles", realm.name))
            .authorization_bearer(&ctx.admin_token)
            .json(&json!({ "name": name, "permissions": permissions }))
            .await;
        assert!(created.status_code().is_success(), "{}", created.text());
        let (id,): (Uuid,) =
            sqlx::query_as("SELECT id FROM roles WHERE name = $1 AND realm_id = $2")
                .bind(&name)
                .bind(realm.id)
                .fetch_one(&ctx.pool)
                .await
                .expect("role id");
        id
    }

    async fn grant(ctx: &Ctx, user: Uuid, role: Uuid) {
        sqlx::query(
            "INSERT INTO user_role (user_id, role_id, created_at, updated_at) VALUES ($1, $2, now(), now())",
        )
        .bind(user)
        .bind(role)
        .execute(&ctx.pool)
        .await
        .expect("assign role");
    }

    async fn create_pep(
        ctx: &Ctx,
        server: &TestServer,
        realm: &Realm,
        permissions: &[&str],
    ) -> Pep {
        let client_id = format!("pep-{}", Uuid::new_v4().simple());
        let created = server
            .post(&format!("/realms/{}/clients", realm.name))
            .authorization_bearer(&ctx.admin_token)
            .json(&json!({
                "name": client_id,
                "client_id": client_id,
                "client_type": "confidential",
                "service_account_enabled": true,
                "enabled": true,
            }))
            .await;
        assert_eq!(created.status_code(), 201, "{}", created.text());

        let secret = format!("secret-{}", Uuid::new_v4().simple());
        let (client_uuid,): (Uuid,) = sqlx::query_as(
            "UPDATE clients SET secret = $1 WHERE client_id = $2 AND realm_id = $3 RETURNING id",
        )
        .bind(&secret)
        .bind(&client_id)
        .bind(realm.id)
        .fetch_one(&ctx.pool)
        .await
        .expect("client secret");

        if !permissions.is_empty() {
            let (service_account,): (Uuid,) =
                sqlx::query_as("SELECT id FROM users WHERE client_id = $1")
                    .bind(client_uuid)
                    .fetch_one(&ctx.pool)
                    .await
                    .expect("service account");
            let role = create_role(ctx, server, realm, permissions).await;
            grant(ctx, service_account, role).await;
        }

        let token = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm.name
            ))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client_id.as_str()),
                ("client_secret", secret.as_str()),
            ])
            .await;
        assert_eq!(token.status_code(), 200, "{}", token.text());
        Pep {
            token: token.json::<Value>()["access_token"]
                .as_str()
                .expect("access_token")
                .to_string(),
        }
    }

    fn base(realm: &Realm) -> String {
        format!("/realms/{}/authz", realm.name)
    }

    fn static_policy(name: &str, source: &str) -> Value {
        json!({ "name": name, "body": { "type": "static", "source": source } })
    }

    const READ_ANY: &str =
        "permit (principal, action == App::Action::\"document:read\", resource);";

    async fn setup_schema(ctx: &Ctx, server: &TestServer, realm: &Realm) {
        let response = server
            .put(&format!("{}/schema", base(realm)))
            .authorization_bearer(&ctx.admin_token)
            .json(&json!({ "source": DOCUMENT_SCHEMA }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_realm_writes_its_schema_policies_and_state_through_the_api() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let token = &ctx.admin_token;

            let missing = server
                .get(&format!("{}/schema", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(missing.status_code(), 404, "{}", missing.text());

            setup_schema(ctx, &server, &realm).await;
            let schema = server
                .get(&format!("{}/schema", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(schema.status_code(), 200, "{}", schema.text());
            assert_eq!(schema.json::<Value>()["namespace"], json!("App"));

            let created = server
                .post(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .json(&static_policy("readers", READ_ANY))
                .await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let created = created.json::<Value>();
            let id = created["id"].as_str().expect("id").to_string();
            assert_eq!(created["enabled"], json!(true));
            assert_eq!(created["origin"], json!("custom"));

            let listed = server
                .get(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(listed.json::<Value>().as_array().map(Vec::len), Some(1));

            let replaced = server
                .put(&format!("{}/policies/{id}", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({
                    "name": "readers",
                    "description": "everyone reads",
                    "enabled": false,
                    "body": { "type": "static", "source": READ_ANY },
                }))
                .await;
            assert_eq!(replaced.status_code(), 200, "{}", replaced.text());
            assert_eq!(replaced.json::<Value>()["enabled"], json!(false));

            let state = server
                .put(&format!("{}/state", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({ "enabled": true }))
                .await;
            assert_eq!(state.status_code(), 200, "{}", state.text());
            assert_eq!(state.json::<Value>()["enabled"], json!(true));
            let state = server
                .get(&format!("{}/state", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(state.json::<Value>()["enabled"], json!(true));

            let deleted = server
                .delete(&format!("{}/policies/{id}", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(deleted.status_code(), 204, "{}", deleted.text());
            let gone = server
                .get(&format!("{}/policies/{id}", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(gone.status_code(), 404, "{}", gone.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_policy_that_does_not_validate_is_refused_and_validate_reports_positions() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            setup_schema(ctx, &server, &realm).await;
            let token = &ctx.admin_token;

            let refused = server
                .post(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .json(&static_policy(
                    "folders",
                    "permit (\n  principal,\n  action,\n  resource is App::Folder\n);",
                ))
                .await;
            assert_eq!(refused.status_code(), 422, "{}", refused.text());

            let validate = |policies: Value| {
                let server = &server;
                let url = format!("{}/policies/validate", base(&realm));
                async move {
                    server
                        .post(&url)
                        .authorization_bearer(token)
                        .json(&json!({ "policies": policies }))
                        .await
                }
            };

            let good = validate(json!([static_policy("ok", READ_ANY)])).await;
            assert_eq!(good.status_code(), 200, "{}", good.text());
            assert_eq!(good.json::<Value>(), json!({ "valid": true, "issues": [] }));

            let typed = validate(json!([static_policy(
                "folders",
                "permit (\n  principal,\n  action,\n  resource is App::Folder\n);"
            )]))
            .await;
            assert_eq!(typed.status_code(), 200, "{}", typed.text());
            let typed = typed.json::<Value>();
            assert_eq!(typed["valid"], json!(false));
            assert_eq!(typed["issues"][0]["policy"], json!("folders"));
            assert_eq!(typed["issues"][0]["line"], json!(4));
            assert!(typed["issues"][0]["column"].is_u64(), "{typed}");

            let syntax = validate(json!([static_policy(
                "broken",
                "permit (principal action resource);"
            )]))
            .await;
            let syntax = syntax.json::<Value>();
            assert_eq!(syntax["valid"], json!(false));
            assert_eq!(syntax["issues"][0]["policy"], json!("broken"));

            let listed = server
                .get(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(
                listed.json::<Value>().as_array().map(Vec::len),
                Some(0),
                "validate stores nothing"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_simulation_decides_from_the_given_policies_and_entities_without_saving() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            setup_schema(ctx, &server, &realm).await;
            let token = &ctx.admin_token;

            let simulate = |parents: Value| {
                let server = &server;
                let url = format!("{}/simulate", base(&realm));
                async move {
                    server
                        .post(&url)
                        .authorization_bearer(token)
                        .json(&json!({
                            "policies": [static_policy(
                                "support-reads",
                                "permit (principal in FerrisKey::Group::\"support\", action == App::Action::\"document:read\", resource is App::Document);"
                            )],
                            "entities": [],
                            "request": {
                                "subject": {
                                    "type": "FerrisKey::User",
                                    "id": "u1",
                                    "properties": {
                                        "username": "u1",
                                        "email_verified": true,
                                        "enabled": true,
                                    },
                                    "parents": parents,
                                },
                                "action": { "name": "document:read" },
                                "resource": {
                                    "type": "App::Document",
                                    "id": "1",
                                    "properties": { "classification": "public" },
                                },
                            },
                        }))
                        .await
                }
            };

            let member = simulate(json!([{ "type": "FerrisKey::Group", "id": "support" }])).await;
            assert_eq!(member.status_code(), 200, "{}", member.text());
            assert_eq!(
                member.json::<Value>(),
                json!({
                    "decision": true,
                    "context": { "reason_admin": { "policies": ["support-reads"] } },
                })
            );

            let outsider = simulate(json!([])).await;
            assert_eq!(outsider.status_code(), 200, "{}", outsider.text());
            assert_eq!(outsider.json::<Value>()["decision"], json!(false));

            let listed = server
                .get(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(listed.json::<Value>().as_array().map(Vec::len), Some(0));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_manager_only_touches_the_policies_it_labelled() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            setup_schema(ctx, &server, &realm).await;
            let token = &ctx.admin_token;
            let url = format!("{}/policies", base(&realm));

            let mine = server
                .post(&format!("{url}?managed_by=autharie"))
                .authorization_bearer(token)
                .json(&static_policy("agents-read", READ_ANY))
                .await;
            assert_eq!(mine.status_code(), 201, "{}", mine.text());
            let mine = mine.json::<Value>();
            assert_eq!(mine["managed_by"], json!("autharie"));
            let mine_id = mine["id"].as_str().expect("id").to_string();

            let by_hand = server
                .post(&url)
                .authorization_bearer(token)
                .json(&static_policy("by-hand", READ_ANY))
                .await;
            assert_eq!(by_hand.status_code(), 201, "{}", by_hand.text());
            let by_hand_id = by_hand.json::<Value>()["id"]
                .as_str()
                .expect("id")
                .to_string();
            assert!(by_hand.json::<Value>()["managed_by"].is_null());

            let scoped = server
                .get(&format!("{url}?managed_by=autharie"))
                .authorization_bearer(token)
                .await;
            let scoped = scoped.json::<Value>();
            assert_eq!(scoped.as_array().map(Vec::len), Some(1));
            assert_eq!(scoped[0]["name"], json!("agents-read"));
            let all = server.get(&url).authorization_bearer(token).await;
            assert_eq!(all.json::<Value>().as_array().map(Vec::len), Some(2));

            let foreign_delete = server
                .delete(&format!("{url}/{by_hand_id}?managed_by=autharie"))
                .authorization_bearer(token)
                .await;
            assert_eq!(
                foreign_delete.status_code(),
                404,
                "{}",
                foreign_delete.text()
            );
            let foreign_update = server
                .put(&format!("{url}/{by_hand_id}?managed_by=autharie"))
                .authorization_bearer(token)
                .json(&static_policy("by-hand", READ_ANY))
                .await;
            assert_eq!(
                foreign_update.status_code(),
                404,
                "{}",
                foreign_update.text()
            );
            let other_manager = server
                .get(&format!("{url}/{mine_id}?managed_by=someone-else"))
                .authorization_bearer(token)
                .await;
            assert_eq!(other_manager.status_code(), 404, "{}", other_manager.text());

            let own_delete = server
                .delete(&format!("{url}/{mine_id}?managed_by=autharie"))
                .authorization_bearer(token)
                .await;
            assert_eq!(own_delete.status_code(), 204, "{}", own_delete.text());

            let bad_label = server
                .get(&format!("{url}?managed_by=Not%20Valid"))
                .authorization_bearer(token)
                .await;
            assert_eq!(bad_label.status_code(), 422, "{}", bad_label.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn templates_are_managed_and_cannot_be_deleted_while_linked() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            setup_schema(ctx, &server, &realm).await;
            let token = &ctx.admin_token;
            let templates = format!("{}/templates", base(&realm));

            let created = server
                .post(&templates)
                .authorization_bearer(token)
                .json(&json!({
                    "name": "role-reads",
                    "source": "permit (principal in ?principal, action == App::Action::\"document:read\", resource);",
                }))
                .await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let template_id = created.json::<Value>()["id"]
                .as_str()
                .expect("id")
                .to_string();

            let fetched = server
                .get(&format!("{templates}/{template_id}"))
                .authorization_bearer(token)
                .await;
            assert_eq!(fetched.status_code(), 200, "{}", fetched.text());

            let linked = server
                .post(&format!("{}/policies", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({
                    "name": "editors-read",
                    "body": {
                        "type": "linked",
                        "template_id": template_id,
                        "links": { "?principal": { "type": "FerrisKey::Role", "id": "editors" } },
                    },
                }))
                .await;
            assert_eq!(linked.status_code(), 201, "{}", linked.text());
            assert_eq!(linked.json::<Value>()["origin"], json!("generated"));

            let in_use = server
                .delete(&format!("{templates}/{template_id}"))
                .authorization_bearer(token)
                .await;
            assert_eq!(in_use.status_code(), 409, "{}", in_use.text());

            let listed = server.get(&templates).authorization_bearer(token).await;
            assert_eq!(listed.json::<Value>().as_array().map(Vec::len), Some(1));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn reading_needs_view_authorization_and_writing_needs_manage_authorization() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            setup_schema(ctx, &server, &realm).await;

            let nobody = create_pep(ctx, &server, &realm, &[]).await;
            let viewer = create_pep(ctx, &server, &realm, &["view_authorization"]).await;
            let manager = create_pep(ctx, &server, &realm, &["manage_authorization"]).await;
            let policies = format!("{}/policies", base(&realm));
            let body = static_policy("p", READ_ANY);

            for (who, token, reads, writes) in [
                ("nobody", &nobody.token, false, false),
                ("viewer", &viewer.token, true, false),
                ("manager", &manager.token, true, true),
            ] {
                let read = server.get(&policies).authorization_bearer(token).await;
                assert_eq!(read.status_code() == 200, reads, "{who}: {}", read.text());
                if !reads {
                    assert_eq!(read.status_code(), 403, "{who}");
                }
                let validate = server
                    .post(&format!("{policies}/validate"))
                    .authorization_bearer(token)
                    .json(&json!({ "policies": [body] }))
                    .await;
                assert_eq!(
                    validate.status_code() == 200,
                    reads,
                    "{who}: {}",
                    validate.text()
                );
                let write = server
                    .post(&policies)
                    .authorization_bearer(token)
                    .json(&body)
                    .await;
                assert_eq!(
                    write.status_code() == 201,
                    writes,
                    "{who}: {}",
                    write.text()
                );
                if !writes {
                    assert_eq!(write.status_code(), 403, "{who}");
                }
                let state = server
                    .put(&format!("{}/state", base(&realm)))
                    .authorization_bearer(token)
                    .json(&json!({ "enabled": true }))
                    .await;
                assert_eq!(
                    state.status_code() == 200,
                    writes,
                    "{who}: {}",
                    state.text()
                );
            }

            let anonymous = server.get(&policies).await;
            assert_eq!(anonymous.status_code(), 401, "{}", anonymous.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_management_routes_are_absent_when_the_feature_is_off() {
        let ctx = ctx();
        rt().block_on(async {
            let realm = create_realm(ctx, &ctx.server()).await;
            let off = ctx.disabled_server();
            let response = off
                .get(&format!("{}/policies", base(&realm)))
                .authorization_bearer(&ctx.admin_token)
                .await;
            assert_eq!(response.status_code(), 404, "{}", response.text());
        });
    }

    const SUPPORT_POLICY: &str = r#"permit (
  principal in FerrisKey::Group::"support",
  action == FerrisKey::Action::"call_tool",
  resource is FerrisKey::Tool
) when {
  context.agent == "claude" &&
  resource has annotations &&
  resource.annotations has readOnlyHint &&
  resource.annotations.readOnlyHint &&
  context.time.weekday <= 5 &&
  context.time.hour >= 8 &&
  context.time.hour < 18
};"#;

    fn tool_call(groups: &[&str], tool: &str, read_only: bool, weekday: u32) -> Value {
        json!({
            "subject": {
                "type": "user",
                "id": "9b4c7a52-0000-4000-8000-000000000001",
                "properties": { "roles": ["dev"], "groups": groups },
            },
            "action": { "name": "call_tool" },
            "resource": {
                "type": "tool",
                "id": format!("files/{tool}"),
                "properties": {
                    "server": "files",
                    "tool": tool,
                    "annotations": { "readOnlyHint": read_only },
                },
            },
            "context": {
                "agent": "claude",
                "delegated": true,
                "arguments": { "path": "/tmp/report.txt" },
                "time": { "weekday": weekday, "hour": 10, "iso": "2026-10-22T10:00:00Z" },
            },
        })
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_agent_policy_validates_and_decides_tool_calls_for_the_users_it_names() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let token = &ctx.admin_token;

            let builtin = server
                .get(&format!("{}/schema/builtin", base(&realm)))
                .authorization_bearer(token)
                .await;
            assert_eq!(builtin.status_code(), 200, "{}", builtin.text());
            let source = builtin.json::<Value>()["source"]
                .as_str()
                .expect("source")
                .to_string();
            assert!(source.contains("entity Tool"), "{source}");

            let schema = server
                .put(&format!("{}/schema", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({ "source": "namespace App {}" }))
                .await;
            assert_eq!(schema.status_code(), 200, "{}", schema.text());

            let wrong = server
                .post(&format!("{}/policies/validate", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({ "policies": [static_policy(
                    "wrong",
                    "permit (principal is FerrisKey::Agent, action == FerrisKey::Action::\"call_tool\", resource is FerrisKey::Prompt);"
                )] }))
                .await;
            assert_eq!(wrong.json::<Value>()["valid"], json!(true), "a warning does not block");

            let typo = server
                .post(&format!("{}/policies/validate", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({ "policies": [static_policy(
                    "typo",
                    "permit (principal, action == FerrisKey::Action::\"call_tools\", resource);"
                )] }))
                .await;
            assert_eq!(typo.json::<Value>()["valid"], json!(false));

            let created = server
                .post(&format!("{}/policies?managed_by=autharie", base(&realm)))
                .authorization_bearer(token)
                .json(&static_policy("support-read-only", SUPPORT_POLICY))
                .await;
            assert_eq!(created.status_code(), 201, "{}", created.text());
            let enabled = server
                .put(&format!("{}/state", base(&realm)))
                .authorization_bearer(token)
                .json(&json!({ "enabled": true }))
                .await;
            assert_eq!(enabled.status_code(), 200, "{}", enabled.text());

            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let evaluate = |body: Value| {
                let server = &server;
                let pep = &pep;
                let url = format!("/realms/{}/authzen/access/v1/evaluation", realm.name);
                async move {
                    server
                        .post(&url)
                        .authorization_bearer(&pep.token)
                        .json(&body)
                        .await
                }
            };

            let allowed = evaluate(tool_call(&["support"], "read", true, 3)).await;
            assert_eq!(allowed.status_code(), 200, "{}", allowed.text());
            assert_eq!(allowed.json::<Value>()["decision"], json!(true));

            for (label, body) in [
                ("other group", tool_call(&["sales"], "read", true, 3)),
                ("not read only", tool_call(&["support"], "delete", false, 3)),
                ("weekend", tool_call(&["support"], "read", true, 6)),
            ] {
                let response = evaluate(body).await;
                assert_eq!(response.status_code(), 200, "{label}: {}", response.text());
                assert_eq!(response.json::<Value>()["decision"], json!(false), "{label}");
            }

            let batch = server
                .post(&format!(
                    "/realms/{}/authzen/access/v1/evaluations",
                    realm.name
                ))
                .authorization_bearer(&pep.token)
                .json(&json!({
                    "subject": tool_call(&["support"], "read", true, 3)["subject"],
                    "action": { "name": "call_tool" },
                    "context": tool_call(&["support"], "read", true, 3)["context"],
                    "evaluations": [
                        { "resource": tool_call(&["support"], "read", true, 3)["resource"] },
                        { "resource": tool_call(&["support"], "delete", false, 3)["resource"] },
                    ],
                }))
                .await;
            assert_eq!(batch.status_code(), 200, "{}", batch.text());
            let decisions: Vec<bool> = batch.json::<Value>()["evaluations"]
                .as_array()
                .expect("evaluations")
                .iter()
                .map(|item| item["decision"].as_bool().expect("decision"))
                .collect();
            assert_eq!(decisions, [true, false]);
        });
    }
}
