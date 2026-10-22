mod common;

use common::{ADMIN_CLIENT_ID, ClientSpec, TestApp, authorize, password_grant, rt, sealed_app};
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha1::Sha1;
use uuid::Uuid;

const PASSWORD: &str = "Str0ng!P@ssword#2024";

fn base32_decode(input: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut bits: u32 = 0;
    let mut held = 0;
    let mut out = Vec::new();
    for symbol in input.trim_end_matches('=').bytes() {
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

fn totp_for(secret: &str) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs();
    let mut mac = Hmac::<Sha1>::new_from_slice(&base32_decode(secret)).expect("hmac key");
    mac.update(&(now / 30).to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = (digest[19] & 0x0f) as usize;
    let code = ((u32::from(digest[offset]) & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);
    format!("{:06}", code % 1_000_000)
}

async fn stored(app: &TestApp, sql: &str, id: Uuid) -> String {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(&app.pool)
        .await
        .expect("read the stored value")
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test sealed_secrets_test -- --ignored"]
fn the_signing_key_of_a_realm_is_sealed_and_its_tokens_still_verify() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let token = app.admin_token(&server).await;

        let profile = server
            .get(&app.path("/users/me"))
            .authorization_bearer(&token)
            .await;
        assert_eq!(profile.status_code(), 200, "{}", profile.text());

        let key: String = stored(
            app,
            "SELECT private_key FROM jwt_keys WHERE realm_id = $1",
            app.realm_id,
        )
        .await;
        assert!(key.starts_with("fk1:kid-a:"), "{key}");
        assert!(!key.contains("PRIVATE KEY"));
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test sealed_secrets_test -- --ignored"]
fn an_enrolled_totp_seed_is_sealed_and_still_accepted_at_login() {
    let app = sealed_app();
    rt().block_on(async {
        let server = app.server();
        let user = app.user(&server, PASSWORD).await;
        let user_id: Uuid = user.id.parse().expect("user id");
        let token = password_grant(
            &server,
            app,
            ADMIN_CLIENT_ID,
            None,
            &user.username,
            PASSWORD,
        )
        .await
        .json::<Value>()["access_token"]
            .as_str()
            .expect("access token")
            .to_string();

        let elevation = server
            .post(&app.path("/users/me/reauthenticate"))
            .authorization_bearer(&token)
            .json(&json!({ "password": PASSWORD }))
            .await;
        assert_eq!(elevation.status_code(), 200, "{}", elevation.text());
        let elevation_id = elevation.json::<Value>()["elevation_id"]
            .as_str()
            .expect("elevation id")
            .to_string();

        let started = server
            .post(&app.path("/users/me/mfa/otp"))
            .authorization_bearer(&token)
            .json(&json!({ "elevation_id": elevation_id }))
            .await;
        assert_eq!(started.status_code(), 200, "{}", started.text());
        let seed = started.json::<Value>()["secret"]
            .as_str()
            .expect("enrolment secret")
            .to_string();

        let pending: String =
            sqlx::query_scalar("SELECT secret FROM otp_enrollments WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&app.pool)
                .await
                .expect("read the pending secret");
        assert!(pending.starts_with("fk1:kid-a:"), "{pending}");

        let confirmed = server
            .put(&app.path("/users/me/mfa/otp"))
            .authorization_bearer(&token)
            .json(&json!({
                "elevation_id": elevation_id,
                "code": totp_for(&seed),
                "label": "Test authenticator",
            }))
            .await;
        assert_eq!(confirmed.status_code(), 200, "{}", confirmed.text());

        let at_rest: String = sqlx::query_scalar(
            "SELECT secret_data FROM credentials WHERE user_id = $1 AND credential_type = 'otp'",
        )
        .bind(user_id)
        .fetch_one(&app.pool)
        .await
        .expect("read the stored seed");
        assert!(at_rest.starts_with("fk1:kid-a:"), "{at_rest}");
        assert!(!at_rest.contains(&seed));

        let client = app.client(ClientSpec::public()).await;
        let authorized = authorize(&server, app, &client.client_id, &[]).await;
        assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
        let session = authorized.cookie("FERRISKEY_SESSION");
        let session_id = session.value().to_string();

        let login = server
            .post(&app.path("/login-actions/authenticate"))
            .add_cookie(session)
            .add_query_param("client_id", &client.client_id)
            .json(&json!({ "username": user.username, "password": PASSWORD }))
            .await;
        assert_eq!(login.status_code(), 200, "{}", login.text());
        assert_eq!(login.json::<Value>()["status"], "RequiresOtpChallenge");
        let step_token = login.cookie("FERRISKEY_LOGIN_ACTION").value().to_string();

        let challenged = server
            .post(&app.path("/login-actions/challenge-otp"))
            .add_header(
                "Cookie",
                format!("FERRISKEY_SESSION={session_id}; FERRISKEY_LOGIN_ACTION={step_token}")
                    .parse::<axum::http::HeaderValue>()
                    .expect("cookies"),
            )
            .json(&json!({ "code": totp_for(&seed) }))
            .await;

        assert_eq!(challenged.status_code(), 200, "{}", challenged.text());
        assert!(
            challenged.json::<Value>()["url"]
                .as_str()
                .is_some_and(|url| url.contains("code=")),
            "the unsealed seed must validate the code: {}",
            challenged.text()
        );
    });
}
