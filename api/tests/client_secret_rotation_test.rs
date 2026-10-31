mod common;

use axum_test::TestServer;
use common::{ClientSpec, SeededClient, TestApp, cached_app, client_credentials, rt, sealed_app};
use serde_json::{Value, json};
use uuid::Uuid;

async fn confidential_client(
    server: &TestServer,
    app: &TestApp,
    token: &str,
) -> (Uuid, String, String) {
    let client_id = format!("rotating-{}", Uuid::new_v4().simple());
    let created = server
        .post(&app.path("/clients"))
        .authorization_bearer(token)
        .json(&json!({
            "client_id": client_id,
            "name": "rotating",
            "client_type": "confidential",
            "protocol": "openid-connect",
            "public_client": false,
            "service_account_enabled": true,
            "enabled": true,
        }))
        .await;
    assert_eq!(created.status_code(), 201, "{}", created.text());
    let body: Value = created.json();
    let id = body["id"]
        .as_str()
        .and_then(|id| id.parse().ok())
        .expect("client id");
    let secret = body["client_secret"]
        .as_str()
        .expect("the secret")
        .to_string();
    (id, client_id, secret)
}

async fn rotate(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    id: Uuid,
    body: Value,
) -> axum_test::TestResponse {
    server
        .post(&app.path(&format!("/clients/{id}/client-secret/rotate")))
        .authorization_bearer(token)
        .json(&body)
        .await
}

async fn status_with(server: &TestServer, app: &TestApp, client_id: &str, secret: &str) -> u16 {
    client_credentials(
        server,
        app,
        &SeededClient {
            id: Uuid::nil(),
            client_id: client_id.to_string(),
            secret: Some(secret.to_string()),
        },
    )
    .await
    .status_code()
    .as_u16()
}

async fn events_of(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    event_type: &str,
    id: Uuid,
) -> usize {
    let response = server
        .get(&format!(
            "/realms/{}/seawatch/v1/security-events?event_types={event_type}&limit=100",
            app.realm
        ))
        .authorization_bearer(token)
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json::<Value>()["data"]
        .as_array()
        .expect("events")
        .iter()
        .filter(|event| event["target_id"] == id.to_string().as_str())
        .count()
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn both_secrets_work_during_the_grace_period_and_only_the_new_one_after_a_revocation() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;
        let (id, client_id, first) = confidential_client(&server, app, &token).await;

        let rotated = rotate(
            &server,
            app,
            &token,
            id,
            json!({ "grace_period_secs": 3_600 }),
        )
        .await;
        assert_eq!(rotated.status_code(), 200, "{}", rotated.text());
        let body: Value = rotated.json();
        let second = body["client_secret"]
            .as_str()
            .expect("the new secret")
            .to_string();
        assert_ne!(second, first);
        assert!(body["previous_secret_expires_at"].is_string(), "{body}");

        assert_eq!(status_with(&server, app, &client_id, &second).await, 200);
        assert_eq!(status_with(&server, app, &client_id, &first).await, 200);
        assert_eq!(status_with(&server, app, &client_id, "neither").await, 401);

        let revealed: Value = server
            .get(&app.path(&format!("/clients/{id}/client-secret")))
            .authorization_bearer(&token)
            .await
            .json();
        assert_eq!(revealed["client_secret"], second.as_str());
        assert_eq!(revealed["previous_secret"]["client_secret"], first.as_str());
        assert!(
            revealed["previous_secret"]["expires_at"].is_string(),
            "{revealed}"
        );

        let listed: Value = server
            .get(&app.path(&format!("/clients/{id}")))
            .authorization_bearer(&token)
            .await
            .json();
        let shown = &listed["data"]["previous_secret"];
        assert_eq!(shown["secret"], "***", "{listed}");
        assert!(shown["expires_at"].is_string(), "{listed}");

        for column in ["secret", "previous_secret"] {
            let at_rest: String =
                sqlx::query_scalar(&format!("SELECT {column} FROM clients WHERE id = $1"))
                    .bind(id)
                    .fetch_one(&app.pool)
                    .await
                    .expect("read the stored value");
            assert!(at_rest.starts_with("fk1:kid-a:"), "{column}: {at_rest}");
        }

        let revoked = server
            .delete(&app.path(&format!("/clients/{id}/client-secret/previous")))
            .authorization_bearer(&token)
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());
        assert_eq!(status_with(&server, app, &client_id, &first).await, 401);
        assert_eq!(status_with(&server, app, &client_id, &second).await, 200);

        let again = server
            .delete(&app.path(&format!("/clients/{id}/client-secret/previous")))
            .authorization_bearer(&token)
            .await;
        assert_eq!(again.status_code(), 404, "{}", again.text());

        assert_eq!(
            events_of(&server, app, &token, "client_secret_rotated", id).await,
            1
        );
        assert_eq!(
            events_of(&server, app, &token, "client_secret_revoked", id).await,
            1
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn the_old_secret_stops_working_when_its_grace_period_ends_without_waiting_for_the_sweep() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;
        let (id, client_id, first) = confidential_client(&server, app, &token).await;
        let rotated = rotate(&server, app, &token, id, json!({ "grace_period_secs": 3_600 })).await;
        let second = rotated.json::<Value>()["client_secret"]
            .as_str()
            .expect("the new secret")
            .to_string();
        assert_eq!(status_with(&server, app, &client_id, &first).await, 200);

        sqlx::query(
            "UPDATE clients SET previous_secret_expires_at = (NOW() AT TIME ZONE 'UTC') - interval '1 second' WHERE id = $1",
        )
        .bind(id)
        .execute(&app.pool)
        .await
        .expect("end the grace period");

        assert_eq!(status_with(&server, app, &client_id, &first).await, 401);
        assert_eq!(status_with(&server, app, &client_id, &second).await, 200);

        let revealed: Value = server
            .get(&app.path(&format!("/clients/{id}/client-secret")))
            .authorization_bearer(&token)
            .await
            .json();
        assert!(revealed["previous_secret"].is_null(), "{revealed}");
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn rotating_without_a_grace_period_cuts_the_old_secret_even_through_the_client_cache() {
    let app = cached_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;
        let (id, client_id, first) = confidential_client(&server, app, &token).await;
        assert_eq!(status_with(&server, app, &client_id, &first).await, 200);

        let rotated = rotate(&server, app, &token, id, json!({})).await;
        assert_eq!(rotated.status_code(), 200, "{}", rotated.text());
        let body: Value = rotated.json();
        assert!(body["previous_secret_expires_at"].is_null(), "{body}");
        let second = body["client_secret"].as_str().expect("the new secret");

        assert_eq!(status_with(&server, app, &client_id, &first).await, 401);
        assert_eq!(status_with(&server, app, &client_id, second).await, 200);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn rotating_twice_in_a_row_keeps_only_the_latest_previous_secret() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;
        let (id, client_id, first) = confidential_client(&server, app, &token).await;
        let grace = json!({ "grace_period_secs": 3_600 });
        let second = rotate(&server, app, &token, id, grace.clone())
            .await
            .json::<Value>()["client_secret"]
            .as_str()
            .expect("second secret")
            .to_string();
        let third = rotate(&server, app, &token, id, grace)
            .await
            .json::<Value>()["client_secret"]
            .as_str()
            .expect("third secret")
            .to_string();

        assert_eq!(status_with(&server, app, &client_id, &first).await, 401);
        assert_eq!(status_with(&server, app, &client_id, &second).await, 200);
        assert_eq!(status_with(&server, app, &client_id, &third).await, 200);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn a_public_client_and_a_bad_grace_period_are_refused() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;

        let public = app.client(ClientSpec::public()).await;
        let refused = rotate(&server, app, &token, public.id, json!({})).await;
        assert_eq!(refused.status_code(), 400, "{}", refused.text());
        assert_eq!(refused.json::<Value>()["reason"], "client_has_no_secret");

        let (id, client_id, first) = confidential_client(&server, app, &token).await;
        for secs in [-1_i64, 2_592_001] {
            let refused = rotate(
                &server,
                app,
                &token,
                id,
                json!({ "grace_period_secs": secs }),
            )
            .await;
            assert_eq!(refused.status_code(), 400, "{secs}: {}", refused.text());
            assert_eq!(
                refused.json::<Value>()["reason"],
                "invalid_secret_grace_period"
            );
        }
        assert_eq!(status_with(&server, app, &client_id, &first).await, 200);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_secret_rotation_test -- --ignored"]
fn rotation_needs_a_token() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;
        let (id, _, _) = confidential_client(&server, app, &token).await;

        let anonymous = server
            .post(&app.path(&format!("/clients/{id}/client-secret/rotate")))
            .json(&json!({}))
            .await;

        assert_eq!(anonymous.status_code(), 401, "{}", anonymous.text());
    });
}
