/// Integration tests for the AuthZEN 1.0 evaluation endpoint.
///
///   DATABASE_HOST=127.0.0.1 DATABASE_PORT=5432 \
///     cargo test -p ferriskey-api --test authzen_test -- --ignored
#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::PasswordHashingCost;
    use std::{
        env,
        sync::{Arc, Mutex, OnceLock},
    };

    use axum::Router;
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

    const MASTER: &str = "master";
    const ADMIN_PASSWORD: &str = "admin";

    const DOCUMENT_SCHEMA: &str = r#"namespace App {
    entity Document { classification: String };
    action "document:read" appliesTo { principal: [FerrisKey::User], resource: [Document] };
}"#;

    const REPORT_SCHEMA: &str = r#"namespace App {
    entity Document { classification: String, owner_team: String };
    action "document:update" appliesTo { principal: [FerrisKey::User], resource: [Document], context: { ip: String, time: String } };
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
        let schema = format!("authzen_{}", Uuid::new_v4().simple());
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

    struct Policy<'a> {
        name: &'a str,
        source: String,
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

    async fn seed_rules(ctx: &Ctx, realm: &Realm, schema: &str, policies: &[Policy<'_>]) {
        sqlx::query(
            "INSERT INTO authz_realm_states (realm_id, enabled, version) VALUES ($1, true, 1)
             ON CONFLICT (realm_id) DO UPDATE SET enabled = true, version = authz_realm_states.version + 1",
        )
        .bind(realm.id)
        .execute(&ctx.pool)
        .await
        .expect("enable authorization");
        sqlx::query(
            "INSERT INTO authz_schemas (id, realm_id, namespace, source) VALUES ($1, $2, 'App', $3)",
        )
        .bind(Uuid::new_v4())
        .bind(realm.id)
        .bind(schema)
        .execute(&ctx.pool)
        .await
        .expect("insert schema");
        for policy in policies {
            sqlx::query(
                "INSERT INTO authz_policies (id, realm_id, name, origin, enabled, source) VALUES ($1, $2, $3, 'custom', true, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(realm.id)
            .bind(policy.name)
            .bind(&policy.source)
            .execute(&ctx.pool)
            .await
            .expect("insert policy");
        }
    }

    async fn create_user(ctx: &Ctx, server: &TestServer, realm: &Realm) -> Uuid {
        let username = format!("u-{}", Uuid::new_v4().simple());
        let created = server
            .post(&format!("/realms/{}/users", realm.name))
            .authorization_bearer(&ctx.admin_token)
            .json(&json!({
                "username": username,
                "firstname": "Test",
                "lastname": "User",
                "email": format!("{username}@ferriskey.test"),
                "email_verified": true,
            }))
            .await;
        assert_eq!(created.status_code(), 200, "{}", created.text());
        let (id,): (Uuid,) =
            sqlx::query_as("SELECT id FROM users WHERE username = $1 AND realm_id = $2")
                .bind(&username)
                .bind(realm.id)
                .fetch_one(&ctx.pool)
                .await
                .expect("user id");
        id
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

    fn evaluation_path(realm: &str) -> String {
        format!("/realms/{realm}/authzen/access/v1/evaluation")
    }

    async fn evaluate(
        server: &TestServer,
        realm: &str,
        token: Option<&str>,
        body: &Value,
        request_id: Option<&str>,
    ) -> TestResponse {
        let mut request = server.post(&evaluation_path(realm));
        if let Some(token) = token {
            request = request.authorization_bearer(token);
        }
        if let Some(request_id) = request_id {
            request = request.add_header("x-request-id", request_id);
        }
        request.json(body).await
    }

    fn read_request(user: Uuid, classification: &str) -> Value {
        json!({
            "subject": { "type": "FerrisKey::User", "id": user.to_string() },
            "action": { "name": "document:read" },
            "resource": {
                "type": "App::Document",
                "id": "42",
                "properties": { "classification": classification },
            },
        })
    }

    fn decision(response: &TestResponse) -> bool {
        response.json::<Value>()["decision"]
            .as_bool()
            .expect("decision")
    }

    fn assert_no_decision(response: &TestResponse) {
        assert!(
            response.json::<Value>().get("decision").is_none(),
            "an error carried a decision: {}",
            response.text()
        );
    }

    fn role_policy(role: Uuid) -> Vec<Policy<'static>> {
        vec![Policy {
            name: "readers-can-read",
            source: format!(
                "permit (principal in FerrisKey::Role::\"{role}\", action == App::Action::\"document:read\", resource is App::Document);"
            ),
        }]
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_user_holding_the_role_is_permitted_and_one_without_is_not() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;

            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;
            let outsider = create_user(ctx, &server, &realm).await;

            let permitted = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(holder, "public"),
                None,
            )
            .await;
            assert_eq!(permitted.status_code(), 200, "{}", permitted.text());
            assert!(decision(&permitted));

            let refused = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(outsider, "public"),
                None,
            )
            .await;
            assert_eq!(refused.status_code(), 200, "{}", refused.text());
            assert!(!decision(&refused));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_resource_property_drives_the_decision() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(
                ctx,
                &realm,
                DOCUMENT_SCHEMA,
                &[Policy {
                    name: "public-documents",
                    source: "permit (principal, action == App::Action::\"document:read\", resource is App::Document) when { resource.classification == \"public\" };".to_string(),
                }],
            )
            .await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &realm).await;

            let public = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(user, "public"),
                None,
            )
            .await;
            assert_eq!(public.status_code(), 200, "{}", public.text());
            assert!(decision(&public));

            let secret = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(user, "secret"),
                None,
            )
            .await;
            assert_eq!(secret.status_code(), 200, "{}", secret.text());
            assert!(!decision(&secret));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn only_a_service_account_with_the_permission_may_evaluate() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &[]).await;
            let user = create_user(ctx, &server, &realm).await;
            let body = read_request(user, "public");

            let without = create_pep(ctx, &server, &realm, &[]).await;
            let response = evaluate(&server, &realm.name, Some(&without.token), &body, None).await;
            assert_eq!(response.status_code(), 403, "{}", response.text());
            assert_no_decision(&response);

            let anonymous = evaluate(&server, &realm.name, None, &body, None).await;
            assert_eq!(anonymous.status_code(), 401, "{}", anonymous.text());
            assert_no_decision(&anonymous);

            let human = evaluate(&server, &realm.name, Some(&ctx.admin_token), &body, None).await;
            assert_eq!(human.status_code(), 403, "{}", human.text());
            assert_no_decision(&human);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_pep_of_one_realm_cannot_evaluate_in_another() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let home = create_realm(ctx, &server).await;
            let other = create_realm(ctx, &server).await;
            seed_rules(ctx, &home, DOCUMENT_SCHEMA, &[]).await;
            seed_rules(ctx, &other, DOCUMENT_SCHEMA, &[]).await;
            let pep = create_pep(ctx, &server, &home, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &other).await;

            let response = evaluate(
                &server,
                &other.name,
                Some(&pep.token),
                &read_request(user, "public"),
                None,
            )
            .await;
            assert_eq!(response.status_code(), 404, "{}", response.text());
            assert_no_decision(&response);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_subject_from_another_realm_is_refused() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let other = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let stranger = create_user(ctx, &server, &other).await;
            grant(ctx, stranger, role).await;

            let response = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(stranger, "public"),
                None,
            )
            .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert!(!decision(&response));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn subject_properties_cannot_turn_a_deny_into_an_allow() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let outsider = create_user(ctx, &server, &realm).await;

            let mut body = read_request(outsider, "public");
            body["subject"]["properties"] = json!({
                "roles": [role.to_string()],
                "role_ids": [role.to_string()],
                "attributes": { "role": role.to_string() },
                "parents": [{ "type": "FerrisKey::Role", "id": role.to_string() }],
            });

            let response = evaluate(&server, &realm.name, Some(&pep.token), &body, None).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert!(!decision(&response));
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn malformed_requests_are_rejected_without_a_decision() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &[]).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &realm).await;

            let mut unknown_action = read_request(user, "public");
            unknown_action["action"]["name"] = json!("document:burn");

            let mut unknown_resource = read_request(user, "public");
            unknown_resource["resource"]["type"] = json!("App::Folder");

            let mut wrong_attribute = read_request(user, "public");
            wrong_attribute["resource"]["properties"]["classification"] = json!(5);

            let mut missing_type = read_request(user, "public");
            missing_type["subject"]
                .as_object_mut()
                .expect("subject object")
                .remove("type");

            let mut missing_action = read_request(user, "public");
            missing_action
                .as_object_mut()
                .expect("request object")
                .remove("action");

            for (label, body) in [
                ("unknown action", unknown_action),
                ("unknown resource type", unknown_resource),
                ("wrong attribute type", wrong_attribute),
                ("missing subject type", missing_type),
                ("missing action", missing_action),
            ] {
                let response = evaluate(&server, &realm.name, Some(&pep.token), &body, None).await;
                assert_eq!(response.status_code(), 400, "{label}: {}", response.text());
                assert_no_decision(&response);
            }
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_realm_without_authorization_answers_not_found() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &realm).await;

            let response = evaluate(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(user, "public"),
                None,
            )
            .await;
            assert_eq!(response.status_code(), 404, "{}", response.text());
            assert_no_decision(&response);
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_routes_are_absent_when_the_feature_is_off() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &[]).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &realm).await;

            let off = ctx.disabled_server();
            let response = evaluate(
                &off,
                &realm.name,
                Some(&pep.token),
                &read_request(user, "public"),
                None,
            )
            .await;
            assert_eq!(response.status_code(), 404, "{}", response.text());

            let metadata = off
                .get(&format!(
                    "/.well-known/authzen-configuration/realms/{MASTER}/authzen"
                ))
                .await;
            assert_eq!(metadata.status_code(), 404, "{}", metadata.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_request_id_is_echoed_and_reasons_follow_view_authorization() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let plain = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let viewer = create_pep(
                ctx,
                &server,
                &realm,
                &["evaluate_authorization", "view_authorization"],
            )
            .await;
            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;
            let body = read_request(holder, "public");

            let echoed = evaluate(
                &server,
                &realm.name,
                Some(&plain.token),
                &body,
                Some("req-7f1c"),
            )
            .await;
            assert_eq!(echoed.status_code(), 200, "{}", echoed.text());
            assert_eq!(echoed.header("x-request-id"), "req-7f1c");
            assert!(echoed.json::<Value>().get("context").is_none());

            let mut malformed = body.clone();
            malformed["action"] = json!({});
            let rejected = evaluate(
                &server,
                &realm.name,
                Some(&plain.token),
                &malformed,
                Some("req-400"),
            )
            .await;
            assert_eq!(rejected.status_code(), 400, "{}", rejected.text());
            assert_eq!(rejected.header("x-request-id"), "req-400");

            let unauthenticated =
                evaluate(&server, &realm.name, None, &body, Some("req-401")).await;
            assert_eq!(unauthenticated.status_code(), 401);
            assert_eq!(unauthenticated.header("x-request-id"), "req-401");

            let absent = evaluate(&server, &realm.name, Some(&plain.token), &body, None).await;
            assert!(absent.maybe_header("x-request-id").is_none());

            let explained = evaluate(&server, &realm.name, Some(&viewer.token), &body, None).await;
            assert_eq!(explained.status_code(), 200, "{}", explained.text());
            let explained = explained.json::<Value>();
            assert_eq!(explained["decision"], json!(true));
            assert_eq!(
                explained["context"]["reason_admin"]["policies"],
                json!(["readers-can-read"])
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_metadata_document_names_the_decision_point() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();

            let response = server
                .get(&format!(
                    "/.well-known/authzen-configuration/realms/{MASTER}/authzen"
                ))
                .await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            let document = response.json::<Value>();
            let pdp = document["policy_decision_point"]
                .as_str()
                .expect("policy_decision_point");
            assert!(pdp.ends_with(&format!("/realms/{MASTER}/authzen")), "{pdp}");
            assert!(pdp.starts_with("http"), "{pdp}");
            assert_eq!(
                document["access_evaluation_endpoint"],
                json!(format!("{pdp}/access/v1/evaluation"))
            );
            assert_eq!(
                document["access_evaluations_endpoint"],
                json!(format!("{pdp}/access/v1/evaluations"))
            );

            let unknown = server
                .get("/.well-known/authzen-configuration/realms/no-such-realm/authzen")
                .await;
            assert_eq!(unknown.status_code(), 404, "{}", unknown.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn the_specification_example_is_accepted() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(
                ctx,
                &realm,
                REPORT_SCHEMA,
                &[Policy {
                    name: "editors-can-update",
                    source: format!(
                        "permit (principal in FerrisKey::Role::\"{role}\", action == App::Action::\"document:update\", resource is App::Document) when {{ resource.owner_team == \"eng\" }};"
                    ),
                }],
            )
            .await;
            let pep = create_pep(
                ctx,
                &server,
                &realm,
                &["evaluate_authorization", "view_authorization"],
            )
            .await;
            let editor = create_user(ctx, &server, &realm).await;
            grant(ctx, editor, role).await;

            let body = json!({
                "subject": { "type": "FerrisKey::User", "id": editor.to_string() },
                "action": { "name": "document:update" },
                "resource": {
                    "type": "App::Document",
                    "id": "42",
                    "properties": { "classification": "internal", "owner_team": "eng" },
                },
                "context": { "ip": "10.0.3.4", "time": "2026-09-24T10:00:00Z" },
            });
            let response = evaluate(&server, &realm.name, Some(&pep.token), &body, Some("7f1c")).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert_eq!(response.header("x-request-id"), "7f1c");
            assert_eq!(
                response.json::<Value>(),
                json!({
                    "decision": true,
                    "context": { "reason_admin": { "policies": ["editors-can-update"] } },
                })
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn an_oversized_body_is_refused() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &[]).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let user = create_user(ctx, &server, &realm).await;

            let mut body = read_request(user, "public");
            body["context"] = json!({ "padding": "x".repeat(70 * 1024) });
            let response = evaluate(&server, &realm.name, Some(&pep.token), &body, None).await;
            assert_eq!(response.status_code(), 400, "{}", response.text());
            assert_no_decision(&response);
        });
    }

    fn evaluations_path(realm: &str) -> String {
        format!("/realms/{realm}/authzen/access/v1/evaluations")
    }

    fn batch_body(users: &[Uuid], semantic: Option<&str>) -> Value {
        let mut body = json!({
            "action": { "name": "document:read" },
            "resource": {
                "type": "App::Document",
                "id": "42",
                "properties": { "classification": "public" },
            },
            "evaluations": users
                .iter()
                .map(|user| json!({ "subject": { "type": "FerrisKey::User", "id": user.to_string() } }))
                .collect::<Vec<_>>(),
        });
        if let Some(semantic) = semantic {
            body["options"] = json!({ "evaluations_semantic": semantic });
        }
        body
    }

    async fn evaluations(
        server: &TestServer,
        realm: &str,
        token: Option<&str>,
        body: &Value,
    ) -> TestResponse {
        let mut request = server.post(&evaluations_path(realm));
        if let Some(token) = token {
            request = request.authorization_bearer(token);
        }
        request.json(body).await
    }

    fn decisions(response: &TestResponse) -> Vec<bool> {
        response.json::<Value>()["evaluations"]
            .as_array()
            .expect("evaluations")
            .iter()
            .map(|item| item["decision"].as_bool().expect("decision"))
            .collect()
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_batch_applies_the_defaults_and_answers_each_item_in_order() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(
                ctx,
                &server,
                &realm,
                &["evaluate_authorization", "view_authorization"],
            )
            .await;
            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;
            let outsider = create_user(ctx, &server, &realm).await;

            let response = evaluations(
                &server,
                &realm.name,
                Some(&pep.token),
                &batch_body(&[holder, outsider, holder], None),
            )
            .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert_eq!(decisions(&response), [true, false, true]);
            assert_eq!(
                response.json::<Value>()["evaluations"][0]["context"]["reason_admin"]["policies"],
                json!(["readers-can-read"])
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_batch_stops_where_its_semantic_says() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;
            let outsider = create_user(ctx, &server, &realm).await;
            let users = [holder, outsider, holder];

            for (semantic, expected) in [
                ("execute_all", vec![true, false, true]),
                ("deny_on_first_deny", vec![true, false]),
                ("permit_on_first_permit", vec![true]),
            ] {
                let response = evaluations(
                    &server,
                    &realm.name,
                    Some(&pep.token),
                    &batch_body(&users, Some(semantic)),
                )
                .await;
                assert_eq!(
                    response.status_code(),
                    200,
                    "{semantic}: {}",
                    response.text()
                );
                assert_eq!(decisions(&response), expected, "{semantic}");
            }

            let unknown = evaluations(
                &server,
                &realm.name,
                Some(&pep.token),
                &batch_body(&users, Some("first_wins")),
            )
            .await;
            assert_eq!(unknown.status_code(), 400, "{}", unknown.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_batch_item_the_schema_rejects_is_a_deny_with_its_error() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;

            let mut body = batch_body(&[holder, holder], None);
            body["evaluations"][1]["action"] = json!({ "name": "document:burn" });
            let response = evaluations(&server, &realm.name, Some(&pep.token), &body).await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert_eq!(decisions(&response), [true, false]);
            assert_eq!(
                response.json::<Value>()["evaluations"][1]["context"]["error"]["status"],
                json!(400)
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_malformed_batch_and_a_caller_without_the_permission_get_no_decisions() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &[]).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let without = create_pep(ctx, &server, &realm, &[]).await;
            let user = create_user(ctx, &server, &realm).await;
            let body = batch_body(&[user], None);

            let missing_subject = json!({
                "action": { "name": "document:read" },
                "resource": { "type": "App::Document", "id": "1" },
                "evaluations": [{}],
            });
            let empty = json!({ "evaluations": [] });
            for (label, payload) in [("missing subject", missing_subject), ("empty", empty)] {
                let response = evaluations(&server, &realm.name, Some(&pep.token), &payload).await;
                assert_eq!(response.status_code(), 400, "{label}: {}", response.text());
                assert_no_decision(&response);
            }

            let forbidden = evaluations(&server, &realm.name, Some(&without.token), &body).await;
            assert_eq!(forbidden.status_code(), 403, "{}", forbidden.text());
            let anonymous = evaluations(&server, &realm.name, None, &body).await;
            assert_eq!(anonymous.status_code(), 401, "{}", anonymous.text());

            let off = ctx.disabled_server();
            let absent = evaluations(&off, &realm.name, Some(&pep.token), &body).await;
            assert_eq!(absent.status_code(), 404, "{}", absent.text());
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL"]
    fn a_request_without_evaluations_is_a_single_evaluation() {
        let ctx = ctx();
        rt().block_on(async {
            let server = ctx.server();
            let realm = create_realm(ctx, &server).await;
            let role = create_role(ctx, &server, &realm, &[]).await;
            seed_rules(ctx, &realm, DOCUMENT_SCHEMA, &role_policy(role)).await;
            let pep = create_pep(ctx, &server, &realm, &["evaluate_authorization"]).await;
            let holder = create_user(ctx, &server, &realm).await;
            grant(ctx, holder, role).await;

            let response = evaluations(
                &server,
                &realm.name,
                Some(&pep.token),
                &read_request(holder, "public"),
            )
            .await;

            assert_eq!(response.status_code(), 200, "{}", response.text());
            assert_eq!(response.json::<Value>(), json!({ "decision": true }));
        });
    }
}
