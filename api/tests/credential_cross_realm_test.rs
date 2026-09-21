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
    use sqlx::{Executor, Row};
    use uuid::Uuid;

    const WEBAPP_URL: &str = "http://localhost:5555";
    const SEEDED_CLIENT_ID: &str = "ferriskey-admin";
    const RECOVERY_CODE_FORMAT: &str = "b32-split-4";

    struct SharedContext {
        app: std::sync::Mutex<Router>,
        pool: sqlx::PgPool,
        realm_name: String,
        admin_user_id: Uuid,
        bystander_user_id: Uuid,
        neighbour_realm_name: String,
        neighbour_user_id: Uuid,
        neighbour_credential_id: Uuid,
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

        let schema = format!("credential_cross_realm_test_{}", Uuid::new_v4().simple());

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

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());

        service
            .initialize_application(StartupConfig {
                webapp_url: WEBAPP_URL.to_string(),
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: SEEDED_CLIENT_ID.to_string(),
            })
            .await
            .expect("initialize application");

        let realm_id: Uuid = sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(&realm_name)
            .fetch_one(&pool)
            .await
            .expect("fetch master realm")
            .get("id");

        let admin_user_id: Uuid =
            sqlx::query("SELECT id FROM users WHERE realm_id = $1 AND username = $2")
                .bind(realm_id)
                .bind("admin")
                .fetch_one(&pool)
                .await
                .expect("fetch admin user")
                .get("id");

        let bystander_user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(bystander_user_id)
        .bind(realm_id)
        .bind("bystander")
        .bind("By")
        .bind("Stander")
        .bind("bystander@test.local")
        .execute(&pool)
        .await
        .expect("insert bystander user");

        let neighbour_realm_id = Uuid::new_v4();
        let neighbour_realm_name = format!("neighbour-{}", Uuid::new_v4().simple());
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(neighbour_realm_id)
        .bind(&neighbour_realm_name)
        .execute(&pool)
        .await
        .expect("insert neighbour realm");

        let neighbour_user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(neighbour_user_id)
        .bind(neighbour_realm_id)
        .bind("victim")
        .bind("Victim")
        .bind("User")
        .bind("victim@neighbour.local")
        .execute(&pool)
        .await
        .expect("insert neighbour user");

        let neighbour_credential_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data) VALUES ($1, $2, $3, $4, $5::jsonb)",
        )
        .bind(neighbour_credential_id)
        .bind("totp")
        .bind(neighbour_user_id)
        .bind("seed-secret")
        .bind("{}")
        .execute(&pool)
        .await
        .expect("insert neighbour credential");

        let args = Arc::new(Args::default());
        let state = AppState::new(args, service);
        let app = router(state).expect("build router");

        SharedContext {
            app: std::sync::Mutex::new(app),
            pool,
            realm_name,
            admin_user_id,
            bystander_user_id,
            neighbour_realm_name,
            neighbour_user_id,
            neighbour_credential_id,
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

    fn realm() -> &'static str {
        ctx().realm_name.as_str()
    }

    fn neighbour_realm() -> &'static str {
        ctx().neighbour_realm_name.as_str()
    }

    async fn get_admin_token(server: &TestServer) -> String {
        let response = server
            .post(&format!(
                "/realms/{}/protocol/openid-connect/token",
                realm()
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;

        assert_eq!(response.status_code(), 200, "admin token request failed");
        let body: Value = response.json();
        body["access_token"]
            .as_str()
            .expect("access_token in response")
            .to_string()
    }

    fn auth_header(token: &str) -> HeaderValue {
        format!("Bearer {}", token).parse().unwrap()
    }

    async fn seed_local_user(label: &str) -> Uuid {
        let user_id = Uuid::new_v4();
        let realm_id: Uuid = sqlx::query("SELECT id FROM realms WHERE name = $1")
            .bind(realm())
            .fetch_one(&ctx().pool)
            .await
            .expect("fetch master realm")
            .get("id");

        sqlx::query(
            "INSERT INTO users (id, realm_id, username, firstname, lastname, email) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(user_id)
        .bind(realm_id)
        .bind(format!("{label}-{}", Uuid::new_v4().simple()))
        .bind("Local")
        .bind("User")
        .bind(format!("{label}-{}@test.local", Uuid::new_v4().simple()))
        .execute(&ctx().pool)
        .await
        .expect("insert local user");

        user_id
    }

    async fn seed_credential(user_id: Uuid, label: &str) -> Uuid {
        let credential_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO credentials (id, credential_type, user_id, user_label, secret_data, credential_data) VALUES ($1, $2, $3, $4, $5, $6::jsonb)",
        )
        .bind(credential_id)
        .bind("totp")
        .bind(user_id)
        .bind(label)
        .bind("seed-secret")
        .bind("{}")
        .execute(&ctx().pool)
        .await
        .expect("insert credential");

        credential_id
    }

    async fn credential_exists(credential_id: Uuid) -> bool {
        sqlx::query("SELECT 1 AS present FROM credentials WHERE id = $1")
            .bind(credential_id)
            .fetch_optional(&ctx().pool)
            .await
            .expect("query credential")
            .is_some()
    }

    async fn list_credentials(server: &TestServer, realm: &str, user_id: Uuid) -> TestResponse {
        let token = get_admin_token(server).await;
        server
            .get(&format!("/realms/{}/users/{}/credentials", realm, user_id))
            .add_header("Authorization", auth_header(&token))
            .await
    }

    async fn delete_credential(
        server: &TestServer,
        realm: &str,
        user_id: Uuid,
        credential_id: Uuid,
    ) -> TestResponse {
        let token = get_admin_token(server).await;
        server
            .delete(&format!(
                "/realms/{}/users/{}/credentials/{}",
                realm, user_id, credential_id
            ))
            .add_header("Authorization", auth_header(&token))
            .await
    }

    async fn generate_recovery_codes(server: &TestServer, realm: &str) -> TestResponse {
        let token = get_admin_token(server).await;
        server
            .post(&format!(
                "/realms/{}/login-actions/generate-recovery-codes",
                realm
            ))
            .add_header("Authorization", auth_header(&token))
            .json(&json!({ "amount": 2, "code_format": RECOVERY_CODE_FORMAT }))
            .await
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
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn listing_credentials_of_a_user_in_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let witness = list_credentials(&server, realm(), ctx().admin_user_id).await;
            let witness_body = witness.text();
            assert_eq!(
                witness.status_code(),
                200,
                "a user of the url realm must be listable, otherwise the refusal below proves \
                 nothing: {} {witness_body}",
                witness.status_code()
            );
            assert!(
                witness_body.contains("password"),
                "the admin has no listed password credential, so an empty list would satisfy the \
                 refusal below for the wrong reason: {witness_body}"
            );

            let refused = list_credentials(&server, realm(), ctx().neighbour_user_id).await;
            let refused_body = refused.text();

            assert_not_found(
                &refused,
                "listing the credentials of a neighbour-realm user from the url realm",
            );
            assert_body_free_of(
                &refused_body,
                &[
                    ctx().neighbour_credential_id.to_string().as_str(),
                    "seed-secret",
                ],
                "the refused listing of a neighbour-realm user's credentials",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn listing_credentials_of_an_unknown_user_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let refused = list_credentials(&server, realm(), Uuid::new_v4()).await;

            assert_not_found(
                &refused,
                "listing the credentials of a user id that exists in no realm",
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn deleting_a_credential_of_a_user_in_another_realm_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let owner = seed_local_user("own-realm-deletable").await;
            let deletable = seed_credential(owner, "own-realm-deletable").await;
            let accepted = delete_credential(&server, realm(), owner, deletable).await;
            assert_eq!(
                accepted.status_code(),
                200,
                "a credential of the url realm must be deletable, otherwise the refusal below \
                 proves nothing: {} {}",
                accepted.status_code(),
                accepted.text()
            );
            assert!(
                !credential_exists(deletable).await,
                "the accepted deletion did not reach the database, so the refusal below is not \
                 evidence of scoping"
            );

            let refused = delete_credential(
                &server,
                realm(),
                ctx().neighbour_user_id,
                ctx().neighbour_credential_id,
            )
            .await;

            assert_not_found(
                &refused,
                "deleting a neighbour-realm credential from the url realm",
            );
            assert!(
                credential_exists(ctx().neighbour_credential_id).await,
                "the neighbour realm's credential was deleted despite the refusal"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn deleting_a_neighbour_credential_addressed_under_a_local_user_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let local = seed_local_user("local-witness").await;
            let deletable = seed_credential(local, "local-witness").await;
            let accepted = delete_credential(&server, realm(), local, deletable).await;
            assert_eq!(
                accepted.status_code(),
                200,
                "the very same route must accept a credential that does belong to this user: {} {}",
                accepted.status_code(),
                accepted.text()
            );

            let refused =
                delete_credential(&server, realm(), local, ctx().neighbour_credential_id).await;

            assert_not_found(
                &refused,
                "deleting a neighbour-realm credential id under a local user id",
            );
            assert!(
                credential_exists(ctx().neighbour_credential_id).await,
                "a neighbour-realm credential was deleted by naming a local user in the url"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn deleting_a_local_credential_addressed_under_another_local_user_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let owner = seed_local_user("credential-owner").await;
            let victim = seed_credential(owner, "owned-by-another-user").await;

            let refused =
                delete_credential(&server, realm(), ctx().bystander_user_id, victim).await;

            assert_not_found(
                &refused,
                "deleting a credential of one realm user under another realm user's id",
            );
            assert!(
                credential_exists(victim).await,
                "a credential was deleted through a user id that does not own it"
            );

            let accepted = delete_credential(&server, realm(), owner, victim).await;
            assert_eq!(
                accepted.status_code(),
                200,
                "the owner's url must still delete the credential, otherwise the refusal above is \
                 not evidence of a parent check: {} {}",
                accepted.status_code(),
                accepted.text()
            );
            assert!(
                !credential_exists(victim).await,
                "the accepted deletion did not reach the database"
            );
        });
    }

    #[test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-api --test credential_cross_realm_test -- --ignored"]
    fn generating_recovery_codes_through_another_realms_url_is_refused() {
        rt().block_on(async {
            let server = make_server();

            let accepted = generate_recovery_codes(&server, realm()).await;
            let accepted_body = accepted.text();
            assert_eq!(
                accepted.status_code(),
                200,
                "a user must be able to generate recovery codes through its own realm's url, \
                 otherwise the refusal below proves nothing: {} {accepted_body}",
                accepted.status_code()
            );
            assert!(
                accepted_body.contains("codes"),
                "the accepted call returned no codes: {accepted_body}"
            );

            let refused = generate_recovery_codes(&server, neighbour_realm()).await;

            assert_not_found(
                &refused,
                "generating recovery codes for a url realm the caller does not belong to",
            );

            let recovery_codes_in_neighbour: i64 = sqlx::query(
                "SELECT COUNT(*) AS total FROM credentials c JOIN users u ON u.id = c.user_id WHERE u.realm_id = (SELECT id FROM realms WHERE name = $1) AND c.credential_type = 'recovery-code'",
            )
            .bind(neighbour_realm())
            .fetch_one(&ctx().pool)
            .await
            .expect("count neighbour recovery codes")
            .get("total");

            assert_eq!(
                recovery_codes_in_neighbour, 0,
                "the refused call still wrote recovery codes into the neighbour realm"
            );
        });
    }
}
