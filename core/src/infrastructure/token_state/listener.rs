use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use sqlx::postgres::{PgListener, PgPoolOptions};
use tokio::time::{Instant, interval, sleep, timeout};
use tracing::warn;
use uuid::Uuid;

use super::state::{ChangeFeed, TokenState};

const CHANNEL: &str = "ferriskey_token_state";
const HEARTBEAT_PREFIX: &str = "ping:";
const HEARTBEAT_EVERY: Duration = Duration::from_secs(15);
const SILENCE_LIMIT: Duration = Duration::from_secs(45);
const ANNOUNCE_TIMEOUT: Duration = Duration::from_secs(5);
const FIRST_RETRY: Duration = Duration::from_secs(1);
const LONGEST_RETRY: Duration = Duration::from_secs(30);

/// Keeps `state` in step with replicas of its schema.
///
/// NOTIFY reaches every session of the database.
/// Each announcement names its schema: `<schema>/<replica>`.
/// The schema is named as PostgreSQL resolves it.
/// A replica ignores announcements of another schema.
pub fn spawn_token_state_sync(
    pool: &PgPool,
    schema: &str,
    state: Arc<TokenState>,
    feed: ChangeFeed,
) {
    let options = pool.connect_options().as_ref().clone();
    let announcing = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(ANNOUNCE_TIMEOUT)
        .connect_lazy_with(options.clone());
    let listening = PgPoolOptions::new()
        .max_connections(1)
        .connect_lazy_with(options);
    let origin = format!("{}/{}", canonical_schema(schema), Uuid::new_v4());

    tokio::spawn(publish(announcing, origin.clone(), feed));
    tokio::spawn(listen(listening, origin, state));
}

async fn publish(pool: PgPool, origin: String, mut feed: ChangeFeed) {
    let mut heartbeat = interval(HEARTBEAT_EVERY);

    loop {
        tokio::select! {
            more = feed.next() => {
                if !more {
                    return;
                }
                announce_until_sent(&pool, &origin).await;
            }
            _ = heartbeat.tick() => {
                let _ = announce(&pool, &format!("{HEARTBEAT_PREFIX}{origin}")).await;
            }
        }
    }
}

async fn announce_until_sent(pool: &PgPool, payload: &str) {
    let mut retry = FIRST_RETRY;

    while !announce(pool, payload).await {
        sleep(retry).await;
        retry = (retry * 2).min(LONGEST_RETRY);
    }
}

async fn announce(pool: &PgPool, payload: &str) -> bool {
    let sent = timeout(
        ANNOUNCE_TIMEOUT,
        sqlx::query("SELECT pg_notify($1, $2)")
            .bind(CHANNEL)
            .bind(payload)
            .execute(pool),
    )
    .await;

    match sent {
        Ok(Ok(_)) => true,
        Ok(Err(error)) => {
            warn!(%error, "could not tell the other replicas that tokens or sessions changed");
            false
        }
        Err(_) => {
            warn!("timed out telling the other replicas that tokens or sessions changed");
            false
        }
    }
}

async fn listen(pool: PgPool, origin: String, state: Arc<TokenState>) {
    let mut retry = FIRST_RETRY;

    loop {
        match subscribe(&pool).await {
            Ok(mut listener) => {
                retry = FIRST_RETRY;
                state.changed_elsewhere();
                state.set_synced(true);
                relay(&mut listener, &origin, &state, SILENCE_LIMIT).await;
                state.set_synced(false);
            }
            Err(error) => warn!(%error, "could not listen for token and session changes"),
        }

        sleep(retry).await;
        retry = (retry * 2).min(LONGEST_RETRY);
    }
}

async fn subscribe(pool: &PgPool) -> Result<PgListener, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(CHANNEL).await?;

    Ok(listener)
}

async fn relay(listener: &mut PgListener, origin: &str, state: &TokenState, silence: Duration) {
    let quiet = sleep(silence);
    tokio::pin!(quiet);

    loop {
        tokio::select! {
            received = listener.try_recv() => match received {
                Ok(Some(notification)) => react(notification.payload(), origin, state),
                Ok(None) => state.changed_elsewhere(),
                Err(error) => {
                    warn!(%error, "the token and session change listener stopped");
                    return;
                }
            },
            _ = &mut quiet => {
                warn!("the token and session change listener heard nothing, reconnecting");
                return;
            }
        }
        quiet.as_mut().reset(Instant::now() + silence);
    }
}

fn react(payload: &str, origin: &str, state: &TokenState) {
    if payload == origin || payload.starts_with(HEARTBEAT_PREFIX) {
        return;
    }

    // A replica older than this change announces no schema.
    // Its announcement still empties the caches.
    // Ignoring it would serve revoked sessions until expiry.
    if let (Some(theirs), Some(ours)) = (schema_of(payload), schema_of(origin))
        && theirs != ours
    {
        return;
    }

    state.changed_elsewhere();
}

/// The schema an announcement names, if any.
fn schema_of(announcement: &str) -> Option<&str> {
    announcement.rsplit_once('/').map(|(schema, _)| schema)
}

/// The schema as PostgreSQL resolves it from the search_path.
///
/// An unquoted name is folded to lowercase.
/// A quoted name keeps its case.
/// Inside it, `""` stands for one quote.
fn canonical_schema(schema: &str) -> String {
    match schema
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        Some(quoted) => quoted.replace("\"\"", "\""),
        None => schema.to_ascii_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::PgPool;
    use tokio::sync::watch;

    use super::*;

    const LISTENING_AT_LAST: Duration = Duration::from_secs(20);

    #[test]
    fn a_foreign_announcement_empties_the_caches() {
        let (state, _feed) = TokenState::new(Duration::from_secs(60));

        react("another-replica", "this-replica", &state);

        assert_eq!(state.generation(), 1);
    }

    #[test]
    fn a_replica_ignores_its_own_announcement_and_every_heartbeat() {
        let (state, _feed) = TokenState::new(Duration::from_secs(60));

        react("this-replica", "this-replica", &state);
        react("ping:another-replica", "this-replica", &state);
        react("ping:this-replica", "this-replica", &state);

        assert_eq!(state.generation(), 0);
    }

    #[test]
    fn an_announcement_of_another_schema_leaves_the_caches_alone() {
        let (state, _feed) = TokenState::new(Duration::from_secs(60));

        react(
            "oidc_other/another-replica",
            "oidc_mine/this-replica",
            &state,
        );

        assert_eq!(state.generation(), 0);
    }

    #[test]
    fn an_announcement_of_the_same_schema_or_without_one_empties_the_caches() {
        let (state, _feed) = TokenState::new(Duration::from_secs(60));

        react(
            "oidc_mine/another-replica",
            "oidc_mine/this-replica",
            &state,
        );
        react("another-replica", "oidc_mine/this-replica", &state);

        assert_eq!(state.generation(), 2);
    }

    #[test]
    fn an_unquoted_schema_is_folded_to_lowercase_as_postgresql_does() {
        assert_eq!(canonical_schema("PUBLIC"), canonical_schema("public"));
        assert_eq!(canonical_schema("Oidc_Mine"), "oidc_mine");
    }

    #[test]
    fn a_quoted_schema_keeps_its_case() {
        assert_eq!(canonical_schema("\"Public\""), "Public");
        assert_ne!(canonical_schema("\"Public\""), canonical_schema("public"));
        assert_eq!(canonical_schema("\"a\"\"B\""), "a\"B");
        assert_eq!(canonical_schema("\"public\""), canonical_schema("PUBLIC"));
    }

    #[test]
    fn announcements_of_two_spellings_of_one_schema_empty_the_caches() {
        let (state, _feed) = TokenState::new(Duration::from_secs(60));
        let theirs = format!("{}/another-replica", canonical_schema("PUBLIC"));
        let ours = format!("{}/this-replica", canonical_schema("public"));

        react(&theirs, &ours, &state);

        assert_eq!(state.generation(), 1);
    }

    async fn replica(pool: &PgPool, schema: &str) -> Arc<TokenState> {
        let (state, feed) = TokenState::new(Duration::from_secs(60));
        let mut ready = state.subscribe();
        spawn_token_state_sync(pool, schema, Arc::clone(&state), feed);

        wait_past(&mut ready, 0).await;
        state
    }

    async fn wait_past(generation: &mut watch::Receiver<u64>, seen: u64) {
        tokio::time::timeout(LISTENING_AT_LAST, generation.wait_for(|now| *now > seen))
            .await
            .expect("the generation moved in time")
            .expect("the state is alive");
    }

    async fn pool() -> PgPool {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });

        PgPool::connect(&url).await.expect("connect")
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_change_on_one_replica_empties_the_other() {
        let pool = pool().await;
        let first = replica(&pool, "public").await;
        let second = replica(&pool, "public").await;
        let mut watching = second.subscribe();
        let seen = second.generation();

        first.changed();

        wait_past(&mut watching, seen).await;
        assert!(second.synced());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_change_reaches_a_replica_whose_schema_is_spelled_in_another_case() {
        let pool = pool().await;
        let first = replica(&pool, "public").await;
        let second = replica(&pool, "PUBLIC").await;
        let mut watching = second.subscribe();
        let seen = second.generation();

        first.changed();

        wait_past(&mut watching, seen).await;
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_change_in_one_schema_leaves_the_replicas_of_another_schema_alone() {
        let pool = pool().await;
        let mine = format!("token_state_{}", Uuid::new_v4().simple());
        let theirs = format!("token_state_{}", Uuid::new_v4().simple());
        let first = replica(&pool, &mine).await;
        let sibling = replica(&pool, &mine).await;
        let stranger = replica(&pool, &theirs).await;
        let (mut sibling_moves, sibling_seen) = (sibling.subscribe(), sibling.generation());
        let (mut stranger_moves, stranger_seen) = (stranger.subscribe(), stranger.generation());

        first.changed();

        // The sibling hearing the change proves it was sent.
        // The stranger shares the channel, so it heard it too.
        wait_past(&mut sibling_moves, sibling_seen).await;
        let stranger_moved = tokio::time::timeout(
            Duration::from_secs(1),
            stranger_moves.wait_for(|now| *now > stranger_seen),
        )
        .await
        .is_ok();
        assert!(
            !stranger_moved,
            "a change in another schema emptied the caches"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_listener_that_hears_nothing_gives_up_instead_of_waiting_forever() {
        let pool = pool().await;
        let (state, _feed) = TokenState::new(Duration::from_secs(60));
        let mut listener = subscribe(&pool).await.expect("subscribe");

        let returned = tokio::time::timeout(
            LISTENING_AT_LAST,
            relay(&mut listener, "origin", &state, Duration::from_millis(300)),
        )
        .await;

        assert!(returned.is_ok(), "the relay never gave up");
    }
}
