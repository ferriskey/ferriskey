mod common;

use axum::http::HeaderValue;
use axum_test::{TestResponse, TestServer};
use chrono::{DateTime, Utc};
use common::{ClientSpec, SeededClient, SeededUser, TestApp, app, query_param, rt};
use serde_json::{Value, json};
use uuid::Uuid;

const PASSWORD: &str = "Str0ng!P@ssword#2024";

#[derive(Default, Clone)]
struct Browser {
    device: Option<String>,
    sso: Option<String>,
}

impl Browser {
    fn cookies(&self, session: Option<&str>) -> HeaderValue {
        let mut pairs = Vec::new();
        if let Some(session) = session {
            pairs.push(format!("FERRISKEY_SESSION={session}"));
        }
        if let Some(device) = &self.device {
            pairs.push(format!("FERRISKEY_DEVICE={device}"));
        }
        if let Some(sso) = &self.sso {
            pairs.push(format!("FERRISKEY_SSO={sso}"));
        }
        HeaderValue::from_str(&pairs.join("; ")).expect("cookie header")
    }

    fn remember(&mut self, response: &TestResponse) {
        if let Some(device) = set_cookie_value(response, "FERRISKEY_DEVICE") {
            self.device = Some(device);
        }
        if let Some(sso) = set_cookie_value(response, "FERRISKEY_SSO") {
            self.sso = Some(sso);
        }
    }
}

fn set_cookie_raw(response: &TestResponse, name: &str) -> Option<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with(&format!("{name}=")))
        .map(str::to_string)
}

fn set_cookie_value(response: &TestResponse, name: &str) -> Option<String> {
    set_cookie_raw(response, name).map(|raw| {
        raw.split(';')
            .next()
            .and_then(|pair| pair.split_once('='))
            .map(|(_, value)| value.to_string())
            .unwrap_or_default()
    })
}

async fn set_chooser(app: &TestApp, realm_id: Uuid, enabled: bool) {
    sqlx::query("UPDATE realm_settings SET account_chooser_enabled = $1 WHERE realm_id = $2")
        .bind(enabled)
        .bind(realm_id)
        .execute(&app.pool)
        .await
        .expect("set account chooser");
}

async fn chooser_app() -> &'static TestApp {
    let app = app();
    set_chooser(app, app.realm_id, true).await;
    app
}

async fn begin(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    browser: &Browser,
    prompt_login: bool,
) -> String {
    let mut request = server
        .get(&app.oidc("auth"))
        .add_query_param("response_type", "code")
        .add_query_param("client_id", &client.client_id)
        .add_query_param("redirect_uri", common::CALLBACK)
        .add_query_param("scope", "openid")
        .add_query_param("state", "chooser-state")
        .add_header("Cookie", browser.cookies(None));
    if prompt_login {
        request = request.add_query_param("prompt", "login");
    }
    let response = request.await;
    assert_eq!(
        response.status_code(),
        302,
        "authorize: {}",
        response.text()
    );
    response.cookie("FERRISKEY_SESSION").value().to_string()
}

async fn login_as(
    server: &TestServer,
    app: &TestApp,
    client: &SeededClient,
    user: &SeededUser,
    browser: &mut Browser,
    user_agent: Option<&'static str>,
) -> TestResponse {
    let session = begin(server, app, client, browser, browser.sso.is_some()).await;

    let mut request = server
        .post(&app.path("/login-actions/authenticate"))
        .add_header("Cookie", browser.cookies(Some(&session)))
        .add_query_param("client_id", &client.client_id)
        .json(&json!({ "username": user.username, "password": user.password }));
    if let Some(user_agent) = user_agent {
        request = request.add_header("User-Agent", HeaderValue::from_static(user_agent));
    }
    let response = request.await;
    assert_eq!(
        response.status_code(),
        200,
        "authenticate: {}",
        response.text()
    );
    browser.remember(&response);
    response
}

async fn list(
    server: &TestServer,
    app: &TestApp,
    realm: &str,
    client: &SeededClient,
    browser: &Browser,
    max_age: Option<i64>,
) -> TestResponse {
    let _ = app;
    let mut request = server
        .get(&format!("/realms/{realm}/login-actions/accounts"))
        .add_query_param("client_id", &client.client_id)
        .add_header("Cookie", browser.cookies(None));
    if let Some(max_age) = max_age {
        request = request.add_query_param("max_age", max_age);
    }
    request.await
}

fn entries(response: &TestResponse) -> Vec<Value> {
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json::<Value>()["data"]
        .as_array()
        .expect("data array")
        .clone()
}

async fn session_of_user(app: &TestApp, user: &SeededUser) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM user_sessions WHERE user_id = $1::uuid")
        .bind(&user.id)
        .fetch_one(&app.pool)
        .await
        .expect("the user's session")
}

async fn sso_hash(app: &TestApp, session_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT sso_token_hash FROM user_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&app.pool)
    .await
    .expect("sso_token_hash")
}

async fn authenticated_at(app: &TestApp, session_id: Uuid) -> DateTime<Utc> {
    sqlx::query_scalar::<_, DateTime<Utc>>(
        "SELECT authenticated_at FROM user_sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_one(&app.pool)
    .await
    .expect("authenticated_at")
}

async fn select(
    server: &TestServer,
    app: &TestApp,
    browser: &Browser,
    auth_session: &str,
    session_id: Uuid,
    max_age: Option<i64>,
) -> TestResponse {
    let mut request = server
        .post(&app.path("/login-actions/accounts/select"))
        .add_header("Cookie", browser.cookies(Some(auth_session)))
        .json(&json!({ "session_id": session_id }));
    if let Some(max_age) = max_age {
        request = request.add_query_param("max_age", max_age);
    }
    request.await
}

struct Fixture {
    server: TestServer,
    client: SeededClient,
    alice: SeededUser,
    bob: SeededUser,
    browser: Browser,
}

async fn alice_then_bob(app: &'static TestApp) -> Fixture {
    let server = app.server();
    let client = app.client(ClientSpec::public()).await;
    let alice = app.user(&server, PASSWORD).await;
    let bob = app.user(&server, PASSWORD).await;
    let mut browser = Browser::default();

    login_as(&server, app, &client, &alice, &mut browser, None).await;
    login_as(&server, app, &client, &bob, &mut browser, None).await;

    Fixture {
        server,
        client,
        alice,
        bob,
        browser,
    }
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_second_account_on_the_same_browser_is_remembered() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;

        let response = list(&f.server, app, &app.realm, &f.client, &f.browser, None).await;
        let entries = entries(&response);

        assert_eq!(entries.len(), 2, "{entries:?}");
        assert!(
            entries.iter().all(|entry| entry.get("user_id").is_none()),
            "{entries:?}"
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry.get("masked_email").is_none()),
            "the email stays hidden unless the realm shows it: {entries:?}"
        );

        let active: Vec<&Value> = entries
            .iter()
            .filter(|entry| entry["is_active"] == true)
            .collect();
        assert_eq!(active.len(), 1);
        let bob_session = session_of_user(app, &f.bob).await;
        assert_eq!(active[0]["session_id"], bob_session.to_string());
        let alice_session = session_of_user(app, &f.alice).await;
        let alice_entry = entries
            .iter()
            .find(|entry| entry["session_id"] == alice_session.to_string())
            .expect("alice's entry");
        assert_eq!(alice_entry["is_active"], false);
        assert_eq!(alice_entry["state"], "active");
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn the_device_cookie_is_re_set_on_every_login_with_safe_attributes() {
    rt().block_on(async {
        let app = chooser_app().await;
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;
        let alice = app.user(&server, PASSWORD).await;
        let bob = app.user(&server, PASSWORD).await;
        let mut browser = Browser::default();

        let first = login_as(
            &server,
            app,
            &client,
            &alice,
            &mut browser,
            Some("chooser-test/1.0"),
        )
        .await;
        let raw = set_cookie_raw(&first, "FERRISKEY_DEVICE").expect("a device cookie");
        assert!(raw.contains("HttpOnly"), "{raw}");
        assert!(raw.contains("SameSite=Lax"), "{raw}");
        assert!(raw.contains("Max-Age="), "{raw}");
        assert!(
            set_cookie_value(&first, "FERRISKEY_DEVICE")
                .expect("value")
                .len()
                >= 43,
            "a 256-bit opaque value"
        );

        let agent = sqlx::query_scalar::<_, Option<String>>(
            "SELECT user_agent FROM user_sessions WHERE user_id = $1::uuid",
        )
        .bind(&alice.id)
        .fetch_one(&app.pool)
        .await
        .expect("user agent");
        assert_eq!(agent.as_deref(), Some("chooser-test/1.0"));

        let device = browser.device.clone();
        let second = login_as(&server, app, &client, &bob, &mut browser, None).await;
        let raw = set_cookie_raw(&second, "FERRISKEY_DEVICE")
            .expect("a known device has its cookie re-set with a fresh lifetime");
        assert!(raw.contains("Max-Age="), "{raw}");
        assert_eq!(
            set_cookie_value(&second, "FERRISKEY_DEVICE"),
            device,
            "the same token is sent back"
        );
        assert_eq!(browser.device, device);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn the_chooser_list_is_empty_without_a_device_cookie() {
    rt().block_on(async {
        let app = chooser_app().await;
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;

        let absent = list(&server, app, &app.realm, &client, &Browser::default(), None).await;
        let forged = Browser {
            device: Some("ab".repeat(32)),
            sso: None,
        };
        let random = list(&server, app, &app.realm, &client, &forged, None).await;

        assert_eq!(absent.status_code(), 200);
        assert_eq!(absent.json::<Value>(), json!({ "data": [] }));
        assert_eq!(random.status_code(), absent.status_code());
        assert_eq!(random.text(), absent.text());
        assert_eq!(
            random.headers().get("content-type"),
            absent.headers().get("content-type")
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn selecting_an_account_resumes_its_session_without_credentials() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;
        let before_hash = sso_hash(app, alice_session).await;
        let before_authenticated = authenticated_at(app, alice_session).await;
        let before_secret = f.browser.sso.clone();

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;

        let response = select(&f.server, app, &browser, &auth_session, alice_session, None).await;

        assert_eq!(response.status_code(), 302, "{}", response.text());
        let location = response
            .headers()
            .get("location")
            .expect("location")
            .to_str()
            .expect("ascii")
            .to_string();
        assert!(location.starts_with(common::CALLBACK), "{location}");
        assert!(query_param(&location, "code").is_some(), "{location}");
        assert_eq!(
            query_param(&location, "state").as_deref(),
            Some("chooser-state")
        );

        let fresh = set_cookie_value(&response, "FERRISKEY_SSO").expect("a fresh SSO cookie");
        assert_ne!(Some(fresh), before_secret);

        let after_hash = sso_hash(app, alice_session).await;
        assert!(after_hash.is_some());
        assert_ne!(after_hash, before_hash);
        assert_eq!(
            authenticated_at(app, alice_session).await,
            before_authenticated
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn max_age_forces_re_authentication_from_the_chooser() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;

        sqlx::query(
            "UPDATE user_sessions SET authenticated_at = NOW() - INTERVAL '600 seconds' WHERE id = $1",
        )
        .bind(alice_session)
        .execute(&app.pool)
        .await
        .expect("age the session");

        let response = list(&f.server, app, &app.realm, &f.client, &f.browser, Some(300)).await;
        let entries = entries(&response);
        let alice_entry = entries
            .iter()
            .find(|entry| entry["session_id"] == alice_session.to_string())
            .expect("alice's entry");
        assert_eq!(alice_entry["requires_reauthentication"], true);

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;

        let selected = select(
            &f.server,
            app,
            &browser,
            &auth_session,
            alice_session,
            Some(300),
        )
        .await;

        assert_eq!(selected.status_code(), 200, "{}", selected.text());
        let body: Value = selected.json();
        assert_eq!(body["status"], "requires_credentials", "{body}");
        assert_eq!(body["login_hint"], f.alice.username, "{body}");
        assert!(body.get("code").is_none() && body.get("url").is_none(), "{body}");
        assert!(set_cookie_raw(&selected, "FERRISKEY_SSO").is_none());

        let remembered = sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT reauth_session_id FROM auth_sessions WHERE id = $1::uuid",
        )
        .bind(&auth_session)
        .fetch_one(&app.pool)
        .await
        .expect("reauth_session_id");
        assert_eq!(remembered, Some(alice_session));
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_revoked_session_disappears_from_the_chooser() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;
        let token = app.admin_token(&f.server).await;

        let revoked = f
            .server
            .delete(&app.path(&format!("/users/{}/sessions/{alice_session}", f.alice.id)))
            .authorization_bearer(&token)
            .await;
        assert_eq!(revoked.status_code(), 204, "{}", revoked.text());

        let response = list(&f.server, app, &app.realm, &f.client, &f.browser, None).await;
        let entries = entries(&response);

        assert_eq!(entries.len(), 1, "{entries:?}");
        assert!(
            entries
                .iter()
                .all(|entry| entry["session_id"] != alice_session.to_string())
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn removing_an_account_from_the_device_keeps_the_session_row() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;
        let bob_session = session_of_user(app, &f.bob).await;

        let removed = f
            .server
            .delete(&app.path(&format!("/login-actions/accounts/{alice_session}")))
            .add_header("Cookie", f.browser.cookies(None))
            .await;
        assert_eq!(removed.status_code(), 204, "{}", removed.text());

        let response = list(&f.server, app, &app.realm, &f.client, &f.browser, None).await;
        let entries = entries(&response);
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0]["session_id"], bob_session.to_string());

        let token = app.admin_token(&f.server).await;
        let sessions = f
            .server
            .get(&app.path(&format!("/users/{}/sessions", f.alice.id)))
            .authorization_bearer(&token)
            .await;
        assert_eq!(sessions.status_code(), 200, "{}", sessions.text());
        let sessions: Value = sessions.json();
        assert!(
            sessions["data"]
                .as_array()
                .expect("sessions")
                .iter()
                .any(|session| session["id"] == alice_session.to_string()),
            "{sessions}"
        );
        assert_eq!(sso_hash(app, alice_session).await, None);
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_realm_with_the_chooser_off_remembers_nothing() {
    rt().block_on(async {
        let app = chooser_app().await;
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;
        let alice = app.user(&server, PASSWORD).await;
        let bob = app.user(&server, PASSWORD).await;

        set_chooser(app, app.realm_id, false).await;
        let mut cold = Browser::default();
        let login = login_as(&server, app, &client, &alice, &mut cold, None).await;
        assert!(
            set_cookie_raw(&login, "FERRISKEY_DEVICE").is_none(),
            "no device cookie while the chooser is off"
        );
        assert!(cold.device.is_none());

        set_chooser(app, app.realm_id, true).await;
        let mut browser = Browser::default();
        login_as(&server, app, &client, &alice, &mut browser, None).await;
        login_as(&server, app, &client, &bob, &mut browser, None).await;
        let on = list(&server, app, &app.realm, &client, &browser, None).await;
        assert_eq!(entries(&on).len(), 2);

        set_chooser(app, app.realm_id, false).await;
        let off = list(&server, app, &app.realm, &client, &browser, None).await;
        set_chooser(app, app.realm_id, true).await;

        assert_eq!(off.status_code(), 200);
        assert_eq!(off.json::<Value>(), json!({ "data": [] }));
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_device_cookie_from_another_realm_resolves_to_nothing() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        assert!(f.browser.device.is_some());

        let (other_realm, other_realm_id) = app.create_realm().await;
        sqlx::query(
            "INSERT INTO realm_settings (id, realm_id, account_chooser_enabled) VALUES ($1, $2, TRUE)",
        )
        .bind(Uuid::new_v4())
        .bind(other_realm_id)
        .execute(&app.pool)
        .await
        .expect("enable the chooser in the other realm");
        let other_client = app
            .client_in(other_realm_id, ClientSpec::public())
            .await;

        let response = list(
            &f.server,
            app,
            &other_realm,
            &other_client,
            &f.browser,
            None,
        )
        .await;

        assert_eq!(response.status_code(), 200, "{}", response.text());
        assert_eq!(response.json::<Value>(), json!({ "data": [] }));
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_session_not_on_this_device_is_a_404_to_select_and_to_remove() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;

        let server = &f.server;
        let carol = app.user(server, PASSWORD).await;
        let mut other_browser = Browser::default();
        login_as(server, app, &f.client, &carol, &mut other_browser, None).await;
        let carol_session = session_of_user(app, &carol).await;

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(server, app, &f.client, &browser, false).await;

        let foreign = select(server, app, &browser, &auth_session, carol_session, None).await;
        assert_eq!(foreign.status_code(), 404, "{}", foreign.text());
        let unknown = select(server, app, &browser, &auth_session, Uuid::new_v4(), None).await;
        assert_eq!(unknown.status_code(), 404, "{}", unknown.text());

        let removed_foreign = server
            .delete(&app.path(&format!("/login-actions/accounts/{carol_session}")))
            .add_header("Cookie", f.browser.cookies(None))
            .await;
        assert_eq!(
            removed_foreign.status_code(),
            404,
            "{}",
            removed_foreign.text()
        );
        let removed_without_device = server
            .delete(&app.path(&format!("/login-actions/accounts/{carol_session}")))
            .await;
        assert_eq!(removed_without_device.status_code(), 404);

        assert!(sso_hash(app, carol_session).await.is_some());
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn logging_out_removes_the_account_from_the_chooser() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let bob_session = session_of_user(app, &f.bob).await;

        let logout = f
            .server
            .post(&app.oidc("logout"))
            .add_header("Cookie", f.browser.cookies(None))
            .form(&json!({}))
            .await;
        assert_eq!(logout.status_code(), 204, "{}", logout.text());

        let response = list(&f.server, app, &app.realm, &f.client, &f.browser, None).await;
        let entries = entries(&response);

        assert_eq!(entries.len(), 1, "{entries:?}");
        assert!(
            entries
                .iter()
                .all(|entry| entry["session_id"] != bob_session.to_string())
        );
    });
}

const OTP_SECRET: &str = "JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP";

fn current_totp() -> String {
    use hmac::{Hmac, KeyInit, Mac};
    use sha1::Sha1;

    let secret = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, OTP_SECRET)
        .expect("base32 secret");
    let counter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs()
        / 30;
    let mut mac = Hmac::<Sha1>::new_from_slice(&secret).expect("hmac key");
    mac.update(&counter.to_be_bytes());
    let digest = mac.finalize().into_bytes();
    let offset = (digest[19] & 0x0f) as usize;
    let code = ((u32::from(digest[offset]) & 0x7f) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);
    format!("{:06}", code % 1_000_000)
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn a_login_completed_through_an_otp_challenge_is_remembered() {
    rt().block_on(async {
        let app = chooser_app().await;
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;
        let user = app.user(&server, PASSWORD).await;
        sqlx::query(
            "INSERT INTO credentials (id, credential_type, user_id, secret_data, credential_data, user_label)
             VALUES ($1, 'otp', $2::uuid, $3, '{}'::jsonb, 'test-authenticator')",
        )
        .bind(Uuid::new_v4())
        .bind(&user.id)
        .bind(OTP_SECRET)
        .execute(&app.pool)
        .await
        .expect("enrol the authenticator");

        let mut browser = Browser::default();
        let session = begin(&server, app, &client, &browser, false).await;
        let login = server
            .post(&app.path("/login-actions/authenticate"))
            .add_header("Cookie", browser.cookies(Some(&session)))
            .add_query_param("client_id", &client.client_id)
            .json(&json!({ "username": user.username, "password": user.password }))
            .await;
        assert_eq!(login.status_code(), 200, "{}", login.text());
        assert!(
            set_cookie_raw(&login, "FERRISKEY_DEVICE").is_none(),
            "no session is open before the second factor"
        );
        let step_token = set_cookie_value(&login, "FERRISKEY_LOGIN_ACTION").expect("step token");

        let challenged = server
            .post(&app.path("/login-actions/challenge-otp"))
            .add_header(
                "Cookie",
                HeaderValue::from_str(&format!(
                    "FERRISKEY_SESSION={session}; FERRISKEY_LOGIN_ACTION={step_token}"
                ))
                .expect("cookies"),
            )
            .json(&json!({ "code": current_totp() }))
            .await;
        assert_eq!(challenged.status_code(), 200, "{}", challenged.text());
        assert!(
            set_cookie_raw(&challenged, "FERRISKEY_DEVICE").is_some(),
            "the device cookie follows the completed login"
        );
        browser.remember(&challenged);

        let response = list(&server, app, &app.realm, &client, &browser, None).await;
        let entries = entries(&response);
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0]["is_active"], true);
        assert_eq!(
            entries[0]["session_id"],
            session_of_user(app, &user).await.to_string()
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_test -- --ignored"]
fn turning_the_chooser_off_stops_select_and_remove() {
    rt().block_on(async {
        let app = chooser_app().await;
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;

        set_chooser(app, app.realm_id, false).await;
        let selected = select(&f.server, app, &browser, &auth_session, alice_session, None).await;
        let removed = f
            .server
            .delete(&app.path(&format!("/login-actions/accounts/{alice_session}")))
            .add_header("Cookie", f.browser.cookies(None))
            .await;
        let listed = list(&f.server, app, &app.realm, &f.client, &f.browser, None).await;
        set_chooser(app, app.realm_id, true).await;

        assert_eq!(selected.status_code(), 404, "{}", selected.text());
        assert_eq!(removed.status_code(), 404, "{}", removed.text());
        assert_eq!(listed.json::<Value>(), json!({ "data": [] }));
        assert!(sso_hash(app, alice_session).await.is_some());
    });
}
