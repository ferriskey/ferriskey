mod common;

use axum::http::HeaderValue;
use axum_test::{TestResponse, TestServer};
use common::{ClientSpec, SeededClient, SeededUser, TestApp, app, rt};
use serde_json::{Value, json};
use uuid::Uuid;

const SELECT_AGENT: &str = "ChooserAudit/2.0 (select)";
const LOGIN_AGENT: &str = "ChooserAudit/1.0 (login)";
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

fn set_cookie_value(response: &TestResponse, name: &str) -> Option<String> {
    response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|value| value.starts_with(&format!("{name}=")))
        .and_then(|raw| raw.split(';').next())
        .and_then(|pair| pair.split_once('='))
        .map(|(_, value)| value.to_string())
}

async fn prepare(app: &TestApp) {
    sqlx::query(
        "UPDATE realm_settings SET account_chooser_enabled = true, compass_enabled = true WHERE realm_id = $1",
    )
    .bind(app.realm_id)
    .execute(&app.pool)
    .await
    .expect("enable the chooser and compass");
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
        .add_query_param("state", "audit-state")
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
) {
    let session = begin(server, app, client, browser, browser.sso.is_some()).await;
    let response = server
        .post(&app.path("/login-actions/authenticate"))
        .add_header("Cookie", browser.cookies(Some(&session)))
        .add_query_param("client_id", &client.client_id)
        .add_header("User-Agent", HeaderValue::from_static(LOGIN_AGENT))
        .json(&json!({ "username": user.username, "password": user.password }))
        .await;
    assert_eq!(
        response.status_code(),
        200,
        "authenticate: {}",
        response.text()
    );
    browser.remember(&response);
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
    request
        .add_header("User-Agent", HeaderValue::from_static(SELECT_AGENT))
        .await
}

async fn session_of_user(app: &TestApp, user: &SeededUser) -> Uuid {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM user_sessions WHERE user_id = $1::uuid")
        .bind(&user.id)
        .fetch_one(&app.pool)
        .await
        .expect("the user's session")
}

async fn agent_of(app: &TestApp, event_type: &str, session_id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT user_agent FROM security_events \
         WHERE realm_id = $1 AND event_type = $2 AND target_id = $3",
    )
    .bind(app.realm_id)
    .bind(event_type)
    .bind(session_id)
    .fetch_one(&app.pool)
    .await
    .expect("read the event user agent")
}

async fn events_of(app: &TestApp, event_type: &str, session_id: Uuid) -> Vec<Value> {
    sqlx::query_scalar::<_, Value>(
        "SELECT details FROM security_events \
         WHERE realm_id = $1 AND event_type = $2 AND target_type = 'session' AND target_id = $3",
    )
    .bind(app.realm_id)
    .bind(event_type)
    .bind(session_id)
    .fetch_all(&app.pool)
    .await
    .expect("read the audit trail")
}

async fn flow_steps(app: &TestApp, auth_session: &str) -> Vec<(String, String)> {
    for _ in 0..50 {
        let steps = sqlx::query_as::<_, (String, String)>(
            "SELECT s.step_name, s.status::text FROM compass_flow_steps s \
             JOIN compass_flows f ON f.id = s.flow_id \
             JOIN auth_sessions a ON a.compass_flow_id = f.id \
             WHERE a.id = $1::uuid ORDER BY s.started_at",
        )
        .bind(auth_session)
        .fetch_all(&app.pool)
        .await
        .expect("read the compass steps");
        if steps.iter().any(|(name, _)| name == "account_selection") {
            return steps;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    Vec::new()
}

struct Fixture {
    server: TestServer,
    client: SeededClient,
    alice: SeededUser,
    browser: Browser,
}

async fn alice_then_bob(app: &'static TestApp) -> Fixture {
    prepare(app).await;
    let server = app.server();
    let client = app.client(ClientSpec::public()).await;
    let alice = app.user(&server, PASSWORD).await;
    let bob = app.user(&server, PASSWORD).await;
    let mut browser = Browser::default();

    login_as(&server, app, &client, &alice, &mut browser).await;
    login_as(&server, app, &client, &bob, &mut browser).await;

    Fixture {
        server,
        client,
        alice,
        browser,
    }
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_audit_test -- --ignored"]
fn selecting_an_account_is_audited_and_traced() {
    rt().block_on(async {
        let app = app();
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;
        let response = select(&f.server, app, &browser, &auth_session, alice_session, None).await;
        assert_eq!(response.status_code(), 302, "{}", response.text());

        let selected = events_of(app, "account_selected", alice_session).await;
        assert_eq!(selected.len(), 1, "{selected:?}");
        assert_eq!(selected[0]["via"], "account_chooser");
        assert!(
            Uuid::parse_str(
                selected[0]["device_session_id"]
                    .as_str()
                    .unwrap_or_default()
            )
            .is_ok(),
            "{}",
            selected[0]
        );
        assert!(selected[0].get("reason").is_none());
        assert_eq!(
            agent_of(app, "account_selected", alice_session)
                .await
                .as_deref(),
            Some(SELECT_AGENT)
        );
        assert!(
            events_of(app, "account_selection_denied", alice_session)
                .await
                .is_empty()
        );

        let steps = flow_steps(app, &auth_session).await;
        assert!(
            steps
                .iter()
                .any(|(name, status)| name == "account_selection" && status == "success"),
            "{steps:?}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_audit_test -- --ignored"]
fn a_refused_selection_is_audited_with_its_reason() {
    rt().block_on(async {
        let app = app();
        let f = alice_then_bob(app).await;
        let alice_session = session_of_user(app, &f.alice).await;

        sqlx::query(
            "UPDATE user_sessions SET authenticated_at = NOW() - INTERVAL '600 seconds' WHERE id = $1",
        )
        .bind(alice_session)
        .execute(&app.pool)
        .await
        .expect("age the session");

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;
        let response = select(
            &f.server,
            app,
            &browser,
            &auth_session,
            alice_session,
            Some(300),
        )
        .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        assert_eq!(response.json::<Value>()["status"], "requires_credentials");

        let denied = events_of(app, "account_selection_denied", alice_session).await;
        assert_eq!(denied.len(), 1, "{denied:?}");
        assert_eq!(denied[0]["via"], "account_chooser");
        assert_eq!(denied[0]["reason"], "reauthentication_required");
        assert!(denied[0]["device_session_id"].is_string());
        assert_eq!(
            agent_of(app, "account_selection_denied", alice_session)
                .await
                .as_deref(),
            Some(SELECT_AGENT)
        );
        assert!(
            events_of(app, "account_selected", alice_session)
                .await
                .is_empty()
        );

        let steps = flow_steps(app, &auth_session).await;
        assert!(
            steps
                .iter()
                .any(|(name, status)| name == "account_selection" && status == "failure"),
            "{steps:?}"
        );
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_audit_test -- --ignored"]
fn a_selection_for_a_session_off_the_device_records_nothing() {
    rt().block_on(async {
        let app = app();
        let f = alice_then_bob(app).await;
        let stranger = app.user(&f.server, PASSWORD).await;
        let other_server = app.server();
        let other_client = app.client(ClientSpec::public()).await;
        let mut other_browser = Browser::default();
        login_as(
            &other_server,
            app,
            &other_client,
            &stranger,
            &mut other_browser,
        )
        .await;
        let stranger_session = session_of_user(app, &stranger).await;

        let mut browser = f.browser.clone();
        browser.sso = None;
        let auth_session = begin(&f.server, app, &f.client, &browser, false).await;
        let response = select(
            &f.server,
            app,
            &browser,
            &auth_session,
            stranger_session,
            None,
        )
        .await;
        assert_eq!(response.status_code(), 404, "{}", response.text());

        for event_type in ["account_selected", "account_selection_denied"] {
            assert!(
                events_of(app, event_type, stranger_session)
                    .await
                    .is_empty()
            );
        }
    });
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test account_chooser_audit_test -- --ignored"]
fn the_first_login_on_a_browser_records_the_device_session() {
    rt().block_on(async {
        let app = app();
        prepare(app).await;
        let server = app.server();
        let client = app.client(ClientSpec::public()).await;
        let alice = app.user(&server, PASSWORD).await;
        let bob = app.user(&server, PASSWORD).await;
        let mut browser = Browser::default();

        login_as(&server, app, &client, &alice, &mut browser).await;
        let alice_session = session_of_user(app, &alice).await;
        login_as(&server, app, &client, &bob, &mut browser).await;
        let bob_session = session_of_user(app, &bob).await;

        let created = events_of(app, "device_session_created", alice_session).await;
        assert_eq!(created.len(), 1, "{created:?}");
        assert_eq!(created[0]["via"], "account_chooser");
        assert!(created[0]["device_session_id"].is_string());
        assert_eq!(
            agent_of(app, "device_session_created", alice_session)
                .await
                .as_deref(),
            Some(LOGIN_AGENT)
        );
        assert!(
            events_of(app, "device_session_created", bob_session)
                .await
                .is_empty(),
            "joining an existing device creates no device session"
        );
    });
}
