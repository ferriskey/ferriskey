mod common;

use common::{ClientSpec, TestApp, app, decode_unverified, rt};
use serde_json::{Value, json};

const ALLOWED: &str = "https://api.example";
const OTHER: &str = "https://other.example";

async fn token(
    server: &axum_test::TestServer,
    app: &TestApp,
    form: &[(&str, &str)],
) -> axum_test::TestResponse {
    server.post(&app.oidc("token")).form(form).await
}

async fn sessions_of(app: &TestApp, user_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM user_sessions WHERE user_id = $1::uuid")
        .bind(user_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the sessions of the user")
}

fn audience(body: &Value) -> Value {
    decode_unverified(body["access_token"].as_str().expect("access_token"))["aud"].clone()
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_resource_direct_grants_test -- --ignored"]
fn the_client_credentials_grant_honours_the_resource_parameter() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        app.set_allowed_resources(&[ALLOWED]).await;
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;
        let secret = client.secret.clone().expect("secret");
        let base = [
            ("grant_type", "client_credentials"),
            ("client_id", client.client_id.as_str()),
            ("client_secret", secret.as_str()),
        ];

        let mut allowed = base.to_vec();
        allowed.push(("resource", ALLOWED));
        let granted = token(&server, app, &allowed).await;
        assert_eq!(granted.status_code(), 200, "{}", granted.text());
        assert_eq!(audience(&granted.json()), json!([ALLOWED]));

        let mut refused = base.to_vec();
        refused.push(("resource", OTHER));
        let refused = token(&server, app, &refused).await;
        assert_eq!(refused.status_code(), 400, "{}", refused.text());
        assert_eq!(refused.json::<Value>()["error"], "invalid_target");
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_resource_direct_grants_test -- --ignored"]
fn the_password_grant_honours_the_resource_parameter() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        app.set_allowed_resources(&[ALLOWED]).await;
        let client = app.client(ClientSpec::public().with_direct_access()).await;
        let user = app.user(&server, "Correct-horse-battery-7").await;
        let base = [
            ("grant_type", "password"),
            ("client_id", client.client_id.as_str()),
            ("username", user.username.as_str()),
            ("password", user.password.as_str()),
        ];

        let mut refused = base.to_vec();
        refused.push(("resource", OTHER));
        let refused = token(&server, app, &refused).await;
        assert_eq!(refused.status_code(), 400, "{}", refused.text());
        assert_eq!(refused.json::<Value>()["error"], "invalid_target");
        assert_eq!(
            sessions_of(app, &user.id).await,
            0,
            "a grant refused on its resource must not open a session"
        );

        let mut allowed = base.to_vec();
        allowed.push(("resource", ALLOWED));
        let granted = token(&server, app, &allowed).await;
        assert_eq!(granted.status_code(), 200, "{}", granted.text());
        assert_eq!(audience(&granted.json()), json!([ALLOWED]));
        assert_eq!(sessions_of(app, &user.id).await, 1);
    });
}
