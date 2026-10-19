mod common;

use std::time::Duration;

use common::{ClientSpec, SeededClient, TestApp, forwarded_app, rt};
use serde_json::Value;
use uuid::Uuid;

const FAILURES_BEFORE_THROTTLE: usize = 10;
const DEVICE_CODE_GRANT: &str = "urn:ietf:params:oauth:grant-type:device_code";

async fn token(
    server: &axum_test::TestServer,
    app: &TestApp,
    address: &str,
    form: &[(&str, &str)],
) -> axum_test::TestResponse {
    server
        .post(&app.oidc("token"))
        .add_header("x-forwarded-for", address)
        .form(form)
        .await
}

async fn client_credentials(
    server: &axum_test::TestServer,
    app: &TestApp,
    address: &str,
    client: &SeededClient,
    secret: &str,
) -> axum_test::TestResponse {
    token(
        server,
        app,
        address,
        &[
            ("grant_type", "client_credentials"),
            ("client_id", client.client_id.as_str()),
            ("client_secret", secret),
        ],
    )
    .await
}

async fn service_client(server: &axum_test::TestServer, app: &TestApp) -> (SeededClient, String) {
    let client = app
        .client(ClientSpec::confidential().with_service_account())
        .await;
    app.bind_service_account(server, &client).await;
    let secret = client
        .secret
        .clone()
        .expect("a confidential client has a secret");
    (client, secret)
}

async fn flows_of(app: &TestApp, client: &SeededClient) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM compass_flows WHERE client_id = $1")
        .bind(&client.client_id)
        .fetch_one(&app.pool)
        .await
        .expect("count the flows of the client")
}

fn assert_throttled(response: &axum_test::TestResponse) {
    assert_eq!(response.status_code(), 429, "{}", response.text());
    assert_eq!(response.header("retry-after"), "60");
    assert_eq!(response.json::<Value>()["error"], "temporarily_unavailable");
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_endpoint_throttle_test -- --ignored"]
fn failed_client_authentications_write_no_flow_and_are_throttled_per_address() {
    let app = forwarded_app();
    rt().block_on(async {
        let server = app.server();
        let address = "203.0.113.10";
        let (client, secret) = service_client(&server, app).await;

        let accepted = client_credentials(&server, app, address, &client, &secret).await;
        assert_eq!(accepted.status_code(), 200, "{}", accepted.text());
        tokio::time::sleep(Duration::from_millis(300)).await;
        let flows_after_one_grant = flows_of(app, &client).await;
        assert_eq!(
            flows_after_one_grant, 1,
            "an authenticated grant opens one flow"
        );

        for attempt in 1..=FAILURES_BEFORE_THROTTLE {
            let refused =
                client_credentials(&server, app, address, &client, "not-the-secret").await;
            assert_eq!(
                refused.status_code(),
                401,
                "attempt {attempt}: {}",
                refused.text()
            );
            assert_eq!(refused.json::<Value>()["error"], "invalid_client");
        }

        assert_throttled(
            &client_credentials(&server, app, address, &client, "not-the-secret").await,
        );
        assert_throttled(&client_credentials(&server, app, address, &client, &secret).await);

        let elsewhere = client_credentials(&server, app, "203.0.113.11", &client, &secret).await;
        assert_eq!(
            elsewhere.status_code(),
            200,
            "another address is not blocked: {}",
            elsewhere.text()
        );

        tokio::time::sleep(Duration::from_millis(300)).await;
        assert_eq!(
            flows_of(app, &client).await,
            flows_after_one_grant + 1,
            "a request refused on the client secret must not open a Compass flow"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_endpoint_throttle_test -- --ignored"]
fn successful_requests_are_never_throttled() {
    let app = forwarded_app();
    rt().block_on(async {
        let server = app.server();
        let address = "203.0.113.20";
        let (client, secret) = service_client(&server, app).await;

        for request in 1..=(FAILURES_BEFORE_THROTTLE * 3) {
            let granted = client_credentials(&server, app, address, &client, &secret).await;
            assert_eq!(
                granted.status_code(),
                200,
                "request {request}: {}",
                granted.text()
            );
        }
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test token_endpoint_throttle_test -- --ignored"]
fn device_code_polls_with_a_wrong_secret_count_as_failures() {
    let app = forwarded_app();
    rt().block_on(async {
        let server = app.server();
        let address = "203.0.113.30";
        let (client, secret) = service_client(&server, app).await;

        let device_code = Uuid::new_v4().to_string();
        for attempt in 1..=FAILURES_BEFORE_THROTTLE {
            let refused = token(
                &server,
                app,
                address,
                &[
                    ("grant_type", DEVICE_CODE_GRANT),
                    ("device_code", device_code.as_str()),
                    ("client_id", client.client_id.as_str()),
                    ("client_secret", "not-the-secret"),
                ],
            )
            .await;
            assert_eq!(
                refused.json::<Value>()["error"],
                "invalid_client",
                "attempt {attempt}: {}",
                refused.text()
            );
        }

        assert_throttled(&client_credentials(&server, app, address, &client, &secret).await);
    });
}
