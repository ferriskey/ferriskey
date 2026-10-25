mod common;

use common::{ClientSpec, SeededClient, TestApp, app, decode_unverified, introspect, rt};
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
            0,
            "an issued client_credentials token is not stored"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_credentials_test -- --ignored"]
fn a_client_credentials_token_can_be_revoked_though_it_was_never_stored() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;

        let body = grant(&server, app, &client, None).await;
        let token = body["access_token"].as_str().expect("access_token");
        let service_account = subject(&body);

        assert_eq!(
            introspect(&server, app, &client, token).await["active"],
            true
        );

        let revoked = server
            .post(&app.oidc("revoke"))
            .form(&[
                ("client_id", client.client_id.as_str()),
                (
                    "client_secret",
                    client.secret.as_deref().unwrap_or_default(),
                ),
                ("token", token),
            ])
            .await;
        assert_eq!(revoked.status_code(), 200, "{}", revoked.text());

        assert_eq!(
            introspect(&server, app, &client, token).await["active"],
            false
        );
        assert_eq!(
            stored_tokens(app, "access_tokens", service_account).await,
            1
        );

        let other = grant(&server, app, &client, None).await;
        let other = other["access_token"].as_str().expect("access_token");
        assert_eq!(
            introspect(&server, app, &client, other).await["active"],
            true,
            "revoking one token leaves the others valid"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test client_credentials_test -- --ignored"]
fn an_id_token_comes_only_with_the_openid_scope() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;

        let without_scope = grant(&server, app, &client, None).await;
        let without_openid = grant(&server, app, &client, Some("profile email")).await;
        let with_openid = grant(&server, app, &client, Some("openid")).await;

        assert!(
            without_scope.get("id_token").is_none(),
            "no scope asked, no ID token: {without_scope}"
        );
        assert!(
            without_openid.get("id_token").is_none(),
            "openid not asked, no ID token: {without_openid}"
        );
        let id_token = with_openid["id_token"]
            .as_str()
            .expect("openid asked, an ID token comes back");
        assert_eq!(
            decode_unverified(id_token)["sub"],
            Value::String(subject(&with_openid).to_string())
        );
    });
}

async fn traced_flows(app: &TestApp, client: &SeededClient) -> Vec<String> {
    sqlx::query_scalar("SELECT status::text FROM compass_flows WHERE client_id = $1")
        .bind(&client.client_id)
        .fetch_all(&app.pool)
        .await
        .expect("read compass flows")
}

#[test]
fn only_a_failed_client_credentials_grant_is_traced() {
    let app = app();

    rt().block_on(async {
        sqlx::query("UPDATE realm_settings SET compass_enabled = true WHERE realm_id = $1")
            .bind(app.realm_id)
            .execute(&app.pool)
            .await
            .expect("enable compass");
        let server = app.server();

        let served = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &served).await;
        grant(&server, app, &served, None).await;

        let unbound = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        let refused = server
            .post(&app.oidc("token"))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", unbound.client_id.as_str()),
                (
                    "client_secret",
                    unbound.secret.as_deref().unwrap_or_default(),
                ),
            ])
            .await;
        assert!(refused.status_code().is_client_error());

        let mut failed = Vec::new();
        for _ in 0..40 {
            failed = traced_flows(app, &unbound).await;
            if !failed.is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
        assert_eq!(failed.len(), 1, "the refused grant leaves one flow");
        assert!(failed[0].eq_ignore_ascii_case("failure"), "{failed:?}");
        assert!(
            traced_flows(app, &served).await.is_empty(),
            "a served grant leaves no flow"
        );
    });
}
