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
    use sqlx::{Executor, PgPool, Row};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";

    const MASTER_REALM: &str = "master";
    const TENANT_A: &str = "tenant-a";
    const TENANT_B: &str = "tenant-b";

    const VICTIM: &str = "victim";

    const TENANT_B_DELIVERY_MARKER: &str = "tenant-b-delivery-witness";
    const TENANT_B_TEMPLATE_MARKER: &str = "tenant-b-template-witness";
    const TENANT_B_THEME_MARKER: &str = "tenant-b-theme-witness";
    const TENANT_B_LAYOUT_MARKER: &str = "tenant-b-layout-witness";
    const POISON_MARKER: &str = "tenant-a-poison-attempt";

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: PgPool,
        admin_token: String,
        victim_id: Uuid,
        tenant_a_webhook: Uuid,
        tenant_b_webhook: Uuid,
        tenant_b_readable_delivery: Uuid,
        tenant_b_replayable_delivery: Uuid,
        tenant_b_survivor_delivery: Uuid,
        tenant_b_template: Uuid,
        tenant_b_theme: Uuid,
        tenant_b_layout: Uuid,
        tenant_a_layout: Uuid,
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

        let schema = format!("observability_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let tenant_a_id = realm_id(&pool, TENANT_A).await;
        let tenant_b_id = realm_id(&pool, TENANT_B).await;

        let victim_id = create_user(&server, &admin_token, TENANT_B, VICTIM).await;

        let tenant_a_webhook = create_webhook(&server, &admin_token, TENANT_A).await;
        let tenant_b_webhook = create_webhook(&server, &admin_token, TENANT_B).await;

        let tenant_b_readable_delivery = seed_delivery(
            &pool,
            tenant_b_id,
            tenant_b_webhook,
            "failed",
            TENANT_B_DELIVERY_MARKER,
        )
        .await;
        let tenant_b_replayable_delivery = seed_delivery(
            &pool,
            tenant_b_id,
            tenant_b_webhook,
            "failed",
            TENANT_B_DELIVERY_MARKER,
        )
        .await;
        let tenant_b_survivor_delivery = seed_delivery(
            &pool,
            tenant_b_id,
            tenant_b_webhook,
            "failed",
            TENANT_B_DELIVERY_MARKER,
        )
        .await;

        let tenant_b_template =
            create_template(&server, &admin_token, TENANT_B, TENANT_B_TEMPLATE_MARKER).await;
        let tenant_b_layout =
            create_layout(&server, &admin_token, TENANT_B, TENANT_B_LAYOUT_MARKER).await;
        let tenant_a_layout =
            create_layout(&server, &admin_token, TENANT_A, "tenant-a-layout").await;
        let tenant_b_theme = create_theme(
            &server,
            &admin_token,
            TENANT_B,
            TENANT_B_THEME_MARKER,
            Some(tenant_b_layout),
        )
        .await;

        assert_ne!(
            tenant_a_id, tenant_b_id,
            "the two tenants must be distinct realms for any of this to mean anything"
        );

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            admin_token,
            victim_id,
            tenant_a_webhook,
            tenant_b_webhook,
            tenant_b_readable_delivery,
            tenant_b_replayable_delivery,
            tenant_b_survivor_delivery,
            tenant_b_template,
            tenant_b_theme,
            tenant_b_layout,
            tenant_a_layout,
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

    async fn realm_id(pool: &PgPool, name: &str) -> Uuid {
        sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(name)
            .fetch_one(pool)
            .await
            .unwrap_or_else(|e| panic!("fetch realm {name}: {e}"))
            .get("id")
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

    async fn create_webhook(server: &TestServer, admin_token: &str, realm: &str) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/webhooks", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": format!("{realm}-hook"),
                "endpoint": "https://example.test/hook",
                "subscribers": ["user.created"],
            }))
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "creating a webhook in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created webhook: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("webhook id {raw} is not a uuid: {e}"))
    }

    async fn seed_delivery(
        pool: &PgPool,
        realm: Uuid,
        webhook: Uuid,
        status: &str,
        marker: &str,
    ) -> Uuid {
        let delivery_id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO webhook_deliveries \
             (id, realm_id, webhook_id, event, resource_id, payload, status, attempt_count, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6::jsonb, $7, 1, NOW(), NOW())",
        )
        .bind(delivery_id)
        .bind(realm)
        .bind(webhook)
        .bind("user.created")
        .bind(Uuid::new_v4())
        .bind(format!(r#"{{"witness":"{marker}"}}"#))
        .bind(status)
        .execute(pool)
        .await
        .unwrap_or_else(|e| panic!("insert delivery for webhook {webhook}: {e}"));

        delivery_id
    }

    async fn create_template(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
    ) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/email-templates", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "email_type": "email_verification",
                "structure": { "children": [] },
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating an email template in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created template: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("template id {raw} is not a uuid: {e}"))
    }

    async fn create_layout(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
    ) -> Uuid {
        let response = server
            .post(&format!("/realms/{}/portal-layouts", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&json!({
                "name": name,
                "tree": [],
            }))
            .await;

        assert_eq!(
            response.status_code(),
            201,
            "creating a portal layout in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created layout: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("layout id {raw} is not a uuid: {e}"))
    }

    async fn create_theme(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
        layout_id: Option<Uuid>,
    ) -> Uuid {
        let response = create_theme_response(server, admin_token, realm, name, layout_id).await;

        assert_eq!(
            response.status_code(),
            201,
            "creating a portal theme in {realm} failed: {}",
            response.text()
        );

        let body: Value = response.json();
        let raw = body["data"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("no id for the created theme: {body}"));

        Uuid::parse_str(raw).unwrap_or_else(|e| panic!("theme id {raw} is not a uuid: {e}"))
    }

    async fn create_theme_response(
        server: &TestServer,
        admin_token: &str,
        realm: &str,
        name: &str,
        layout_id: Option<Uuid>,
    ) -> TestResponse {
        let mut payload = json!({ "name": name });
        if let Some(layout_id) = layout_id {
            payload["layout_id"] = json!(layout_id.to_string());
        }

        server
            .post(&format!("/realms/{}/portal/themes", realm))
            .add_header("Authorization", auth_header(admin_token))
            .json(&payload)
            .await
    }

    async fn security_events(server: &TestServer, realm: &str) -> TestResponse {
        server
            .get(&format!("/realms/{}/seawatch/v1/security-events", realm))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn get_delivery(
        server: &TestServer,
        realm: &str,
        webhook: Uuid,
        delivery: Uuid,
    ) -> TestResponse {
        server
            .get(&format!(
                "/realms/{}/webhooks/{}/deliveries/{}",
                realm, webhook, delivery
            ))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn retry_delivery(
        server: &TestServer,
        realm: &str,
        webhook: Uuid,
        delivery: Uuid,
    ) -> TestResponse {
        server
            .post(&format!(
                "/realms/{}/webhooks/{}/deliveries/{}/retry",
                realm, webhook, delivery
            ))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn delivery_status(delivery: Uuid) -> String {
        sqlx::query("SELECT status FROM webhook_deliveries WHERE id = $1")
            .bind(delivery)
            .fetch_one(&ctx().pool)
            .await
            .expect("query delivery status")
            .get("status")
    }

    async fn get_template(server: &TestServer, realm: &str, template: Uuid) -> TestResponse {
        server
            .get(&format!("/realms/{}/email-templates/{}", realm, template))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn update_template(
        server: &TestServer,
        realm: &str,
        template: Uuid,
        name: &str,
    ) -> TestResponse {
        server
            .put(&format!("/realms/{}/email-templates/{}", realm, template))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .json(&json!({
                "name": name,
                "structure": { "children": [] },
            }))
            .await
    }

    async fn get_theme(server: &TestServer, realm: &str, theme: Uuid) -> TestResponse {
        server
            .get(&format!("/realms/{}/portal/themes/{}", realm, theme))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    async fn update_theme(
        server: &TestServer,
        realm: &str,
        theme: Uuid,
        name: &str,
    ) -> TestResponse {
        server
            .put(&format!("/realms/{}/portal/themes/{}", realm, theme))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .json(&json!({ "name": name }))
            .await
    }

    async fn theme_name(theme: Uuid) -> String {
        sqlx::query("SELECT name FROM portal_themes WHERE id = $1")
            .bind(theme)
            .fetch_one(&ctx().pool)
            .await
            .expect("query theme name")
            .get("name")
    }

    async fn get_layout(server: &TestServer, realm: &str, layout: Uuid) -> TestResponse {
        server
            .get(&format!("/realms/{}/portal-layouts/{}", realm, layout))
            .add_header("Authorization", auth_header(&ctx().admin_token))
            .await
    }

    fn assert_status(response: &TestResponse, expected: u16, what: &str) -> String {
        let body = response.text();

        assert_eq!(
            response.status_code(),
            expected,
            "{what}: expected {expected}, got {} with body {body}",
            response.status_code()
        );

        body
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn reading_the_security_journal_of_another_realm_leaks_nothing() {
        rt().block_on(async {
            let server = make_server();
            let victim = ctx().victim_id.to_string();

            let witness = security_events(&server, TENANT_B).await;
            let witness_body =
                assert_status(&witness, 200, "reading the tenant-b journal from tenant-b");
            assert!(
                witness_body.contains(&victim),
                "the tenant-b journal does not mention the seeded user, so the refusal below \
                 would prove nothing: {witness_body}"
            );

            let refused = security_events(&server, TENANT_A).await;
            let refused_body =
                assert_status(&refused, 200, "reading the tenant-b journal from tenant-a");

            assert_body_free_of(
                &refused_body,
                &[victim.as_str()],
                "the tenant-a security journal",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn reading_a_webhook_delivery_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let delivery = ctx().tenant_b_readable_delivery;

            let accepted = get_delivery(&server, TENANT_B, ctx().tenant_b_webhook, delivery).await;
            let accepted_body =
                assert_status(&accepted, 200, "reading a tenant-b delivery from tenant-b");
            assert!(
                accepted_body.contains(TENANT_B_DELIVERY_MARKER),
                "the delivery read from its own realm does not carry its payload, so the refusals \
                 below would be satisfied by an empty body: {accepted_body}"
            );

            let through_foreign_webhook =
                get_delivery(&server, TENANT_A, ctx().tenant_b_webhook, delivery).await;
            let foreign_body = assert_status(
                &through_foreign_webhook,
                404,
                "reading a tenant-b delivery through the tenant-a url",
            );
            assert_body_free_of(
                &foreign_body,
                &[TENANT_B_DELIVERY_MARKER],
                "the refused cross-realm delivery read",
            );

            let through_own_webhook =
                get_delivery(&server, TENANT_A, ctx().tenant_a_webhook, delivery).await;
            let own_body = assert_status(
                &through_own_webhook,
                404,
                "reading a tenant-b delivery through a tenant-a webhook",
            );
            assert_body_free_of(
                &own_body,
                &[TENANT_B_DELIVERY_MARKER],
                "the refused delivery read through a local webhook",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn replaying_a_webhook_delivery_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let accepted = retry_delivery(
                &server,
                TENANT_B,
                ctx().tenant_b_webhook,
                ctx().tenant_b_replayable_delivery,
            )
            .await;
            assert_status(
                &accepted,
                202,
                "replaying a tenant-b delivery from tenant-b must be possible, or the refusal \
                 below proves nothing",
            );
            assert_eq!(
                delivery_status(ctx().tenant_b_replayable_delivery).await,
                "pending",
                "the legitimate replay did not requeue the delivery"
            );

            let survivor = ctx().tenant_b_survivor_delivery;
            assert_eq!(
                delivery_status(survivor).await,
                "failed",
                "the survivor delivery must start out terminal"
            );

            let refused = retry_delivery(&server, TENANT_A, ctx().tenant_b_webhook, survivor).await;
            assert_status(
                &refused,
                404,
                "replaying a tenant-b delivery through the tenant-a url",
            );

            assert_eq!(
                delivery_status(survivor).await,
                "failed",
                "the refused replay still requeued a delivery of another realm"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn reading_an_email_template_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let template = ctx().tenant_b_template;

            let accepted = get_template(&server, TENANT_B, template).await;
            let accepted_body =
                assert_status(&accepted, 200, "reading a tenant-b template from tenant-b");
            assert!(
                accepted_body.contains(TENANT_B_TEMPLATE_MARKER),
                "the template read from its own realm does not carry its name, so the refusal \
                 below would be satisfied by an empty body: {accepted_body}"
            );

            let refused = get_template(&server, TENANT_A, template).await;
            let refused_body = assert_status(
                &refused,
                404,
                "reading a tenant-b template from the tenant-a url",
            );
            assert_body_free_of(
                &refused_body,
                &[TENANT_B_TEMPLATE_MARKER],
                "the refused cross-realm template read",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn updating_an_email_template_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let template = ctx().tenant_b_template;

            let accepted =
                update_template(&server, TENANT_B, template, TENANT_B_TEMPLATE_MARKER).await;
            assert_status(
                &accepted,
                200,
                "updating a tenant-b template from tenant-b must be possible, or the refusal \
                 below proves nothing",
            );

            let refused = update_template(&server, TENANT_A, template, POISON_MARKER).await;
            assert_status(
                &refused,
                404,
                "updating a tenant-b template from the tenant-a url",
            );

            let survivor = get_template(&server, TENANT_B, template).await;
            let survivor_body =
                assert_status(&survivor, 200, "re-reading the template from its own realm");
            assert!(
                survivor_body.contains(TENANT_B_TEMPLATE_MARKER),
                "the template lost its own name after the refused cross-realm update: \
                 {survivor_body}"
            );
            assert_body_free_of(
                &survivor_body,
                &[POISON_MARKER],
                "the template after a refused cross-realm update",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn reading_a_portal_theme_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let theme = ctx().tenant_b_theme;

            let accepted = get_theme(&server, TENANT_B, theme).await;
            let accepted_body =
                assert_status(&accepted, 200, "reading a tenant-b theme from tenant-b");
            assert!(
                accepted_body.contains(TENANT_B_THEME_MARKER),
                "the theme read from its own realm does not carry its name, so the refusal below \
                 would be satisfied by an empty body: {accepted_body}"
            );

            let refused = get_theme(&server, TENANT_A, theme).await;
            let refused_body = assert_status(
                &refused,
                404,
                "reading a tenant-b theme from the tenant-a url",
            );
            assert_body_free_of(
                &refused_body,
                &[TENANT_B_THEME_MARKER],
                "the refused cross-realm theme read",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn updating_a_portal_theme_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();
            let theme = ctx().tenant_b_theme;

            let accepted = update_theme(&server, TENANT_B, theme, TENANT_B_THEME_MARKER).await;
            assert_status(
                &accepted,
                200,
                "updating a tenant-b theme from tenant-b must be possible, or the refusal below \
                 proves nothing",
            );

            let refused = update_theme(&server, TENANT_A, theme, POISON_MARKER).await;
            assert_status(
                &refused,
                404,
                "updating a tenant-b theme from the tenant-a url",
            );

            assert_eq!(
                theme_name(theme).await,
                TENANT_B_THEME_MARKER,
                "the refused cross-realm update still renamed a theme of another realm"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test observability_cross_realm_test -- --ignored"]
    fn binding_a_portal_theme_to_a_layout_of_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let accepted = create_theme_response(
                &server,
                &ctx().admin_token,
                TENANT_A,
                "tenant-a-own-layout",
                Some(ctx().tenant_a_layout),
            )
            .await;
            assert_status(
                &accepted,
                201,
                "a tenant-a theme naming a tenant-a layout must be accepted, or the refusal below \
                 proves nothing",
            );

            let refused = create_theme_response(
                &server,
                &ctx().admin_token,
                TENANT_A,
                POISON_MARKER,
                Some(ctx().tenant_b_layout),
            )
            .await;
            assert_status(&refused, 404, "a tenant-a theme naming a tenant-b layout");

            let stored: Option<Uuid> =
                sqlx::query("SELECT layout_id FROM portal_themes WHERE name = $1")
                    .bind(POISON_MARKER)
                    .fetch_optional(&ctx().pool)
                    .await
                    .expect("query the refused theme")
                    .and_then(|row| row.get("layout_id"));
            assert!(
                stored.is_none(),
                "the refused create still stored a theme pointing at a tenant-b layout: {stored:?}"
            );

            let survivor = get_layout(&server, TENANT_B, ctx().tenant_b_layout).await;
            let survivor_body = assert_status(
                &survivor,
                200,
                "the tenant-b layout must still be readable from its own realm",
            );
            assert!(
                survivor_body.contains(TENANT_B_LAYOUT_MARKER),
                "the tenant-b layout lost its name after the refused cross-realm binding: \
                 {survivor_body}"
            );
        });
    }
}
