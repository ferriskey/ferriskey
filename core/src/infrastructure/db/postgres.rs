use sea_orm::{ConnectOptions, Database, DatabaseConnection};

#[derive(Debug, Clone)]
pub struct Postgres {
    db: DatabaseConnection,
}

pub struct PostgresConfig {
    pub database_url: String,
}

impl Postgres {
    pub async fn new(config: PostgresConfig) -> Result<Self, anyhow::Error> {
        let mut opt = ConnectOptions::new(config.database_url.clone());
        opt.max_connections(100).min_connections(5);
        opt.sqlx_logging(false);

        let db = Database::connect(opt).await?;

        Ok(Self { db })
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}
