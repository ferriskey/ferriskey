use chrono::{DateTime, Utc};
use rand::{Rng, distributions::Alphanumeric};
use uuid::{NoContext, Timestamp, Uuid};

pub mod email;
pub mod entities;
pub mod policies;
pub mod ports;
pub mod services;

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone, Debug)]
pub struct FerriskeyConfig {
    pub database: DatabaseConfig,
    pub encryption: EncryptionConfig,
}

#[derive(Clone, Debug, Default)]
pub struct EncryptionConfig {
    /// When `true`, client secrets are encrypted at rest using AES-256-GCM.
    /// Set `false` to run without encryption (dev/test only).
    pub enabled: bool,
    /// Name of the secret to fetch from the `SecretsProvider`.
    /// The resolved value must be a 32-byte key, hex-encoded (64 hex chars).
    pub master_key_secret_name: String,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
    pub schema: String,
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

pub fn generate_random_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(43)
        .map(char::from)
        .collect()
}
