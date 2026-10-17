mod common;

use std::future::Future;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use common::{
    ADMIN_CLIENT_ID, ADMIN_PASSWORD, ADMIN_USERNAME, ClientSpec, TestApp, authorize, cached_app,
    exchange_code, password_grant, query_param, rt,
};
use serde_json::{Value, json};
use tracing::field::{Field, Visit};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

const PASSWORD_GRANT_BUDGET: usize = 19;
const WRONG_PASSWORD_BUDGET: usize = 8;
const UNKNOWN_USER_BUDGET: usize = 4;
const AUTHORIZE_BUDGET: usize = 4;
const AUTHENTICATE_BUDGET: usize = 14;
const CODE_EXCHANGE_BUDGET: usize = 16;

const BACKGROUND_WORK: &[&str] = &[
    "webhook_deliveries",
    "webhook_outbox",
    "device_auth_sessions",
    "broker_auth_sessions",
    "password_reset_tokens",
    "DELETE FROM \"compass_flows\"",
    "\"compass_flows\".\"started_at\" <",
    "DELETE FROM \"access_tokens\"",
    "DELETE FROM \"refresh_tokens\"",
    "DELETE FROM \"user_sessions\" WHERE \"user_sessions\".\"expires_at\"",
];

static STATEMENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());
static RECORDER: OnceLock<()> = OnceLock::new();

struct StatementRecorder;

impl<S: tracing::Subscriber> Layer<S> for StatementRecorder {
    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }

        let mut visitor = StatementVisitor::default();
        event.record(&mut visitor);
        if let Some(statement) = visitor.statement() {
            STATEMENTS
                .lock()
                .expect("statement log poisoned")
                .push(statement);
        }
    }
}

#[derive(Default)]
struct StatementVisitor {
    summary: Option<String>,
    statement: Option<String>,
}

impl StatementVisitor {
    fn statement(self) -> Option<String> {
        self.statement
            .filter(|statement| !statement.is_empty())
            .or(self.summary)
    }
}

impl Visit for StatementVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        match field.name() {
            "summary" => self.summary = Some(value.to_string()),
            "db.statement" => self.statement = Some(value.trim().to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        let rendered = rendered.trim_matches('"').trim().to_string();
        match field.name() {
            "summary" if self.summary.is_none() => self.summary = Some(rendered),
            "db.statement" if self.statement.is_none() => self.statement = Some(rendered),
            _ => {}
        }
    }
}

fn record_statements() {
    RECORDER.get_or_init(|| {
        tracing_subscriber::registry()
            .with(StatementRecorder)
            .init();
    });
}

fn is_background(statement: &str) -> bool {
    BACKGROUND_WORK
        .iter()
        .any(|pattern| statement.contains(pattern))
}

fn recorded() -> usize {
    STATEMENTS.lock().expect("statement log poisoned").len()
}

async fn settle() {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        let before = recorded();
        tokio::time::sleep(Duration::from_millis(250)).await;
        if recorded() == before || tokio::time::Instant::now() >= deadline {
            break;
        }
    }
}

async fn capture<T>(work: impl Future<Output = T>) -> (T, Vec<String>) {
    settle().await;
    STATEMENTS.lock().expect("statement log poisoned").clear();
    let output = work.await;
    settle().await;

    let statements = STATEMENTS
        .lock()
        .expect("statement log poisoned")
        .drain(..)
        .filter(|statement| !is_background(statement))
        .collect();
    (output, statements)
}

fn assert_within(step: &str, budget: usize, statements: &[String]) {
    let listing: Vec<String> = statements
        .iter()
        .enumerate()
        .map(|(index, statement)| {
            let first_line = statement.lines().next().unwrap_or_default();
            format!(
                "{:>3}  {}",
                index + 1,
                first_line.chars().take(110).collect::<String>()
            )
        })
        .collect();
    println!(
        "{step}: {} statements (budget {budget})\n{}",
        statements.len(),
        listing.join("\n")
    );
    assert!(
        statements.len() <= budget,
        "{step}: {} statements exceed the budget of {budget}",
        statements.len()
    );
}

async fn browser_login(
    server: &axum_test::TestServer,
    app: &TestApp,
    client: &common::SeededClient,
    username: &str,
    password: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (authorized, authorize_statements) =
        capture(authorize(server, app, &client.client_id, &[])).await;
    assert_eq!(authorized.status_code(), 302, "{}", authorized.text());
    let session = authorized.cookie("FERRISKEY_SESSION");

    let (authenticated, authenticate_statements) = capture(async {
        server
            .post(&app.path("/login-actions/authenticate"))
            .add_cookie(session)
            .add_query_param("client_id", &client.client_id)
            .json(&json!({ "username": username, "password": password }))
            .await
    })
    .await;
    assert_eq!(authenticated.status_code(), 200, "{}", authenticated.text());
    let url = authenticated.json::<Value>()["url"]
        .as_str()
        .expect("redirect url")
        .to_string();
    let code = query_param(&url, "code").expect("authorization code");

    let (exchanged, exchange_statements) =
        capture(exchange_code(server, app, client, &code, &[])).await;
    assert_eq!(exchanged.status_code(), 200, "{}", exchanged.text());

    (
        authorize_statements,
        authenticate_statements,
        exchange_statements,
    )
}

#[test]
#[ignore = "requires PostgreSQL — run: cargo test -p ferriskey-api --test login_statement_budget_test -- --ignored"]
fn a_login_stays_within_its_statement_budget() {
    record_statements();
    let app = cached_app();

    rt().block_on(async {
        let server = app.server();
        let client = app.client(ClientSpec::public().with_direct_access()).await;
        let user = app.user(&server, "Correct-horse-battery-7").await;

        let warm = password_grant(
            &server,
            app,
            &client.client_id,
            None,
            &user.username,
            &user.password,
        )
        .await;
        assert_eq!(warm.status_code(), 200, "{}", warm.text());
        browser_login(&server, app, &client, &user.username, &user.password).await;
        password_grant(
            &server,
            app,
            ADMIN_CLIENT_ID,
            None,
            ADMIN_USERNAME,
            ADMIN_PASSWORD,
        )
        .await;

        let (granted, statements) = capture(password_grant(
            &server,
            app,
            &client.client_id,
            None,
            &user.username,
            &user.password,
        ))
        .await;
        assert_eq!(granted.status_code(), 200, "{}", granted.text());
        assert_within("password grant", PASSWORD_GRANT_BUDGET, &statements);

        let (refused, statements) = capture(password_grant(
            &server,
            app,
            &client.client_id,
            None,
            &user.username,
            "not-the-password",
        ))
        .await;
        assert_ne!(refused.status_code(), 200);
        assert_within("wrong password", WRONG_PASSWORD_BUDGET, &statements);

        let (refused, statements) = capture(password_grant(
            &server,
            app,
            &client.client_id,
            None,
            "nobody-here",
            "not-the-password",
        ))
        .await;
        assert_ne!(refused.status_code(), 200);
        assert_within("unknown user", UNKNOWN_USER_BUDGET, &statements);

        let (authorize_statements, authenticate_statements, exchange_statements) =
            browser_login(&server, app, &client, &user.username, &user.password).await;
        assert_within(
            "authorization request",
            AUTHORIZE_BUDGET,
            &authorize_statements,
        );
        assert_within(
            "authenticate",
            AUTHENTICATE_BUDGET,
            &authenticate_statements,
        );
        assert_within("code exchange", CODE_EXCHANGE_BUDGET, &exchange_statements);
    });
}
