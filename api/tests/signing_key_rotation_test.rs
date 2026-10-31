mod common;

use axum_test::TestServer;
use common::{TestApp, app, password_grant, rt};
use serde_json::Value;
use uuid::Uuid;

const PASSWORD: &str = "Str0ng!P@ssword#2024";

fn kid_of(token: &str) -> String {
    jsonwebtoken::decode_header(token)
        .expect("jwt header")
        .kid
        .expect("kid in header")
}

async fn published_kids(server: &TestServer, app: &TestApp) -> Vec<String> {
    let jwks: Value = server.get(&app.oidc("certs")).await.json();
    jwks["keys"]
        .as_array()
        .expect("jwks keys")
        .iter()
        .map(|key| key["kid"].as_str().expect("kid").to_string())
        .collect()
}

async fn listed_keys(server: &TestServer, app: &TestApp, token: &str) -> axum_test::TestResponse {
    server
        .get(&app.path("/keys"))
        .authorization_bearer(token)
        .await
}

fn status_of(keys: &Value, kid: &str) -> String {
    keys["data"]
        .as_array()
        .expect("listed keys")
        .iter()
        .find(|key| key["kid"] == kid)
        .and_then(|key| key["status"].as_str())
        .unwrap_or_else(|| panic!("{kid} is not listed: {keys}"))
        .to_string()
}

async fn security_events(app: &TestApp, event_type: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM security_events WHERE realm_id = $1 AND event_type = $2",
    )
    .bind(app.realm_id)
    .bind(event_type)
    .fetch_one(&app.pool)
    .await
    .expect("count security events")
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test signing_key_rotation_test -- --ignored"]
fn a_rotated_key_keeps_verifying_until_it_is_revoked() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let first_token = app.admin_token(&server).await;
        let first_kid = kid_of(&first_token);

        let user = app.user(&server, PASSWORD).await;
        let user_token = password_grant(
            &server,
            app,
            common::ADMIN_CLIENT_ID,
            None,
            &user.username,
            PASSWORD,
        )
        .await
        .json::<Value>()["access_token"]
            .as_str()
            .expect("user access token")
            .to_string();
        let refused = server
            .post(&app.path("/keys/rotate"))
            .authorization_bearer(&user_token)
            .await;
        assert_eq!(refused.status_code(), 403, "{}", refused.text());

        let rotated = server
            .post(&app.path("/keys/rotate"))
            .authorization_bearer(&first_token)
            .await;
        assert_eq!(rotated.status_code(), 201, "{}", rotated.text());
        let rotated: Value = rotated.json();
        let second_kid = rotated["data"]["kid"]
            .as_str()
            .expect("new kid")
            .to_string();
        assert_ne!(second_kid, first_kid);
        assert_eq!(rotated["data"]["status"], "active");

        assert_eq!(
            published_kids(&server, app).await,
            vec![second_kid.clone(), first_kid.clone()]
        );

        let still_valid = listed_keys(&server, app, &first_token).await;
        assert_eq!(still_valid.status_code(), 200, "{}", still_valid.text());
        let keys: Value = still_valid.json();
        assert_eq!(status_of(&keys, &first_kid), "passive");
        assert_eq!(status_of(&keys, &second_kid), "active");

        let second_token = app.admin_token(&server).await;
        assert_eq!(kid_of(&second_token), second_kid);

        let revoked = server
            .post(&app.path(&format!("/keys/{first_kid}/revoke")))
            .authorization_bearer(&second_token)
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
        assert_eq!(revoked.json::<Value>()["data"]["status"], "revoked");

        let rejected = listed_keys(&server, app, &first_token).await;
        assert_eq!(rejected.status_code(), 401, "{}", rejected.text());
        assert_eq!(published_kids(&server, app).await, vec![second_kid.clone()]);
        assert_eq!(
            listed_keys(&server, app, &second_token).await.status_code(),
            200
        );

        let unknown = server
            .post(&app.path(&format!("/keys/{}/revoke", Uuid::new_v4())))
            .authorization_bearer(&second_token)
            .await;
        assert_eq!(unknown.status_code(), 404, "{}", unknown.text());

        assert_eq!(security_events(app, "signing_key_rotated").await, 1);
        assert_eq!(security_events(app, "signing_key_revoked").await, 1);
    });
}
