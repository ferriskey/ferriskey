mod common;

use std::time::Duration;

use common::{ClientSpec, SeededClient, TestApp, app, rt};
use serde_json::Value;

const FAILURES_BEFORE_THROTTLE: usize = 10;

async fn grant(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    secret: &str,
) -> axum_test::TestResponse {
    server
        .post(&app.oidc("token"))
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", client.client_id.as_str()),
            ("client_secret", secret),
        ])
        .await
}

async fn flows_of(app: &TestApp, client: &SeededClient) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM compass_flows WHERE client_id = $1")
        .bind(&client.client_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the flows of the client")
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_endpoint_throttle_test -- --ignored"]
fn failed_client_authentications_write_no_flow_and_are_throttled_per_address() {
    let app = app();
    rt().block_on(async {
        let server = app.server_with_peer_address();
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;
        let secret = client
            .secret
            .clone()
            .expect("a confidential client has a secret");

        let accepted = grant(&server, app, &client, &secret).await;
        assert_eq!(accepted.status_code(), 200, "{}", accepted.text());
        tokio::time::sleep(Duration::from_millis(300)).await;
        let flows_after_one_grant = flows_of(app, &client).await;
        assert_eq!(
            flows_after_one_grant, 1,
            "an authenticated grant opens one flow"
        );

        for attempt in 1..=FAILURES_BEFORE_THROTTLE {
            let refused = grant(&server, app, &client, "not-the-secret").await;
            assert_eq!(
                refused.status_code(),
                401,
                "attempt {attempt}: {}",
                refused.text()
            );
            assert_eq!(refused.json::<Value>()["error"], "invalid_client");
        }

        let throttled = grant(&server, app, &client, "not-the-secret").await;
        assert_eq!(throttled.status_code(), 429, "{}", throttled.text());
        assert_eq!(throttled.header("retry-after"), "60");
        assert_eq!(
            throttled.json::<Value>()["error"],
            "temporarily_unavailable"
        );

        let right_secret_while_blocked = grant(&server, app, &client, &secret).await;
        assert_eq!(
            right_secret_while_blocked.status_code(),
            429,
            "the block holds for the address, whatever the secret"
        );

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            flows_of(app, &client).await,
            flows_after_one_grant,
            "a request refused on the client secret must not open a Compass flow"
        );
    });
}
