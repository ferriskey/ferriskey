mod common;

use common::{
    ClientSpec, SeededClient, TestApp, app, decode_unverified, exchange_code, password_grant,
    refresh, rt, sign_in_for_code,
};
use serde_json::Value;
use uuid::Uuid;

const PASSWORD: &str = "Correct-horse-battery-7";

async fn stored_access_tokens(app: &TestApp, user_id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM access_tokens WHERE user_id = $1")
        .bind(Uuid::parse_str(user_id).expect("user id"))
        .fetch_one(&app.pool)
        .await
        .expect("count the stored access tokens")
}

async fn participants(app: &TestApp, session_id: Uuid) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM session_participants WHERE session_id = $1")
        .bind(session_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the session participants")
}

fn session_of(body: &Value) -> Uuid {
    let access_token = body["access_token"].as_str().expect("access_token");
    decode_unverified(access_token)["sid"]
        .as_str()
        .and_then(|sid| Uuid::parse_str(sid).ok())
        .expect("the access token names its session")
}

async fn login(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    user: &common::SeededUser,
) -> Value {
    let response = password_grant(
        server,
        app,
        &client.client_id,
        None,
        &user.username,
        &user.password,
    )
    .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json()
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test session_participants_test -- --ignored"]
fn a_password_grant_stores_no_access_token_and_one_session_participant() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::public().with_direct_access()).await;
        let user = app.user(&server, PASSWORD).await;

        let body = login(&server, app, &client, &user).await;

        let session_id = session_of(&body);
        assert_eq!(stored_access_tokens(app, &user.id).await, 0);
        let recorded: i64 = {
            let mut recorded = 0;
            for _ in 0..50 {
                recorded = participants(app, session_id).await;
                if recorded > 0 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            recorded
        };
        assert_eq!(recorded, 1);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test session_participants_test -- --ignored"]
fn a_refresh_stores_no_access_token_and_adds_no_session_participant() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::public().with_direct_access()).await;
        let user = app.user(&server, PASSWORD).await;
        let body = login(&server, app, &client, &user).await;
        let session_id = session_of(&body);
        let before = participants(app, session_id).await;

        let refreshed = refresh(
            &server,
            app,
            &client.client_id,
            None,
            body["refresh_token"].as_str().expect("refresh token"),
        )
        .await;

        assert_eq!(refreshed.status_code(), 200, "{}", refreshed.text());
        assert_eq!(stored_access_tokens(app, &user.id).await, 0);
        assert_eq!(participants(app, session_id).await, before);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test session_participants_test -- --ignored"]
fn an_access_token_is_refused_once_its_authorization_code_is_replayed() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::confidential()).await;
        let user = app.user(&server, PASSWORD).await;
        let code = sign_in_for_code(
            &server,
            app,
            &client.client_id,
            &user.username,
            PASSWORD,
            &[],
        )
        .await;
        let first = exchange_code(&server, app, &client, &code, &[]).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let access_token = first.json::<Value>()["access_token"]
            .as_str()
            .expect("access token")
            .to_string();
        let accepted = server
            .get(&app.oidc("userinfo"))
            .authorization_bearer(&access_token)
            .await;
        assert_eq!(accepted.status_code(), 200, "{}", accepted.text());

        let replay = exchange_code(&server, app, &client, &code, &[]).await;
        assert_eq!(replay.status_code(), 400, "{}", replay.text());

        let refused = server
            .get(&app.oidc("userinfo"))
            .authorization_bearer(&access_token)
            .await;
        assert_eq!(refused.status_code(), 401, "{}", refused.text());
    });
}
