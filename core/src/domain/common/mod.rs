use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric};
use uuid::{NoContext, Timestamp, Uuid};

pub mod entities;
pub mod policies;

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone)]
pub struct FerriskeyConfig {
    pub database: DatabaseConfig,
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}

pub fn generate_timestamp() -> (DateTime<Utc>, Timestamp) {
    let now = Utc::now();
    let seconds = now.timestamp().try_into().unwrap_or(0);
    let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

    (now, timestamp)
}

pub fn generate_uuid_v7() -> Uuid {
    let (_, timestamp) = generate_timestamp();
    Uuid::new_v7(timestamp)
}

pub fn generate_random_string() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}
