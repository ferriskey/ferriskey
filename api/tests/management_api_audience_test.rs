mod common;

use common::{ClientSpec, SeededClient, TestApp, app, password_grant, rt};
use serde_json::Value;

const PASSWORD: &str = "Str0ng!P@ssword#2024";

async fn token_of(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    username: &str,
) -> String {
    let granted = password_grant(server, app, &client.client_id, None, username, PASSWORD).await;
    assert_eq!(granted.status_code(), 200, "{}", granted.text());
    granted.json::<Value>()["access_token"]
        .as_str()
        .expect("access token")
        .to_string()
}

async fn client_registered_as(app: &TestApp, source: &str) -> SeededClient {
    let client = app.client(ClientSpec::public().with_direct_access()).await;
    sqlx::query("UPDATE clients SET registration_source = $2 WHERE id = $1")
        .bind(client.id)
        .bind(source)
        .execute(&app.pool)
        .await
        .expect("set the registration source");
    client
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test management_api_audience_test -- --ignored"]
fn a_token_of_an_admin_registered_client_reaches_the_management_api() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        let client = client_registered_as(app, "admin").await;
        let token = token_of(&server, app, &client, &user.username).await;

        let profile = server
            .get(&app.path("/users/me"))
            .authorization_bearer(&token)
            .await;

        assert_eq!(profile.status_code(), 200, "{}", profile.text());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test management_api_audience_test -- --ignored"]
fn a_token_of_a_client_that_registered_itself_cannot_reach_the_management_api() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;

        for source in ["dynamic", "metadata_document"] {
            let client = client_registered_as(app, source).await;
            let token = token_of(&server, app, &client, &user.username).await;

            let profile = server
                .get(&app.path("/users/me"))
                .authorization_bearer(&token)
                .await;

            assert_eq!(profile.status_code(), 401, "{source}: {}", profile.text());
        }
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test management_api_audience_test -- --ignored"]
fn a_token_of_a_client_that_registered_itself_can_still_call_userinfo() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        let client = client_registered_as(app, "dynamic").await;
        let token = token_of(&server, app, &client, &user.username).await;

        let userinfo = server
            .get(&app.oidc("userinfo"))
            .authorization_bearer(&token)
            .await;

        assert_eq!(userinfo.status_code(), 200, "{}", userinfo.text());
    });
}
