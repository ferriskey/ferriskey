/// Pushed authorization requests (RFC 9126): the PAR endpoint, `/auth` with a
/// `request_uri`, and `require_pushed_authorization_requests`.
///
/// Run with:
///   cargo test -p ferriskey-api --test par_test -- --ignored
mod common;

use axum::http::{HeaderValue, header::AUTHORIZATION};
use axum_test::{TestResponse, TestServer};
use base64::{Engine, engine::general_purpose::STANDARD};
use common::{
    ADMIN_PASSWORD, ADMIN_USERNAME, CALLBACK, ClientSpec, SeededClient, TestApp, app,
    exchange_code, query_param, rt,
};
use serde_json::{Value, json};
use uuid::Uuid;

const REQUEST_URI_PREFIX: &str = "urn:ietf:params:oauth:request_uri:";
const VERIFIER: &str = "dBjftJeZ4CVP-mJ92ZoABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const CHALLENGE: &str = "rBI9bJDRtFfiSCS6FjhyAwXOa--HMMyPWIM2PLVirDU";

fn par_path(realm: &str) -> String {
    format!("/realms/{realm}/protocol/openid-connect/ext/par/request")
}

fn auth_path(realm: &str) -> String {
    format!("/realms/{realm}/protocol/openid-connect/auth")
}

fn pushed_form<'a>(
    client: &'a SeededClient,
    extra: &[(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    let mut form = vec![
        ("client_id", client.client_id.as_str()),
        ("response_type", "code"),
        ("redirect_uri", CALLBACK),
        ("scope", "openid"),
        ("state", "pushed-state"),
        ("code_challenge", CHALLENGE),
        ("code_challenge_method", "S256"),
    ];
    if let Some(secret) = client.secret.as_deref() {
        form.push(("client_secret", secret));
    }
    for (key, value) in extra {
        form.retain(|(existing, _)| existing != key);
        form.push((key, value));
    }
    form
}

async fn push(
    server: &TestServer,
    realm: &str,
    client: &SeededClient,
    extra: &[(&str, &str)],
) -> TestResponse {
    server
        .post(&par_path(realm))
        .form(&pushed_form(client, extra))
        .await
}

async fn pushed_request_uri(server: &TestServer, realm: &str, client: &SeededClient) -> String {
    let response = push(server, realm, client, &[]).await;
    assert_eq!(response.status_code(), 201, "{}", response.text());
    response.json::<Value>()["request_uri"]
        .as_str()
        .expect("request_uri")
        .to_string()
}

async fn authorize_with(
    server: &TestServer,
    realm: &str,
    client_id: &str,
    request_uri: &str,
    extra: &[(&str, &str)],
) -> TestResponse {
    let mut request = server
        .get(&auth_path(realm))
        .add_query_param("client_id", client_id)
        .add_query_param("request_uri", request_uri);
    for (key, value) in extra {
        request = request.add_query_param(key, value);
    }
    request.await
}

fn location(response: &TestResponse) -> String {
    response
        .headers()
        .get("location")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

fn is_login_error(response: &TestResponse) -> bool {
    response.status_code() == 302
        && location(response).contains("login_error=")
        && response.maybe_cookie("FERRISKEY_SESSION").is_none()
}

async fn sign_in(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    authorized: &TestResponse,
) -> String {
    assert_eq!(
        authorized.status_code(),
        302,
        "authorize: {}",
        authorized.text()
    );
    let authenticated = server
        .post(&app.path("/login-actions/authenticate"))
        .add_cookie(authorized.cookie("FERRISKEY_SESSION"))
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": ADMIN_USERNAME, "password": ADMIN_PASSWORD }))
        .await;
    assert_eq!(
        authenticated.status_code(),
        200,
        "authenticate: {}",
        authenticated.text()
    );
    authenticated.json::<Value>()["url"]
        .as_str()
        .expect("redirect url")
        .to_string()
}

async fn require_par(app: &TestApp, client: &SeededClient) {
    sqlx::query("UPDATE clients SET require_pushed_authorization_requests = TRUE WHERE id = $1")
        .bind(client.id)
        .execute(&app.pool)
        .await
        .expect("require par for the client");
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_confidential_client_pushes_a_request_and_gets_a_single_use_request_uri() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;

        let response = push(&server, &app.realm, &client, &[]).await;

        assert_eq!(response.status_code(), 201, "{}", response.text());
        assert_eq!(
            response.headers().get("cache-control"),
            Some(&HeaderValue::from_static("no-store"))
        );
        let body: Value = response.json();
        assert!(
            body["request_uri"]
                .as_str()
                .is_some_and(|uri| uri.starts_with(REQUEST_URI_PREFIX)),
            "{body}"
        );
        assert_eq!(body["expires_in"], 60);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn the_par_endpoint_authenticates_the_client_like_the_token_endpoint() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;

        let wrong = SeededClient {
            id: client.id,
            client_id: client.client_id.clone(),
            secret: Some("wrong".to_string()),
        };
        let response = push(&server, &app.realm, &wrong, &[]).await;
        assert_eq!(response.status_code(), 401, "{}", response.text());
        assert_eq!(response.json::<Value>()["error"], "invalid_client");

        let missing = SeededClient {
            secret: None,
            ..wrong
        };
        let response = push(&server, &app.realm, &missing, &[]).await;
        assert_eq!(response.status_code(), 401, "{}", response.text());

        let basic = STANDARD.encode(format!(
            "{}:{}",
            client.client_id,
            client.secret.as_deref().expect("secret")
        ));
        let response = server
            .post(&par_path(&app.realm))
            .add_header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {basic}")).expect("header"),
            )
            .form(&[
                ("response_type", "code"),
                ("redirect_uri", CALLBACK),
                ("code_challenge", CHALLENGE),
                ("code_challenge_method", "S256"),
            ])
            .await;
        assert_eq!(response.status_code(), 201, "{}", response.text());

        let response = server
            .post(&par_path(&app.realm))
            .add_header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Basic {basic}")).expect("header"),
            )
            .form(&[
                ("client_id", "someone-else"),
                ("response_type", "code"),
                ("redirect_uri", CALLBACK),
            ])
            .await;
        assert_eq!(response.status_code(), 400, "{}", response.text());
        assert_eq!(response.json::<Value>()["error"], "invalid_request");
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn the_par_endpoint_validates_the_request_like_auth() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;

        let response = push(
            &server,
            &app.realm,
            &client,
            &[("redirect_uri", "https://evil.example/cb")],
        )
        .await;
        assert_eq!(response.status_code(), 400, "{}", response.text());
        assert_eq!(response.json::<Value>()["error"], "invalid_request");

        let response = push(
            &server,
            &app.realm,
            &client,
            &[("request_uri", "urn:ietf:params:oauth:request_uri:abc")],
        )
        .await;
        assert_eq!(response.status_code(), 400, "{}", response.text());

        let mut pkce = ClientSpec::confidential();
        pkce.require_pkce = true;
        let pkce_client = app.client(pkce).await;
        let response = server
            .post(&par_path(&app.realm))
            .form(&[
                ("client_id", pkce_client.client_id.as_str()),
                (
                    "client_secret",
                    pkce_client.secret.as_deref().expect("secret"),
                ),
                ("response_type", "code"),
                ("redirect_uri", CALLBACK),
            ])
            .await;
        assert_eq!(response.status_code(), 400, "{}", response.text());
        assert_eq!(response.json::<Value>()["error"], "invalid_request");
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_pushed_request_completes_the_code_flow_and_ignores_query_parameters() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;
        let request_uri = pushed_request_uri(&server, &app.realm, &client).await;

        let authorized = authorize_with(
            &server,
            &app.realm,
            &client.client_id,
            &request_uri,
            &[
                ("redirect_uri", "https://evil.example/cb"),
                ("state", "query-state"),
                ("response_type", "token"),
            ],
        )
        .await;
        let redirect = sign_in(&server, app, &client, &authorized).await;

        assert!(redirect.starts_with(CALLBACK), "{redirect}");
        assert_eq!(
            query_param(&redirect, "state").as_deref(),
            Some("pushed-state")
        );
        let code = query_param(&redirect, "code").expect("code");
        let token =
            exchange_code(&server, app, &client, &code, &[("code_verifier", VERIFIER)]).await;
        assert_eq!(token.status_code(), 200, "{}", token.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_request_uri_is_single_use() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;
        let request_uri = pushed_request_uri(&server, &app.realm, &client).await;

        let first = authorize_with(&server, &app.realm, &client.client_id, &request_uri, &[]).await;
        assert_eq!(first.status_code(), 302, "{}", first.text());
        assert!(first.maybe_cookie("FERRISKEY_SESSION").is_some());

        let second =
            authorize_with(&server, &app.realm, &client.client_id, &request_uri, &[]).await;
        assert!(is_login_error(&second), "{}", location(&second));
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_request_uri_is_bound_to_the_client_that_pushed_it() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;
        let other = app.client(ClientSpec::public()).await;
        let request_uri = pushed_request_uri(&server, &app.realm, &client).await;

        let stolen = authorize_with(&server, &app.realm, &other.client_id, &request_uri, &[]).await;
        assert!(is_login_error(&stolen), "{}", location(&stolen));

        let unknown = authorize_with(
            &server,
            &app.realm,
            &client.client_id,
            &format!("{REQUEST_URI_PREFIX}{}", Uuid::new_v4().simple()),
            &[],
        )
        .await;
        assert!(is_login_error(&unknown), "{}", location(&unknown));

        let owner = authorize_with(&server, &app.realm, &client.client_id, &request_uri, &[]).await;
        assert_eq!(
            owner.status_code(),
            302,
            "a refused attempt must not consume the request: {}",
            owner.text()
        );
        assert!(owner.maybe_cookie("FERRISKEY_SESSION").is_some());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_public_client_pushes_with_pkce_and_no_secret() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;

        let request_uri = pushed_request_uri(&server, &app.realm, &client).await;
        let authorized =
            authorize_with(&server, &app.realm, &client.client_id, &request_uri, &[]).await;

        assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
        assert!(authorized.maybe_cookie("FERRISKEY_SESSION").is_some());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_client_that_requires_par_refuses_a_plain_authorization_request() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;
        require_par(app, &client).await;

        let plain = common::authorize(&server, app, &client.client_id, &[("state", "s1")]).await;
        assert_eq!(plain.status_code(), 302, "{}", plain.text());
        let refused = location(&plain);
        assert!(refused.starts_with(CALLBACK), "{refused}");
        assert_eq!(
            query_param(&refused, "error").as_deref(),
            Some("invalid_request")
        );
        assert_eq!(query_param(&refused, "state").as_deref(), Some("s1"));

        let request_uri = pushed_request_uri(&server, &app.realm, &client).await;
        let pushed =
            authorize_with(&server, &app.realm, &client.client_id, &request_uri, &[]).await;
        assert_eq!(pushed.status_code(), 302, "{}", pushed.text());
        assert!(pushed.maybe_cookie("FERRISKEY_SESSION").is_some());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn a_realm_that_requires_par_refuses_plain_requests_and_says_so_in_discovery() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let (realm, realm_id) = app.create_realm().await;
        sqlx::query("INSERT INTO realm_settings (id, realm_id) VALUES ($1, $2)")
            .bind(Uuid::new_v4())
            .bind(realm_id)
            .execute(&app.pool)
            .await
            .expect("insert realm settings");
        let client = app.client_in(realm_id, ClientSpec::public()).await;

        let metadata = server
            .get(&format!("/realms/{realm}/.well-known/openid-configuration"))
            .await
            .json::<Value>();
        assert!(
            metadata["pushed_authorization_request_endpoint"]
                .as_str()
                .is_some_and(|endpoint| endpoint.ends_with(&par_path(&realm))),
            "{metadata}"
        );
        assert_eq!(metadata["require_pushed_authorization_requests"], false);

        sqlx::query(
            "UPDATE realm_settings SET require_pushed_authorization_requests = TRUE \
             WHERE realm_id = $1",
        )
        .bind(realm_id)
        .execute(&app.pool)
        .await
        .expect("require par for the realm");

        for path in [
            format!("/realms/{realm}/.well-known/openid-configuration"),
            format!("/.well-known/oauth-authorization-server/realms/{realm}"),
        ] {
            let metadata = server.get(&path).await.json::<Value>();
            assert_eq!(
                metadata["require_pushed_authorization_requests"], true,
                "{path}: {metadata}"
            );
        }

        let plain = server
            .get(&auth_path(&realm))
            .add_query_param("response_type", "code")
            .add_query_param("client_id", &client.client_id)
            .add_query_param("redirect_uri", CALLBACK)
            .await;
        assert_eq!(
            query_param(&location(&plain), "error").as_deref(),
            Some("invalid_request"),
            "{}",
            location(&plain)
        );

        let request_uri = pushed_request_uri(&server, &realm, &client).await;
        let pushed = authorize_with(&server, &realm, &client.client_id, &request_uri, &[]).await;
        assert_eq!(pushed.status_code(), 302, "{}", pushed.text());
        assert!(pushed.maybe_cookie("FERRISKEY_SESSION").is_some());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test par_test -- --ignored"]
fn the_client_flag_is_set_through_the_admin_api() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let admin = app.admin_token(&server).await;
        let client = app.client(ClientSpec::confidential()).await;

        let updated = server
            .patch(&app.path(&format!("/clients/{}", client.id)))
            .authorization_bearer(&admin)
            .json(&json!({ "require_pushed_authorization_requests": true }))
            .await;
        assert_eq!(updated.status_code(), 200, "{}", updated.text());
        assert_eq!(
            updated.json::<Value>()["data"]["require_pushed_authorization_requests"],
            true
        );

        let plain = common::authorize(&server, app, &client.client_id, &[]).await;
        assert_eq!(
            query_param(&location(&plain), "error").as_deref(),
            Some("invalid_request")
        );
    });
}
