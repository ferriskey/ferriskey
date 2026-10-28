mod common;

use axum_test::TestServer;
use common::{
    ClientSpec, SeededClient, SeededUser, TestApp, cached_app, introspect, password_grant, rt,
};
use serde_json::{Value, json};

const PASSWORD: &str = "Correct-horse-battery-7";

struct Session {
    access_token: String,
    user: SeededUser,
    client: SeededClient,
}

async fn signed_in(server: &TestServer, app: &TestApp) -> Session {
    let client = app
        .client(ClientSpec::confidential().with_direct_access())
        .await;
    let user = app.user(server, PASSWORD).await;
    let granted = password_grant(
        server,
        app,
        &client.client_id,
        client.secret.as_deref(),
        &user.username,
        &user.password,
    )
    .await;
    assert_eq!(granted.status_code(), 200, "{}", granted.text());
    let access_token = granted.json::<Value>()["access_token"]
        .as_str()
        .expect("access token")
        .to_string();

    Session {
        access_token,
        user,
        client,
    }
}

async fn userinfo_status(server: &TestServer, app: &TestApp, token: &str) -> u16 {
    server
        .get(&app.oidc("userinfo"))
        .authorization_bearer(token)
        .await
        .status_code()
        .as_u16()
}

async fn warm(server: &TestServer, app: &TestApp, session: &Session) {
    for _ in 0..2 {
        assert_eq!(
            userinfo_status(server, app, &session.access_token).await,
            200
        );
    }
    let active = introspect(server, app, &session.client, &session.access_token).await;
    assert_eq!(active["active"], json!(true), "{active}");
}

async fn assert_dead(server: &TestServer, app: &TestApp, session: &Session) {
    assert_eq!(
        userinfo_status(server, app, &session.access_token).await,
        401,
        "userinfo still accepts a token that was ended"
    );
    let introspected = introspect(server, app, &session.client, &session.access_token).await;
    assert_eq!(
        introspected["active"],
        json!(false),
        "introspection still reports a token that was ended: {introspected}"
    );
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_state_cache_test -- --ignored"]
fn revoking_a_session_ends_its_tokens_while_sessions_are_cached() {
    let app = cached_app();
    rt().block_on(async {
        let server = app.server();
        let session = signed_in(&server, app).await;
        warm(&server, app, &session).await;
        let admin = app.admin_token(&server).await;

        let sessions = server
            .get(&app.path(&format!("/users/{}/sessions", session.user.id)))
            .authorization_bearer(&admin)
            .await;
        let session_id = sessions.json::<Value>()["data"][0]["id"]
            .as_str()
            .expect("session id")
            .to_string();
        let revoked = server
            .delete(&app.path(&format!("/users/{}/sessions/{session_id}", session.user.id)))
            .authorization_bearer(&admin)
            .await;
        assert_eq!(revoked.status_code(), 204, "{}", revoked.text());

        assert_dead(&server, app, &session).await;
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_state_cache_test -- --ignored"]
fn revoking_a_token_ends_it_while_revocations_are_cached() {
    let app = cached_app();
    rt().block_on(async {
        let server = app.server();
        let session = signed_in(&server, app).await;
        warm(&server, app, &session).await;

        let revoked = server
            .post(&app.oidc("revoke"))
            .form(&[
                ("token", session.access_token.as_str()),
                ("client_id", session.client.client_id.as_str()),
                (
                    "client_secret",
                    session.client.secret.as_deref().unwrap_or_default(),
                ),
            ])
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());

        assert_dead(&server, app, &session).await;
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_state_cache_test -- --ignored"]
fn resetting_the_password_ends_the_tokens_while_state_is_cached() {
    let app = cached_app();
    rt().block_on(async {
        let server = app.server();
        let session = signed_in(&server, app).await;
        warm(&server, app, &session).await;
        let admin = app.admin_token(&server).await;

        let reset = server
            .put(&app.path(&format!("/users/{}/reset-password", session.user.id)))
            .authorization_bearer(&admin)
            .json(&json!({
                "value": "Another-horse-battery-9",
                "temporary": false,
                "credential_type": "password",
            }))
            .await;
        assert_eq!(reset.status_code(), 200, "{}", reset.text());

        assert_dead(&server, app, &session).await;
    });
}
