use std::future::Future;
use std::time::Duration;

use chrono::{DateTime, Utc};
use ferriskey_security::jwt::ports::{AccessTokenRepository, RefreshTokenRepository};

use crate::domain::session::ports::UserSessionRepository;

pub const EXPIRED_ROW_RETENTION_DAYS: i64 = 7;

pub const SWEEP_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub const RETENTION_BATCH_SIZE: u64 = 1_000;

const MAX_BATCHES_PER_SWEEP: u64 = 1_000;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Swept {
    pub access_tokens: u64,
    pub refresh_tokens: u64,
    pub user_sessions: u64,
}

async fn purge_in_batches<F, Fut, E>(table: &str, batch: u64, mut delete_batch: F) -> u64
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<u64, E>>,
    E: std::fmt::Debug,
{
    let mut purged = 0;
    for _ in 0..MAX_BATCHES_PER_SWEEP {
        match delete_batch().await {
            Ok(deleted) => {
                purged += deleted;
                if deleted < batch {
                    return purged;
                }
            }
            Err(e) => {
                tracing::error!(error = ?e, table, "Retention: failed to purge expired rows");
                return purged;
            }
        }
        tokio::task::yield_now().await;
    }

    tracing::warn!(
        table,
        purged,
        "Retention: stopped after the maximum number of batches, the rest waits for the next sweep"
    );
    purged
}

pub async fn sweep_expired_rows<A, R, S>(
    access_tokens: &A,
    refresh_tokens: &R,
    user_sessions: &S,
    before: DateTime<Utc>,
    batch: u64,
) -> Swept
where
    A: AccessTokenRepository,
    R: RefreshTokenRepository,
    S: UserSessionRepository,
{
    Swept {
        access_tokens: purge_in_batches("access_tokens", batch, || {
            access_tokens.delete_expired(before, batch)
        })
        .await,
        refresh_tokens: purge_in_batches("refresh_tokens", batch, || {
            refresh_tokens.delete_expired(before, batch)
        })
        .await,
        user_sessions: purge_in_batches("user_sessions", batch, || {
            user_sessions.delete_expired(before, batch)
        })
        .await,
    }
}

pub async fn expired_rows_retention_task<A, R, S>(
    access_tokens: A,
    refresh_tokens: R,
    user_sessions: S,
) where
    A: AccessTokenRepository,
    R: RefreshTokenRepository,
    S: UserSessionRepository,
{
    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        ticker.tick().await;

        let before = Utc::now() - chrono::Duration::days(EXPIRED_ROW_RETENTION_DAYS);
        let swept = sweep_expired_rows(
            &access_tokens,
            &refresh_tokens,
            &user_sessions,
            before,
            RETENTION_BATCH_SIZE,
        )
        .await;

        if swept != Swept::default() {
            tracing::info!(
                access_tokens = swept.access_tokens,
                refresh_tokens = swept.refresh_tokens,
                user_sessions = swept.user_sessions,
                retention_days = EXPIRED_ROW_RETENTION_DAYS,
                "Retention: purged rows expired past the retention window"
            );
        }
    }
}

#[cfg(test)]
mod db_tests {
    use chrono::{Duration, Utc};
    use ferriskey_security::jwt::ports::{AccessTokenRepository, RefreshTokenRepository};
    use sea_orm::Database as SeaOrmDatabase;
    use sqlx::Executor as _;
    use uuid::Uuid;

    use super::{Swept, sweep_expired_rows};
    use crate::domain::realm::entities::RealmId;
    use crate::domain::session::entities::UserSession;
    use crate::domain::session::ports::UserSessionRepository;
    use crate::infrastructure::repositories::access_token_repository::PostgresAccessTokenRepository;
    use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;
    use crate::infrastructure::repositories::user_session_repository::PostgresUserSessionRepository;

    struct Fixture {
        access_tokens: PostgresAccessTokenRepository,
        refresh_tokens: PostgresRefreshTokenRepository,
        user_sessions: PostgresUserSessionRepository,
        realm_id: Uuid,
        user_id: Uuid,
    }

    async fn setup() -> Fixture {
        let base_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let schema = format!("retention_test_{}", Uuid::new_v4().simple());

        let admin_pool = sqlx::PgPool::connect(&base_url)
            .await
            .expect("connect admin pool");
        admin_pool
            .execute(sqlx::query(&format!(r#"CREATE SCHEMA "{}""#, schema)))
            .await
            .expect("create test schema");

        let separator = if base_url.contains('?') { '&' } else { '?' };
        let schema_url = format!("{base_url}{separator}options=-c search_path={schema}");
        let pool = sqlx::PgPool::connect(&schema_url)
            .await
            .expect("connect schema pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");

        let realm_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())",
        )
        .bind(realm_id)
        .bind(format!("realm-{}", realm_id.simple()))
        .execute(&pool)
        .await
        .expect("insert realm");
        let user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, realm_id, username, email_verified, enabled, created_at, updated_at) \
             VALUES ($1, $2, $3, TRUE, TRUE, NOW(), NOW())",
        )
        .bind(user_id)
        .bind(realm_id)
        .bind(format!("user-{}", user_id.simple()))
        .execute(&pool)
        .await
        .expect("insert user");

        let db = SeaOrmDatabase::connect(&schema_url)
            .await
            .expect("sea-orm connect");

        Fixture {
            access_tokens: PostgresAccessTokenRepository::new(db.clone()),
            refresh_tokens: PostgresRefreshTokenRepository::new(db.clone()),
            user_sessions: PostgresUserSessionRepository::new(db),
            realm_id,
            user_id,
        }
    }

    async fn access_token(fixture: &Fixture, expires_at: chrono::DateTime<Utc>) -> String {
        let hash = format!("hash-{}", Uuid::new_v4().simple());
        fixture
            .access_tokens
            .create(
                hash.clone(),
                Some(Uuid::new_v4()),
                fixture.user_id,
                RealmId::from(fixture.realm_id),
                Some(expires_at),
                serde_json::json!({}),
            )
            .await
            .expect("create access token");
        hash
    }

    async fn refresh_token(fixture: &Fixture, expires_at: chrono::DateTime<Utc>) -> Uuid {
        let jti = Uuid::new_v4();
        fixture
            .refresh_tokens
            .create(
                jti,
                fixture.user_id,
                Some(expires_at),
                None,
                None,
                None,
                None,
                None,
            )
            .await
            .expect("create refresh token");
        jti
    }

    async fn user_session(fixture: &Fixture, lifetime: Duration) -> Uuid {
        let session = UserSession::new(
            fixture.user_id,
            fixture.realm_id,
            None,
            None,
            lifetime,
            None,
        );
        fixture
            .user_sessions
            .create(&session)
            .await
            .expect("create user session");
        session.id
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sweep_removes_only_the_rows_expired_before_the_cutoff() {
        let fixture = setup().await;
        let now = Utc::now();
        let long_expired = now - Duration::days(10);
        let just_expired = now - Duration::hours(1);

        let gone_access = access_token(&fixture, long_expired).await;
        let also_gone_access = access_token(&fixture, long_expired).await;
        let kept_access = access_token(&fixture, just_expired).await;
        let live_access = access_token(&fixture, now + Duration::hours(1)).await;
        let gone_refresh = refresh_token(&fixture, long_expired).await;
        let kept_refresh = refresh_token(&fixture, now + Duration::days(30)).await;
        let gone_session = user_session(&fixture, Duration::days(-10)).await;
        let kept_session = user_session(&fixture, Duration::hours(8)).await;

        let swept = sweep_expired_rows(
            &fixture.access_tokens,
            &fixture.refresh_tokens,
            &fixture.user_sessions,
            now - Duration::days(7),
            1,
        )
        .await;

        assert_eq!(
            swept,
            Swept {
                access_tokens: 2,
                refresh_tokens: 1,
                user_sessions: 1,
            }
        );
        for hash in [gone_access, also_gone_access] {
            assert!(
                fixture
                    .access_tokens
                    .get_by_token_hash(hash)
                    .await
                    .expect("read")
                    .is_none()
            );
        }
        for hash in [kept_access, live_access] {
            assert!(
                fixture
                    .access_tokens
                    .get_by_token_hash(hash)
                    .await
                    .expect("read")
                    .is_some()
            );
        }
        assert!(
            fixture
                .refresh_tokens
                .get_by_jti(gone_refresh)
                .await
                .is_err()
        );
        assert!(
            fixture
                .refresh_tokens
                .get_by_jti(kept_refresh)
                .await
                .is_ok()
        );
        assert!(
            fixture
                .user_sessions
                .find_by_id(gone_session)
                .await
                .expect("read")
                .is_none()
        );
        assert!(
            fixture
                .user_sessions
                .find_by_id(kept_session)
                .await
                .expect("read")
                .is_some()
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sweep_over_an_empty_realm_removes_nothing() {
        let fixture = setup().await;

        let swept = sweep_expired_rows(
            &fixture.access_tokens,
            &fixture.refresh_tokens,
            &fixture.user_sessions,
            Utc::now(),
            super::RETENTION_BATCH_SIZE,
        )
        .await;

        assert_eq!(swept, Swept::default());
    }
}
