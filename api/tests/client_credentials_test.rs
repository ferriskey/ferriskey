mod common;

use common::{ClientSpec, SeededClient, TestApp, app, decode_unverified, rt};
use serde_json::Value;
use uuid::Uuid;

async fn grant(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    scope: Option<&str>,
) -> Value {
    let mut form: Vec<(&str, &str)> = vec![
        ("grant_type", "client_credentials"),
        ("client_id", client.client_id.as_str()),
    ];
    if let Some(secret) = client.secret.as_deref() {
        form.push(("client_secret", secret));
    }
    if let Some(scope) = scope {
        form.push(("scope", scope));
    }
    let response = server.post(&app.oidc("token")).form(&form).await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json()
}

fn subject(body: &Value) -> Uuid {
    let access_token = body["access_token"].as_str().expect("access_token");
    decode_unverified(access_token)["sub"]
        .as_str()
        .and_then(|sub| Uuid::parse_str(sub).ok())
        .expect("the access token names its service account")
}

async fn stored_tokens(app: &TestApp, table: &str, user_id: Uuid) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table} WHERE user_id = $1"))
        .bind(user_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the stored tokens")
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_credentials_test -- --ignored"]
fn a_client_credentials_grant_issues_no_refresh_token() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;

        let body = grant(&server, app, &client, None).await;

        assert_eq!(body["token_type"], "Bearer");
        assert!(body["expires_in"].as_u64().is_some_and(|ttl| ttl > 0));
        assert!(
            body.get("refresh_token").is_none() && body.get("refresh_expires_in").is_none(),
            "RFC 6749 §4.4.3: no refresh token for this grant, got {body}"
        );
        let service_account = subject(&body);
        assert_eq!(
            stored_tokens(app, "refresh_tokens", service_account).await,
            0
        );
        assert_eq!(
            stored_tokens(app, "access_tokens", service_account).await,
            1
        );
    });
}
