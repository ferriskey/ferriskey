use sea_orm::{ConnectOptions, Database, DatabaseConnection};

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
        let mut opt = ConnectOptions::new(config.database_url.clone());
        opt.max_connections(config.max_connections)
            .min_connections(config.min_connections);
        opt.sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);

        let db = Database::connect(opt).await?;

        Ok(Self { db })
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}
