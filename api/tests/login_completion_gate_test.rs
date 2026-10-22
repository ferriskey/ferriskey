mod common;

use axum::http::HeaderValue;
use common::{ClientSpec, SeededClient, SeededUser, TestApp, app, authorize, query_param, rt};
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha1::Sha1;
use uuid::Uuid;

const PASSWORD: &str = "Str0ng!P@ssword#2024";
const OTP_SECRET_BASE32: &str = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";

fn base32_decode(input: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut bits: u32 = 0;
    let mut held = 0;
    let mut out = Vec::new();
    for symbol in input.bytes() {
        let value = ALPHABET
            .iter()
            .position(|candidate| *candidate == symbol)
            .expect("base32 symbol") as u32;
        bits = (bits << 5) | value;
        held += 5;
        if held >= 8 {
            held -= 8;
            out.push(((bits >> held) & 0xff) as u8);
        }
    }
    out
}

fn current_totp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    let mut mac = Hmac::<Sha1>::new_from_slice(&base32_decode(OTP_SECRET_BASE32))
        .expect("hmac accepts any key length");
    mac.update(&(now / 30).to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = (digest[19] & 0x0f) as usize;
    let code = ((u32::from(digest[offset]) & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);
    format!("{:06}", code % 1_000_000)
}

async fn user_with_otp(app: &TestApp, server: &axum_test::TestServer) -> SeededUser {
    let user = app.user(server, PASSWORD).await;
    sqlx::query(
        "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, user_label)
         VALUES ($1, 'otp', $2::uuid, $3, '{}'::jsonb, 'test-authenticator')",
    )
    .bind(Uuid::new_v4())
    .bind(&user.id)
    .bind(OTP_SECRET_BASE32)
    .execute(&app.pool)
    .await
    .expect("insert otp credential");
    user
}

async fn flag_client(app: &TestApp, client: &SeededClient, assignment: &str) {
    sqlx::query(&format!("UPDATE clients SET {assignment} WHERE id = $1"))
        .bind(client.id)
        .execute(&app.pool)
        .await
        .expect("flag the client");
}

struct OtpStep {
    session_cookie: axum_test::http::HeaderValue,
    cookies: String,
}

async fn reach_the_otp_step(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    user: &SeededUser,
) -> OtpStep {
    let authorized = authorize(server, app, &client.client_id, &[]).await;
    assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
    let session = authorized.cookie("FERRISKEY_SESSION");
    let session_id = session.value().to_string();

    let login = server
        .post(&app.path("/login-actions/authenticate"))
        .add_cookie(session)
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": user.username, "password": user.password }))
        .await;
    assert_eq!(login.status_code(), 200, "{}", login.text());
    let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

    OtpStep {
        session_cookie: HeaderValue::from_str(&format!("FERRISKEY_SESSION={session_id}"))
            .expect("cookie header"),
        cookies: format!("FERRISKEY_SESSION={session_id}; FERRISKEY_LOGIN_ACTION={step_token}"),
    }
}

async fn challenge_otp(
    server: &axum_test::TestServer,
    app: &TestApp,
    step: &OtpStep,
) -> axum_test::TestResponse {
    server
        .post(&app.path("/login-actions/challenge-otp"))
        .add_header(
            "Cookie",
            HeaderValue::from_str(&step.cookies).expect("cookies"),
        )
        .json(&json!({ "code": current_totp() }))
        .await
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test login_completion_gate_test -- --ignored"]
fn an_otp_login_to_a_client_requiring_consent_goes_through_the_consent_screen() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = user_with_otp(app, &server).await;
        let client = app.client(ClientSpec::public()).await;
        flag_client(app, &client, "registration_source = 'dynamic'").await;

        let step = reach_the_otp_step(&server, app, &client, &user).await;
        let challenged = challenge_otp(&server, app, &step).await;
        assert_eq!(challenged.status_code(), 200, "{}", challenged.text());

        let body: Value = challenged.json();
        let url = body["url"].as_str().expect("redirect url");
        assert!(
            url.contains("authentication/consent"),
            "a second factor must not skip the consent screen: {body}"
        );
        assert!(
            !url.contains("code="),
            "no authorization code may be issued before consent: {body}"
        );
        assert!(
            challenged.maybe_cookie("FERRISKEY_SSO").is_none(),
            "no SSO session is opened before consent"
        );

        let consent_token = query_param(url, "consent_token").expect("consent token");
        let decision = server
            .post(&app.path("/auth/consent"))
            .add_header("Cookie", step.session_cookie.clone())
            .json(&json!({ "consent_token": consent_token, "approved_scopes": [] }))
            .await;
        assert_eq!(decision.status_code(), 200, "{}", decision.text());
        let decision: Value = decision.json();
        assert!(
            decision["redirect_url"]
                .as_str()
                .is_some_and(|url| url.contains("code=")),
            "approving the consent must then issue the code: {decision}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test login_completion_gate_test -- --ignored"]
fn an_otp_login_to_a_client_put_in_maintenance_meanwhile_is_refused() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = user_with_otp(app, &server).await;
        let client = app.client(ClientSpec::public()).await;

        let step = reach_the_otp_step(&server, app, &client, &user).await;
        flag_client(app, &client, "maintenance_enabled = true").await;
        let challenged = challenge_otp(&server, app, &step).await;

        assert_ne!(challenged.status_code(), 200, "{}", challenged.text());
        assert!(
            !challenged.text().contains("code="),
            "a refused login must not hand out a code: {}",
            challenged.text()
        );
    });
}
