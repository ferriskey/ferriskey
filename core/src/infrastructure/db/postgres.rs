use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);
pub const ASYNC_COMMIT_CONNECTIONS: u32 = 4;

#[derive(Debug, Clone)]
pub struct Postgres {
    db: DatabaseConnection,
    async_commit_db: DatabaseConnection,
}

pub struct PostgresConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    /// Refresh token issuance and rotation also skip waiting for the disk. The async commit
    /// pool then grows by `max_connections`, since every refresh writes through it.
    pub async_refresh_tokens: bool,
}

impl Postgres {
    pub async fn new(config: PostgresConfig) -> Result<Self, anyhow::Error> {
        let db = Database::connect(connect_options(&config)).await?;
        let async_commit_db = Database::connect(async_commit_options(&config)).await?;

        Ok(Self {
            db,
            async_commit_db,
        })
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }

    /// Commits on this pool return before the WAL reaches the disk. Only for rows whose loss
    /// on a database crash is acceptable: the access token index, and refresh token rotations
    /// when `async_refresh_tokens` is on.
    pub fn get_async_commit_db(&self) -> DatabaseConnection {
        self.async_commit_db.clone()
    }
}

fn connect_options(config: &PostgresConfig) -> ConnectOptions {
    connect_options_releasing_after(config, IDLE_CONNECTION_TIMEOUT)
}

fn connect_options_releasing_after(config: &PostgresConfig, idle: Duration) -> ConnectOptions {
    let mut opt = ConnectOptions::new(config.database_url.clone());
    opt.max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .idle_timeout(idle);
    opt.sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Debug);
    opt
}

fn async_commit_options(config: &PostgresConfig) -> ConnectOptions {
    let refresh_connections = if config.async_refresh_tokens {
        config.max_connections
    } else {
        0
    };
    let mut opt = connect_options(config);
    opt.max_connections(ASYNC_COMMIT_CONNECTIONS + refresh_connections)
        .min_connections(0)
        .map_sqlx_postgres_opts(|options| options.options([("synchronous_commit", "off")]));
    opt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(max_connections: u32, min_connections: u32) -> PostgresConfig {
        PostgresConfig {
            database_url: "postgres://u:p@localhost:5432/ferriskey".to_string(),
            max_connections,
            min_connections,
            async_refresh_tokens: false,
        }
    }

    #[test]
    fn the_pool_follows_the_configured_bounds() {
        let options = connect_options(&config(12, 3));

        assert_eq!(options.get_max_connections(), Some(12));
        assert_eq!(options.get_min_connections(), Some(3));
    }

    #[test]
    fn an_idle_connection_is_released_within_a_minute() {
        let options = connect_options(&config(20, 2));

        assert_eq!(options.get_idle_timeout(), Some(Duration::from_secs(60)));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_pool_that_grew_for_a_burst_shrinks_back_to_its_minimum() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let config = PostgresConfig {
            database_url,
            max_connections: 6,
            min_connections: 1,
            async_refresh_tokens: false,
        };
        let db = Database::connect(connect_options_releasing_after(
            &config,
            Duration::from_secs(2),
        ))
        .await
        .expect("connect");
        let pool = db.get_postgres_connection_pool();

        let mut burst = Vec::new();
        for _ in 0..6 {
            burst.push(pool.acquire().await.expect("a connection"));
        }
        drop(burst);
        assert_eq!(pool.size(), 6);

        let shrunk = tokio::time::timeout(Duration::from_secs(30), async {
            while pool.size() > 1 {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        })
        .await;

        assert!(
            shrunk.is_ok(),
            "the pool still holds {} connections",
            pool.size()
        );
    }

    #[test]
    fn the_async_commit_pool_holds_one_connection_per_access_token_writer() {
        let options = async_commit_options(&config(20, 2));

        assert_eq!(
            options.get_max_connections(),
            Some(ASYNC_COMMIT_CONNECTIONS)
        );
        assert_eq!(options.get_min_connections(), Some(0));
    }

    #[test]
    fn async_refresh_tokens_give_every_request_its_own_async_commit_connection() {
        let options = async_commit_options(&PostgresConfig {
            async_refresh_tokens: true,
            ..config(20, 2)
        });

        assert_eq!(
            options.get_max_connections(),
            Some(ASYNC_COMMIT_CONNECTIONS + 20)
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn only_the_async_commit_pool_skips_waiting_for_the_disk() {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://ferriskey:ferriskey@localhost:5432/ferriskey".to_string()
        });
        let postgres = Postgres::new(PostgresConfig {
            database_url,
            max_connections: 2,
            min_connections: 1,
            async_refresh_tokens: false,
        })
        .await
        .expect("connect");

        assert_eq!(synchronous_commit(&postgres.get_db()).await, "on");
        assert_eq!(
            synchronous_commit(&postgres.get_async_commit_db()).await,
            "off"
        );
    }

    async fn synchronous_commit(db: &DatabaseConnection) -> String {
        use sea_orm::{ConnectionTrait, Statement};

        db.query_one(Statement::from_string(
            db.get_database_backend(),
            "SHOW synchronous_commit",
        ))
        .await
        .expect("query")
        .expect("a row")
        .try_get_by_index::<String>(0)
        .expect("a setting")
    }
}
