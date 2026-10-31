/// Client authentication with a signed JWT (RFC 7523): `private_key_jwt` and
/// `client_secret_jwt` at the token, introspection and revocation endpoints.
///
/// Run with:
///   cargo test -p ferriskey-api --test client_jwt_auth_test -- --ignored
#[cfg(test)]
mod tests {
    use std::{env, sync::Arc};

    use axum::http::HeaderValue;
    use axum_test::TestServer;
    use ferriskey_api::{
        application::http::server::{app_state::AppState, http_server::router},
        args::Args,
    };
    use ferriskey_core::{
        application::create_service,
        domain::{
            common::{
                DatabaseConfig, FerriskeyConfig, PasswordHashingCost, entities::StartupConfig,
                ports::CoreService,
            },
            jwt::entities::JwtKeyPair,
        },
    };
    use jsonwebtoken::{Algorithm, EncodingKey, Header};
    use serde_json::{Value, json};
    use sqlx::Executor;
    use uuid::Uuid;

    const ASSERTION_TYPE: &str = "urn:ietf:params:oauth:client-assertion-type:jwt-bearer";

    fn env_or(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    struct TestContext {
        server: TestServer,
        realm_name: String,
        admin_token: String,
        issuer: String,
        token_endpoint: String,
    }

    impl TestContext {
        fn path(&self, suffix: &str) -> String {
            format!("/realms/{}{suffix}", self.realm_name)
        }
    }

    async fn setup() -> TestContext {
        let db_host = env_or("DATABASE_HOST", "localhost");
        let db_port: u16 = env_or("DATABASE_PORT", "5432").parse().expect("port");
        let db_name = env_or("DATABASE_NAME", "ferriskey");
        let db_user = env_or("DATABASE_USER", "ferriskey");
        let db_password = env_or("DATABASE_PASSWORD", "ferriskey");
        let schema = format!("client_jwt_test_{}", Uuid::new_v4().simple());

        let admin_url = format!("postgres://{db_user}:{db_password}@{db_host}:{db_port}/{db_name}");
        let admin_pool = sqlx::PgPool::connect(&admin_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(
                "CREATE SCHEMA IF NOT EXISTS \"{schema}\""
            )))
            .await
            .expect("create schema");

        let schema_url = format!(
            "{admin_url}?options=-c search_path={}",
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
            client_metadata_allow_private_endpoints: true,
            config_cache_ttl_seconds: 0,
            password_hashing: PasswordHashingCost::default(),
            refresh_token_format: Default::default(),
            refresh_token_commit: Default::default(),
            webapp_url: "http://localhost:5555".to_string(),
            database: DatabaseConfig {
                host: db_host,
                port: db_port,
                username: db_user,
                password: db_password,
                name: db_name,
                schema,
                max_connections: 20,
                min_connections: 1,
            },
        })
        .await
        .expect("create service");

        let realm_name = format!("realm-{}", Uuid::new_v4().simple());
        service
            .initialize_application(StartupConfig {
                webapp_url: "http://localhost:5555".to_string(),
                master_realm_name: realm_name.clone(),
                admin_username: "admin".to_string(),
                admin_password: "admin".to_string(),
                admin_email: "admin@test.local".to_string(),
                default_client_id: "ferriskey-admin".to_string(),
            })
            .await
            .expect("initialize application");

        let state = AppState::new(Arc::new(Args::default()), service);
        let server = TestServer::new(router(state).expect("router")).expect("test server");

        let discovery: Value = server
            .get(&format!(
                "/realms/{realm_name}/.well-known/openid-configuration"
            ))
            .await
            .json();
        let issuer = discovery["issuer"].as_str().expect("issuer").to_string();
        let token_endpoint = discovery["token_endpoint"]
            .as_str()
            .expect("token_endpoint")
            .to_string();

        let response = server
            .post(&format!(
                "/realms/{realm_name}/protocol/openid-connect/token"
            ))
            .form(&[
                ("grant_type", "password"),
                ("client_id", "admin-cli"),
                ("username", "admin"),
                ("password", "admin"),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "admin token");
        let admin_token = response.json::<Value>()["access_token"]
            .as_str()
            .expect("access_token")
            .to_string();

        TestContext {
            server,
            realm_name,
            admin_token,
            issuer,
            token_endpoint,
        }
    }

    struct Signer {
        kid: String,
        encoding: EncodingKey,
        jwk: Value,
    }

    fn new_signer() -> Signer {
        let (private, public) = JwtKeyPair::generate().expect("generate a key");
        let certificate =
            JwtKeyPair::self_signed_certificate(&private, "test").expect("certificate");
        let pair = JwtKeyPair::from_pem(
            &private,
            &public,
            &certificate,
            Uuid::new_v4(),
            Uuid::new_v4(),
        )
        .expect("key pair");
        let jwk = serde_json::to_value(pair.to_jwk_key().expect("jwk")).expect("json");
        Signer {
            kid: jwk["kid"].as_str().expect("kid").to_string(),
            encoding: EncodingKey::from_rsa_pem(private.as_bytes()).expect("encoding key"),
            jwk,
        }
    }

    fn claims(client_id: &str, audience: &str) -> Value {
        let now = chrono::Utc::now().timestamp();
        json!({
            "iss": client_id,
            "sub": client_id,
            "aud": audience,
            "iat": now,
            "exp": now + 60,
            "jti": Uuid::new_v4().to_string(),
        })
    }

    fn sign(signer: &Signer, claims: &Value) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(signer.kid.clone());
        jsonwebtoken::encode(&header, claims, &signer.encoding).expect("sign")
    }

    fn sign_with_secret(secret: &str, claims: &Value) -> String {
        jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("sign")
    }

    async fn create_client(ctx: &TestContext, auth: Value) -> (String, Option<String>) {
        let client_id = format!("jwt-client-{}", Uuid::new_v4().simple());
        let mut body = json!({
            "client_id": client_id,
            "name": "JWT client",
            "client_type": "confidential",
            "protocol": "openid-connect",
            "public_client": false,
            "service_account_enabled": true,
            "enabled": true,
        });
        body.as_object_mut()
            .expect("object")
            .extend(auth.as_object().expect("object").clone());

        let response = ctx
            .server
            .post(&ctx.path("/clients"))
            .add_header(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", ctx.admin_token)).expect("header"),
            )
            .json(&body)
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());
        let created: Value = response.json();
        (
            client_id,
            created["client_secret"].as_str().map(str::to_string),
        )
    }

    async fn client_credentials(ctx: &TestContext, form: &[(&str, &str)]) -> (u16, Value) {
        let mut fields = vec![("grant_type", "client_credentials")];
        fields.extend_from_slice(form);
        let response = ctx
            .server
            .post(&ctx.path("/protocol/openid-connect/token"))
            .form(&fields)
            .await;
        let status = response.status_code().as_u16();
        let body = serde_json::from_str(&response.text()).unwrap_or(Value::Null);
        (status, body)
    }

    async fn with_assertion(ctx: &TestContext, assertion: &str) -> (u16, Value) {
        client_credentials(
            ctx,
            &[
                ("client_assertion_type", ASSERTION_TYPE),
                ("client_assertion", assertion),
            ],
        )
        .await
    }

    #[tokio::test]
    #[ignore]
    async fn a_private_key_jwt_client_gets_a_token_and_cannot_replay_its_assertion() {
        let ctx = setup().await;
        let signer = new_signer();
        let (client_id, _) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;

        let assertion = sign(&signer, &claims(&client_id, &ctx.token_endpoint));

        let (status, body) = with_assertion(&ctx, &assertion).await;
        assert_eq!(status, 200, "{body}");
        assert!(body["access_token"].is_string());

        let (status, body) = with_assertion(&ctx, &assertion).await;
        assert_eq!(status, 401, "a replayed assertion must be refused");
        assert_eq!(body["error"], "invalid_client");
    }

    #[tokio::test]
    #[ignore]
    async fn the_realm_issuer_is_an_accepted_audience() {
        let ctx = setup().await;
        let signer = new_signer();
        let (client_id, _) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;

        let (status, body) =
            with_assertion(&ctx, &sign(&signer, &claims(&client_id, &ctx.issuer))).await;
        assert_eq!(status, 200, "{body}");
    }

    #[tokio::test]
    #[ignore]
    async fn invalid_assertions_are_invalid_client() {
        let ctx = setup().await;
        let signer = new_signer();
        let stranger = new_signer();
        let (client_id, _) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;

        let mut wrong_issuer = claims(&client_id, &ctx.token_endpoint);
        wrong_issuer["iss"] = json!("someone-else");
        let mut expired = claims(&client_id, &ctx.token_endpoint);
        expired["exp"] = json!(chrono::Utc::now().timestamp() - 300);

        let mut forged_header = Header::new(Algorithm::RS256);
        forged_header.kid = Some(signer.kid.clone());
        let forged = jsonwebtoken::encode(
            &forged_header,
            &claims(&client_id, &ctx.token_endpoint),
            &stranger.encoding,
        )
        .expect("sign");

        for (case, assertion) in [
            ("forged signature", forged),
            (
                "wrong audience",
                sign(&signer, &claims(&client_id, "https://elsewhere.example")),
            ),
            ("wrong issuer", sign(&signer, &wrong_issuer)),
            ("expired", sign(&signer, &expired)),
        ] {
            let (status, body) = client_credentials(
                &ctx,
                &[
                    ("client_id", client_id.as_str()),
                    ("client_assertion_type", ASSERTION_TYPE),
                    ("client_assertion", assertion.as_str()),
                ],
            )
            .await;
            assert_eq!(status, 401, "{case}: {body}");
            assert_eq!(body["error"], "invalid_client", "{case}");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn a_private_key_jwt_client_cannot_use_a_secret() {
        let ctx = setup().await;
        let signer = new_signer();
        let (client_id, secret) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;
        let secret = secret.expect("the client still has a secret on record");

        let (status, body) = client_credentials(
            &ctx,
            &[
                ("client_id", client_id.as_str()),
                ("client_secret", secret.as_str()),
            ],
        )
        .await;
        assert_eq!(status, 401, "{body}");
    }

    #[tokio::test]
    #[ignore]
    async fn an_assertion_and_a_secret_together_are_refused() {
        let ctx = setup().await;
        let signer = new_signer();
        let (client_id, secret) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;
        let assertion = sign(&signer, &claims(&client_id, &ctx.token_endpoint));

        let (status, body) = client_credentials(
            &ctx,
            &[
                ("client_secret", secret.as_deref().expect("secret")),
                ("client_assertion_type", ASSERTION_TYPE),
                ("client_assertion", assertion.as_str()),
            ],
        )
        .await;
        assert_eq!(status, 400, "{body}");
        assert_eq!(body["error"], "invalid_request");
    }

    #[tokio::test]
    #[ignore]
    async fn introspection_and_revocation_accept_an_assertion() {
        let ctx = setup().await;
        let signer = new_signer();
        let (client_id, _) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks": { "keys": [signer.jwk] },
            }),
        )
        .await;

        let (status, body) = with_assertion(
            &ctx,
            &sign(&signer, &claims(&client_id, &ctx.token_endpoint)),
        )
        .await;
        assert_eq!(status, 200, "{body}");
        let access_token = body["access_token"].as_str().expect("token").to_string();

        let introspection_endpoint =
            format!("{}/protocol/openid-connect/token/introspect", ctx.issuer);
        let assertion = sign(&signer, &claims(&client_id, &introspection_endpoint));
        let response = ctx
            .server
            .post(&ctx.path("/protocol/openid-connect/token/introspect"))
            .form(&[
                ("token", access_token.as_str()),
                ("client_assertion_type", ASSERTION_TYPE),
                ("client_assertion", assertion.as_str()),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        assert_eq!(response.json::<Value>()["active"], true);

        let response = ctx
            .server
            .post(&ctx.path("/protocol/openid-connect/token/introspect"))
            .form(&[
                ("token", access_token.as_str()),
                ("client_assertion_type", ASSERTION_TYPE),
                ("client_assertion", assertion.as_str()),
            ])
            .await;
        assert_eq!(
            response.status_code(),
            401,
            "a replayed assertion must be refused at introspection"
        );

        let assertion = sign(&signer, &claims(&client_id, &ctx.token_endpoint));
        let response = ctx
            .server
            .post(&ctx.path("/protocol/openid-connect/revoke"))
            .form(&[
                ("token", access_token.as_str()),
                ("client_assertion_type", ASSERTION_TYPE),
                ("client_assertion", assertion.as_str()),
            ])
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
    }

    #[tokio::test]
    #[ignore]
    async fn a_client_secret_jwt_client_signs_with_its_secret() {
        let ctx = setup().await;
        let (client_id, secret) = create_client(
            &ctx,
            json!({ "token_endpoint_auth_method": "client_secret_jwt" }),
        )
        .await;
        let secret = secret.expect("a client_secret_jwt client has a secret");

        let (status, body) = with_assertion(
            &ctx,
            &sign_with_secret(&secret, &claims(&client_id, &ctx.token_endpoint)),
        )
        .await;
        assert_eq!(status, 200, "{body}");

        let (status, _) = client_credentials(
            &ctx,
            &[
                ("client_id", client_id.as_str()),
                ("client_secret", secret.as_str()),
            ],
        )
        .await;
        assert_eq!(status, 401, "the secret itself is no longer accepted");
    }

    #[tokio::test]
    #[ignore]
    async fn keys_are_fetched_from_the_jwks_uri() {
        let ctx = setup().await;
        let signer = new_signer();
        let jwks = json!({ "keys": [signer.jwk] });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let address = listener.local_addr().expect("address");
        let app = axum::Router::new().route(
            "/jwks.json",
            axum::routing::get(move || {
                let jwks = jwks.clone();
                async move { axum::Json(jwks) }
            }),
        );
        tokio::spawn(async move { axum::serve(listener, app).await });

        let (client_id, _) = create_client(
            &ctx,
            json!({
                "token_endpoint_auth_method": "private_key_jwt",
                "jwks_uri": format!("http://{address}/jwks.json"),
            }),
        )
        .await;

        for _ in 0..2 {
            let (status, body) = with_assertion(
                &ctx,
                &sign(&signer, &claims(&client_id, &ctx.token_endpoint)),
            )
            .await;
            assert_eq!(status, 200, "{body}");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn private_key_jwt_without_keys_cannot_be_configured() {
        let ctx = setup().await;
        let response = ctx
            .server
            .post(&ctx.path("/clients"))
            .add_header(
                "Authorization",
                HeaderValue::from_str(&format!("Bearer {}", ctx.admin_token)).expect("header"),
            )
            .json(&json!({
                "client_id": format!("jwt-client-{}", Uuid::new_v4().simple()),
                "name": "JWT client",
                "client_type": "confidential",
                "public_client": false,
                "enabled": true,
                "token_endpoint_auth_method": "private_key_jwt",
            }))
            .await;
        assert_eq!(response.status_code(), 400, "{}", response.text());
    }

    #[tokio::test]
    #[ignore]
    async fn discovery_advertises_the_jwt_methods_and_algorithms() {
        let ctx = setup().await;
        for path in [
            ctx.path("/.well-known/openid-configuration"),
            format!(
                "/.well-known/oauth-authorization-server/realms/{}",
                ctx.realm_name
            ),
        ] {
            let body: Value = ctx.server.get(&path).await.json();
            let methods = body["token_endpoint_auth_methods_supported"].to_string();
            assert!(methods.contains("private_key_jwt"), "{path}: {methods}");
            assert!(methods.contains("client_secret_jwt"), "{path}: {methods}");
            let algs = body["token_endpoint_auth_signing_alg_values_supported"].to_string();
            assert!(
                algs.contains("RS256") && algs.contains("ES256"),
                "{path}: {algs}"
            );
            assert!(!algs.contains("\"none\""), "{path}: {algs}");
        }
    }
}
