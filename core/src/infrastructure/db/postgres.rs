use std::time::Duration;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub const IDLE_CONNECTION_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
pub struct Postgres {
    db: DatabaseConnection,
}

pub struct PostgresConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Postgres {
    pub async fn new(config: PostgresConfig) -> Result<Self, anyhow::Error> {
        let db = Database::connect(connect_options(&config)).await?;

        Ok(Self { db })
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config(max_connections: u32, min_connections: u32) -> PostgresConfig {
        PostgresConfig {
            database_url: "postgres://u:p@localhost:5432/ferriskey".to_string(),
            max_connections,
            min_connections,
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
}
