use std::sync::Arc;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::PgPool;

use crate::env::Env;

#[derive(Debug, Clone)]
pub struct Postgres {
    pub pool: PgPool,
    pub db: DatabaseConnection,
}

impl Postgres {
    pub async fn new(env: Arc<Env>) -> Result<Self, anyhow::Error> {
        let pool = PgPool::connect(&env.database_url).await?;

        let mut opt = ConnectOptions::new(env.database_url.clone());

        opt.max_connections(100).min_connections(5);
        opt.sqlx_logging(false);

        let db = Database::connect(opt).await?;

        Ok(Self { pool, db })
    }

    pub fn get_pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }
}
