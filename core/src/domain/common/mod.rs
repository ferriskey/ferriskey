pub mod entities;
pub mod policies;
pub mod ports;
pub mod services;

pub use ferriskey_domain::{generate_random_string, generate_timestamp, generate_uuid_v7};

pub struct AppConfig {
    pub database_url: String,
}

#[derive(Clone, Debug)]
pub struct FerriskeyConfig {
    pub database: DatabaseConfig,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub name: String,
}
