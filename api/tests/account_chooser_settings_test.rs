mod common;

use std::sync::Mutex;

use axum::http::HeaderValue;
use axum_test::TestServer;
use common::{ClientSpec, SeededClient, SeededUser, TestApp, cached_app, rt};
use serde_json::{Value, json};

const PASSWORD: &str = "Str0ng!P@ssword#2024";

static SERIAL: Mutex<()> = Mutex::new(());

fn bearer(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {token}")).expect("bearer header")
}

async fn put_settings(
    server: &TestServer,
    app: &TestApp,
    token: &str,
    body: Value,
) -> axum_test::TestResponse {
    server
        .put(&app.path("/settings"))
        .add_header("Authorization", bearer(token))
        .json(&body)
        .await
}

async fn login_settings(server: &TestServer, app: &TestApp) -> Value {
    let response = server.get(&app.path("/login-settings")).await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json()
}

async fn sign_in(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    user: &SeededUser,
) -> String {
    let authorize = server
        .get(&app.oidc("auth"))
        .add_query_param("response_type", "code")
        .add_query_param("client_id", &client.client_id)
        .add_query_param("redirect_uri", common::CALLBACK)
        .add_query_param("scope", "openid")
        .add_query_param("state", "chooser-settings")
        .await;
    assert_eq!(authorize.status_code(), 302, "{}", authorize.text());
    let session = authorize.cookie("FERRISKEY_SESSION").value().to_string();

    let response = server
        .post(&app.path("/login-actions/authenticate"))
        .add_header(
            "Cookie",
            HeaderValue::from_str(&format!("FERRISKEY_SESSION={session}")).expect("cookie"),
        )
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": user.username, "password": user.password }))
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.cookie("FERRISKEY_DEVICE").value().to_string()
}

async fn chooser_entries(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    device: &str,
) -> Vec<Value> {
    let response = server
        .get(&app.path("/login-actions/accounts"))
        .add_query_param("client_id", &client.client_id)
        .add_header(
            "Cookie",
            HeaderValue::from_str(&format!("FERRISKEY_DEVICE={device}")).expect("cookie"),
        )
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json::<Value>()["data"]
        .as_array()
        .expect("data array")
        .clone()
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_settings_test -- --ignored"]
fn the_chooser_flags_are_written_through_the_settings_api_and_only_the_toggle_is_public() {
    let _guard = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    rt().block_on(async {
        let app = cached_app();
        let server = app.server();
        let token = app.admin_token(&server).await;

        let reset = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": false, "account_chooser_show_email": false }),
        )
        .await;
        assert_eq!(reset.status_code(), 200, "{}", reset.text());
        let public = login_settings(&server, app).await;
        assert_eq!(public["account_chooser_enabled"], false);
        assert!(
            public.get("account_chooser_show_email").is_none(),
            "{public}"
        );

        let enabled = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": true }),
        )
        .await;
        assert_eq!(enabled.status_code(), 200, "{}", enabled.text());
        let settings = &enabled.json::<Value>()["data"]["settings"];
        assert_eq!(settings["account_chooser_enabled"], true);
        assert_eq!(settings["account_chooser_show_email"], false);

        let public = login_settings(&server, app).await;
        assert_eq!(public["account_chooser_enabled"], true);
        assert!(
            public.get("account_chooser_show_email").is_none(),
            "{public}"
        );

        let email = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_show_email": true }),
        )
        .await;
        assert_eq!(email.status_code(), 200, "{}", email.text());
        let settings = &email.json::<Value>()["data"]["settings"];
        assert_eq!(
            settings["account_chooser_enabled"], true,
            "absent leaves it unchanged"
        );
        assert_eq!(settings["account_chooser_show_email"], true);

        let unrelated =
            put_settings(&server, app, &token, json!({ "remember_me_enabled": true })).await;
        assert_eq!(unrelated.status_code(), 200, "{}", unrelated.text());
        let settings = &unrelated.json::<Value>()["data"]["settings"];
        assert_eq!(settings["account_chooser_enabled"], true);
        assert_eq!(settings["account_chooser_show_email"], true);

        let off = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": false, "account_chooser_show_email": false }),
        )
        .await;
        assert_eq!(off.status_code(), 200, "{}", off.text());
        assert_eq!(
            login_settings(&server, app).await["account_chooser_enabled"],
            false
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_settings_test -- --ignored"]
fn enabling_the_flag_through_the_api_takes_effect_on_the_cached_chooser() {
    let _guard = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    rt().block_on(async {
        let app = cached_app();
        let server = app.server();
        let token = app.admin_token(&server).await;
        let client = app.client(ClientSpec::public()).await;
        let user = app.user(&server, PASSWORD).await;

        let on = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": true }),
        )
        .await;
        assert_eq!(on.status_code(), 200, "{}", on.text());

        let device = sign_in(&server, app, &client, &user).await;

        let off = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": false }),
        )
        .await;
        assert_eq!(off.status_code(), 200, "{}", off.text());
        assert!(
            chooser_entries(&server, app, &client, &device)
                .await
                .is_empty(),
            "a disabled chooser lists nothing"
        );

        let on = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": true }),
        )
        .await;
        assert_eq!(on.status_code(), 200, "{}", on.text());
        assert_eq!(
            chooser_entries(&server, app, &client, &device).await.len(),
            1,
            "the cached realm settings were refreshed by the update"
        );

        let _ = put_settings(
            &server,
            app,
            &token,
            json!({ "account_chooser_enabled": false }),
        )
        .await;
    });
}
