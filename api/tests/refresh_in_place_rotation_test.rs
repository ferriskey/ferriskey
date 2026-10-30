mod common;

use axum_test::{TestResponse, TestServer};
use common::{
    ClientSpec, SeededClient, SeededUser, TestApp, app, opaque_app, password_grant, refresh, rt,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::task::JoinSet;
use uuid::Uuid;

const PASSWORD: &str = "Correct-horse-battery-7";

struct Login {
    client: SeededClient,
    user: SeededUser,
    refresh_token: String,
}

async fn login(app: &TestApp, server: &TestServer) -> Login {
    let client = app.client(ClientSpec::public().with_direct_access()).await;
    let user = app.user(server, PASSWORD).await;
    let granted = password_grant(
        server,
        app,
        &client.client_id,
        None,
        &user.username,
        &user.password,
    )
    .await;
    assert_eq!(granted.status_code(), 200, "{}", granted.text());
    let refresh_token = token_of(&granted);
    Login {
        client,
        user,
        refresh_token,
    }
}

fn token_of(response: &TestResponse) -> String {
    response.json::<Value>()["refresh_token"]
        .as_str()
        .expect("refresh token")
        .to_string()
}

async fn refresh_with(
    app: &TestApp,
    server: &TestServer,
    login: &Login,
    token: &str,
) -> TestResponse {
    refresh(server, app, &login.client.client_id, None, token).await
}

fn assert_invalid_grant(response: &TestResponse) {
    assert_eq!(response.status_code(), 400, "{}", response.text());
    assert_eq!(
        response.json::<Value>()["error"],
        "invalid_grant",
        "{}",
        response.text()
    );
}

async fn rows_of(app: &TestApp, user: &SeededUser) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE user_id = $1")
        .bind(Uuid::parse_str(&user.id).expect("user id"))
        .fetch_one(&app.pool)
        .await
        .expect("count the user's refresh tokens")
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_22_an_opaque_refresh_inserts_no_row_and_stores_the_new_hash_in_place() {
    let app = opaque_app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let before = rows_of(app, &login.user).await;
        assert_eq!(before, 1);

        let first = refresh_with(app, &server, &login, &login.refresh_token).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let second_token = token_of(&first);
        let second = refresh_with(app, &server, &login, &second_token).await;
        assert_eq!(second.status_code(), 200, "{}", second.text());
        let third_token = token_of(&second);

        assert_eq!(rows_of(app, &login.user).await, before);
        let stored: Vec<u8> =
            sqlx::query_scalar("SELECT token_hash FROM refresh_tokens WHERE user_id = $1")
                .bind(Uuid::parse_str(&login.user.id).expect("user id"))
                .fetch_one(&app.pool)
                .await
                .expect("read the hash");
        assert_eq!(stored, Sha256::digest(third_token.as_bytes()).to_vec());
        let previous: Vec<u8> =
            sqlx::query_scalar("SELECT previous_token_hash FROM refresh_tokens WHERE user_id = $1")
                .bind(Uuid::parse_str(&login.user.id).expect("user id"))
                .fetch_one(&app.pool)
                .await
                .expect("read the previous hash");
        assert_eq!(previous, Sha256::digest(second_token.as_bytes()).to_vec());
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_22_a_legacy_refresh_inserts_no_row() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let before = rows_of(app, &login.user).await;

        let first = refresh_with(app, &server, &login, &login.refresh_token).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let second = refresh_with(app, &server, &login, &token_of(&first)).await;
        assert_eq!(second.status_code(), 200, "{}", second.text());

        assert_eq!(rows_of(app, &login.user).await, before);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_23_replaying_the_previous_legacy_token_revokes_the_family() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;
        let first = refresh_with(app, &server, &login, &login.refresh_token).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let current = token_of(&first);

        let replay = refresh_with(app, &server, &login, &login.refresh_token).await;
        assert_invalid_grant(&replay);

        let current_try = refresh_with(app, &server, &login, &current).await;
        assert_invalid_grant(&current_try);
    });
}

#[test]
#[ignore = "requires PostgreSQL"]
fn spec_rtp_24_concurrent_refreshes_of_one_legacy_token_yield_exactly_one_success() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let login = login(app, &server).await;

        let mut attempts = JoinSet::new();
        for _ in 0..8 {
            let server = app.server();
            let client_id = login.client.client_id.clone();
            let token = login.refresh_token.clone();
            attempts.spawn(async move {
                refresh(&server, app, &client_id, None, &token)
                    .await
                    .status_code()
                    .as_u16()
            });
        }
        let mut statuses = Vec::new();
        while let Some(status) = attempts.join_next().await {
            statuses.push(status.expect("join"));
        }

        assert_eq!(
            statuses.iter().filter(|status| **status == 200).count(),
            1,
            "{statuses:?}"
        );
        assert_eq!(rows_of(app, &login.user).await, 1);
    });
}

async fn a_token_two_generations_old_is_refused_and_the_family_survives(app: &'static TestApp) {
    let server = app.server();
    let login = login(app, &server).await;
    let first = refresh_with(app, &server, &login, &login.refresh_token).await;
    assert_eq!(first.status_code(), 200, "{}", first.text());
    let second = refresh_with(app, &server, &login, &token_of(&first)).await;
    assert_eq!(second.status_code(), 200, "{}", second.text());
    let current = token_of(&second);

    let stale = refresh_with(app, &server, &login, &login.refresh_token).await;
    assert_invalid_grant(&stale);

    let still_usable = refresh_with(app, &server, &login, &current).await;
    assert_eq!(still_usable.status_code(), 200, "{}", still_usable.text());
}

#[test]
#[ignore = "requires PostgreSQL"]
fn a_token_two_generations_old_is_refused_and_the_opaque_family_survives() {
    rt().block_on(a_token_two_generations_old_is_refused_and_the_family_survives(opaque_app()));
}

#[test]
#[ignore = "requires PostgreSQL"]
fn a_token_two_generations_old_is_refused_and_the_legacy_family_survives() {
    rt().block_on(a_token_two_generations_old_is_refused_and_the_family_survives(app()));
}
