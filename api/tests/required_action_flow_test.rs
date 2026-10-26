mod common;

use axum::http::HeaderValue;
use common::{ClientSpec, SeededClient, SeededUser, TestApp, app, authorize, rt};
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

fn current_totp(secret_base32: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    let mut mac = Hmac::<Sha1>::new_from_slice(&base32_decode(secret_base32))
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

async fn give_otp(app: &TestApp, user: &SeededUser) {
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
}

async fn require(app: &TestApp, user: &SeededUser, action: &str) {
    sqlx::query(
        "INSERT INTO user_required_actions (id, user_id, action) VALUES ($1, $2::uuid, $3)",
    )
    .bind(Uuid::new_v4())
    .bind(&user.id)
    .bind(action)
    .execute(&app.pool)
    .await
    .expect("insert required action");
}

/// The cookies a browser holds between two steps of the same login.
struct Login {
    session_id: String,
    step_token: String,
}

impl Login {
    fn cookies(&self) -> HeaderValue {
        HeaderValue::from_str(&format!(
            "FERRISKEY_SESSION={}; FERRISKEY_LOGIN_ACTION={}",
            self.session_id, self.step_token
        ))
        .expect("cookie header")
    }

    fn follow(&mut self, response: &axum_test::TestResponse) {
        self.step_token = response
            .maybe_cookie("FERRISKEY_LOGIN_ACTION")
            .expect("a step with actions left must hand over a new step token")
            .value()
            .to_string();
    }
}

async fn sign_in(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &SeededClient,
    user: &SeededUser,
) -> (Login, Value) {
    let authorized = authorize(server, app, &client.client_id, &[]).await;
    assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
    let session = authorized.cookie("FERRISKEY_SESSION");
    let session_id = session.value().to_string();

    let response = server
        .post(&app.path("/login-actions/authenticate"))
        .add_cookie(session)
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": user.username, "password": user.password }))
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());

    let login = Login {
        session_id,
        step_token: response
            .cookie("FERRISKEY_LOGIN_ACTION")
            .value()
            .to_string(),
    };
    (login, response.json())
}

async fn post(
    server: &axum_test::TestServer,
    app: &TestApp,
    login: &Login,
    action: &str,
    body: Value,
) -> axum_test::TestResponse {
    server
        .post(&app.path(&format!("/login-actions/{action}")))
        .add_header("Cookie", login.cookies())
        .json(&body)
        .await
}

fn redirect_with_code(body: &Value) -> bool {
    body["url"]
        .as_str()
        .is_some_and(|url| url.contains("code="))
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test required_action_flow_test -- --ignored"]
fn an_otp_user_owing_a_required_action_answers_the_otp_first() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        give_otp(app, &user).await;
        require(app, &user, "configure_passkey").await;
        let client = app.client(ClientSpec::public()).await;

        let (mut login, body) = sign_in(&server, app, &client, &user).await;
        assert_eq!(body["status"], "RequiresOtpChallenge", "{body}");

        let early = post(
            &server,
            app,
            &login,
            "webauthn-public-key-create-options",
            json!({}),
        )
        .await;
        assert_eq!(
            early.status_code(),
            403,
            "the password alone must not reach a required action: {}",
            early.text()
        );

        let challenged = post(
            &server,
            app,
            &login,
            "challenge-otp",
            json!({ "code": current_totp(OTP_SECRET_BASE32) }),
        )
        .await;
        assert_eq!(challenged.status_code(), 200, "{}", challenged.text());
        let body: Value = challenged.json();
        assert_eq!(
            body["required_actions"],
            json!(["configure_passkey"]),
            "{body}"
        );
        assert!(body.get("url").is_none(), "{body}");

        let spent = login.cookies();
        login.follow(&challenged);

        let replayed = server
            .post(&app.path("/login-actions/webauthn-public-key-create-options"))
            .add_header("Cookie", spent)
            .json(&json!({}))
            .await;
        assert_eq!(
            replayed.status_code(),
            401,
            "the step token spent on the challenge must be dead: {}",
            replayed.text()
        );

        let options = post(
            &server,
            app,
            &login,
            "webauthn-public-key-create-options",
            json!({}),
        )
        .await;
        assert_eq!(
            options.status_code(),
            200,
            "the required action opens once the OTP is answered: {}",
            options.text()
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test required_action_flow_test -- --ignored"]
fn enrolling_otp_as_the_last_required_action_completes_the_login() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        require(app, &user, "configure_otp").await;
        let client = app.client(ClientSpec::public()).await;

        let (login, body) = sign_in(&server, app, &client, &user).await;
        assert_eq!(body["status"], "RequiresActions", "{body}");

        let setup = server
            .get(&app.path("/login-actions/setup-otp"))
            .add_header("Cookie", login.cookies())
            .await;
        assert_eq!(setup.status_code(), 200, "{}", setup.text());
        let secret = setup.json::<Value>()["secret"]
            .as_str()
            .expect("enrolment secret")
            .to_string();

        let verified = post(
            &server,
            app,
            &login,
            "verify-otp",
            json!({ "code": current_totp(&secret), "label": "phone" }),
        )
        .await;
        assert_eq!(verified.status_code(), 200, "{}", verified.text());
        let body: Value = verified.json();
        assert!(
            redirect_with_code(&body),
            "the login must end on the client redirect: {body}"
        );
        assert!(
            verified.maybe_cookie("FERRISKEY_SSO").is_some(),
            "a completed login opens the SSO session"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test required_action_flow_test -- --ignored"]
fn a_password_update_hands_over_to_the_next_required_action() {
    let app = app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        require(app, &user, "update_password").await;
        require(app, &user, "configure_otp").await;
        let client = app.client(ClientSpec::public()).await;

        let (mut login, body) = sign_in(&server, app, &client, &user).await;
        assert_eq!(body["status"], "RequiresActions", "{body}");

        let updated = post(
            &server,
            app,
            &login,
            "update-password",
            json!({ "value": "An0ther!P@ssword#2025" }),
        )
        .await;
        assert_eq!(updated.status_code(), 200, "{}", updated.text());
        let body: Value = updated.json();
        assert_eq!(body["required_actions"], json!(["configure_otp"]), "{body}");
        login.follow(&updated);

        let setup = server
            .get(&app.path("/login-actions/setup-otp"))
            .add_header("Cookie", login.cookies())
            .await;
        assert_eq!(setup.status_code(), 200, "{}", setup.text());
        let secret = setup.json::<Value>()["secret"]
            .as_str()
            .expect("enrolment secret")
            .to_string();

        let verified = post(
            &server,
            app,
            &login,
            "verify-otp",
            json!({ "code": current_totp(&secret), "label": "phone" }),
        )
        .await;
        assert_eq!(verified.status_code(), 200, "{}", verified.text());
        let body: Value = verified.json();
        assert!(
            redirect_with_code(&body),
            "the login must end once every action is done: {body}"
        );
    });
}
