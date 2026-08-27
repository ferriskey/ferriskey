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
    /// Public origin the admin console is served from. Seeding needs it to register
    /// the console's callback as an exact redirect URI — see [`console_callback_uri`].
    pub webapp_url: String,
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

/// The one redirect URI the admin console ever needs, for a given realm.
///
/// Seeded as an exact literal rather than a pattern: the console's callback route is
/// `/realms/{realm}/authentication/callback` (see `front/src/pages/authentication`),
/// so the full URI is known at seeding time and no wildcard is warranted. FerrisKey
/// used to seed `^/*` here, which matched every URI on earth (FK-002).
pub fn console_callback_uri(webapp_url: &str, realm_name: &str) -> String {
    format!(
        "{}/realms/{realm_name}/authentication/callback",
        webapp_url.trim_end_matches('/')
    )
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
