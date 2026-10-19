mod common;

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use common::{ClientSpec, cached_app, client_credentials, rt};
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

const WARM_UP: usize = 20;
const SAMPLES: usize = 200;

struct Timings {
    spans: Vec<(&'static str, Duration)>,
    statements: Vec<(String, Duration)>,
}

static TIMINGS: Mutex<Timings> = Mutex::new(Timings {
    spans: Vec::new(),
    statements: Vec::new(),
});
static PROFILER: OnceLock<()> = OnceLock::new();

struct Started(Instant);

struct Profiler;

impl<S> Layer<S> for Profiler
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, _: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(Started(Instant::now()));
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(&id) else {
            return;
        };
        let name = span.name();
        if !name.starts_with("auth.") {
            return;
        }
        let Some(started) = span.extensions().get::<Started>().map(|s| s.0) else {
            return;
        };
        TIMINGS
            .lock()
            .expect("timings poisoned")
            .spans
            .push((name, started.elapsed()));
    }

    fn on_event(&self, event: &tracing::Event<'_>, _: Context<'_, S>) {
        if event.metadata().target() != "sqlx::query" {
            return;
        }
        let mut visitor = StatementVisitor::default();
        event.record(&mut visitor);
        let elapsed = visitor.elapsed;
        if let (Some(statement), Some(elapsed)) = (visitor.statement(), elapsed) {
            TIMINGS
                .lock()
                .expect("timings poisoned")
                .statements
                .push((statement, Duration::from_secs_f64(elapsed)));
        }
    }
}

#[derive(Default)]
struct StatementVisitor {
    summary: Option<String>,
    statement: Option<String>,
    elapsed: Option<f64>,
}

impl StatementVisitor {
    fn statement(self) -> Option<String> {
        self.statement
            .filter(|statement| !statement.is_empty())
            .or(self.summary)
    }
}

impl Visit for StatementVisitor {
    fn record_f64(&mut self, field: &Field, value: f64) {
        if field.name() == "elapsed_secs" {
            self.elapsed = Some(value);
        }
    }

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

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let rank = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn row(label: &str, per_request: f64, durations: &mut [Duration]) -> String {
    durations.sort();
    let mean = durations.iter().sum::<Duration>() / durations.len().max(1) as u32;
    format!(
        "{:<58} {:>5.2}/req  mean {:>7.3} ms  p50 {:>7.3} ms  p95 {:>7.3} ms  max {:>7.3} ms",
        label,
        per_request,
        ms(mean),
        ms(percentile(durations, 0.50)),
        ms(percentile(durations, 0.95)),
        ms(percentile(durations, 1.0)),
    )
}

fn report(mut wall: Vec<Duration>) {
    let timings = TIMINGS.lock().expect("timings poisoned");
    let requests = wall.len() as f64;

    println!(
        "\n== wall time of {} client_credentials grants (in-process axum, local Postgres)",
        wall.len()
    );
    println!("{}", row("request", 1.0, &mut wall));

    let mut by_span: BTreeMap<&str, Vec<Duration>> = BTreeMap::new();
    for (name, elapsed) in &timings.spans {
        by_span.entry(name).or_default().push(*elapsed);
    }
    println!("\n== spans");
    let mut spans: Vec<(&str, Vec<Duration>)> = by_span.into_iter().collect();
    spans.sort_by(|a, b| {
        let mean = |v: &Vec<Duration>| v.iter().sum::<Duration>() / v.len().max(1) as u32;
        mean(&b.1).cmp(&mean(&a.1))
    });
    for (name, mut durations) in spans {
        let count = durations.len() as f64 / requests;
        println!("{}", row(name, count, &mut durations));
    }

    let mut by_statement: BTreeMap<String, Vec<Duration>> = BTreeMap::new();
    for (statement, elapsed) in &timings.statements {
        let key: String = statement
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(56)
            .collect();
        by_statement.entry(key).or_default().push(*elapsed);
    }
    let total_db: Duration = timings.statements.iter().map(|(_, d)| *d).sum();
    println!(
        "\n== sql statements ({} in total, {:.2} per request, {:.3} ms of statement time per request)",
        timings.statements.len(),
        timings.statements.len() as f64 / requests,
        ms(total_db) / requests
    );
    let mut statements: Vec<(String, Vec<Duration>)> = by_statement.into_iter().collect();
    statements.sort_by_key(|(_, durations)| std::cmp::Reverse(durations.len()));
    for (statement, mut durations) in statements {
        let count = durations.len() as f64 / requests;
        println!("{}", row(&statement, count, &mut durations));
    }
}

#[test]
#[ignore = "measurement, requires PostgreSQL — run: cargo test -p ferriskey-api --test client_credentials_profile_test -- --ignored --nocapture"]
fn where_the_time_of_a_client_credentials_grant_goes() {
    PROFILER.get_or_init(|| {
        tracing_subscriber::registry().with(Profiler).init();
    });
    let app = cached_app();

    rt().block_on(async {
        let server = app.server();
        let client = app
            .client(ClientSpec::confidential().with_service_account())
            .await;
        app.bind_service_account(&server, &client).await;
        sqlx::query(
            "INSERT INTO client_scope_mappings (client_id, client_scope_id, default_scope_type) \
             SELECT $1, id, 'DEFAULT' FROM client_scopes \
             WHERE realm_id = $2 AND name IN ('openid', 'profile', 'email', 'roles')",
        )
        .bind(client.id)
        .bind(app.realm_id)
        .execute(&app.pool)
        .await
        .expect("attach the realm default scopes, as the client service does");

        for _ in 0..WARM_UP {
            let response = client_credentials(&server, app, &client).await;
            assert_eq!(response.status_code(), 200, "{}", response.text());
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
        {
            let mut timings = TIMINGS.lock().expect("timings poisoned");
            timings.spans.clear();
            timings.statements.clear();
        }

        let mut wall = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let started = Instant::now();
            let response = client_credentials(&server, app, &client).await;
            wall.push(started.elapsed());
            assert_eq!(response.status_code(), 200, "{}", response.text());
        }
        tokio::time::sleep(Duration::from_millis(400)).await;

        report(wall);
    });
}
