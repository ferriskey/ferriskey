#![allow(deprecated)]

use std::fmt::Display;

use clap::{Parser, ValueEnum};
use url::Url;

#[derive(Debug, Clone, ValueEnum, Default)]
#[deprecated]
pub enum Environment {
    #[default]
    Development,
    Production,
}

impl From<String> for Environment {
    fn from(value: String) -> Self {
        match value.as_str() {
            "development" => Environment::Development,
            "production" => Environment::Production,
            _ => Environment::Development, // Default to Development if unknown
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(about, version)]
pub struct Args {
    #[command(flatten)]
    pub admin: AdminArgs,
    #[command(flatten)]
    pub db: DatabaseArgs,
    #[arg(
        short,
        long,
        env,
        long_help = "The environment to run the application in",
        default_value_t = Environment::Development,
    )]
    pub env: Environment,
    #[command(flatten)]
    pub log: LogArgs,
    #[command(flatten)]
    pub server: ServerArgs,
    #[arg(
        long,
        env,
        default_value = "http://localhost:5555",
        long_help = "The url to the webapp to use"
    )]
    pub webapp_url: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            admin: AdminArgs::default(),
            db: DatabaseArgs::default(),
            env: Environment::Development,
            log: LogArgs::default(),
            server: ServerArgs::default(),
            webapp_url: "http://localhost:5555".to_string(),
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct AdminArgs {
    #[arg(
        long = "admin-password",
        env = "ADMIN_PASSWORD",
        default_value = "admin",
        name = "ADMIN_PASSWORD",
        long_help = "The admin password to use"
    )]
    pub password: String,
    #[arg(
        long = "admin-email",
        env = "ADMIN_EMAIL",
        default_value = "admin@local",
        name = "ADMIN_EMAIL",
        long_help = "The admin email to use"
    )]
    pub email: String,
    #[arg(
        long = "admin-username",
        env = "ADMIN_USERNAME",
        default_value = "admin",
        name = "ADMIN_USERNAME",
        long_help = "The admin username to use"
    )]
    pub username: String,
}

impl Default for AdminArgs {
    fn default() -> Self {
        Self {
            password: "admin".to_string(),
            email: "admin@local".to_string(),
            username: "admin".to_string(),
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct DatabaseArgs {
    #[arg(
        long = "database-host",
        env = "DATABASE_HOST",
        default_value = "localhost",
        name = "DATABASE_HOST",
        long_help = "The database host to use"
    )]
    pub host: String,
    #[arg(
        long = "database-name",
        env = "DATABASE_NAME",
        default_value = "ferriskey",
        name = "DATABASE_NAME",
        long_help = "The database name to use"
    )]
    pub name: String,
    #[arg(
        long = "database-password",
        env = "DATABASE_PASSWORD",
        default_value = "postgres",
        name = "DATABASE_PASSWORD",
        long_help = "The database password to use"
    )]
    pub password: String,
    #[arg(
        long = "database-port",
        env = "DATABASE_PORT",
        default_value_t = 5432,
        name = "DATABASE_PORT",
        long_help = "The database port to use"
    )]
    pub port: u16,
    #[arg(
        long = "database-user",
        env = "DATABASE_USER",
        default_value = "postgres",
        name = "DATABASE_USER",
        long_help = "The database user to use"
    )]
    pub user: String,
}

impl Default for DatabaseArgs {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            name: "ferriskey".to_string(),
            password: "postgres".to_string(),
            port: 5432,
            user: "postgres".to_string(),
        }
    }
}

impl From<Url> for DatabaseArgs {
    fn from(value: Url) -> Self {
        Self {
            host: value
                .host()
                .unwrap_or(url::Host::Domain("localhost"))
                .to_string(),
            name: value.path().to_string(),
            password: value.password().unwrap_or("").to_string(),
            port: value.port().unwrap_or(5432),
            user: value.username().to_string(),
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct LogArgs {
    #[arg(
        long = "log-filter",
        env = "LOG_FILTER",
        name = "LOG_FILTER",
        long_help = "The log filter to use\nhttps://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives",
        default_value = "info"
    )]
    pub filter: String,
    #[arg(
        long = "log-json",
        env = "LOG_JSON",
        name = "LOG_JSON",
        long_help = "Whether to log in JSON format"
    )]
    pub json: bool,
}

impl Default for LogArgs {
    fn default() -> Self {
        Self {
            filter: "info".to_string(),
            json: false,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
pub struct ServerArgs {
    #[arg(
        short,
        long,
        env,
        num_args = 0..,
        value_delimiter = ',',
        long_help = "The port to run the application on",
    )]
    pub allowed_origins: Vec<String>,
    #[arg(
        short = 'H',
        long = "server-host",
        env = "SERVER_HOST",
        name = "SERVER_HOST",
        default_value = "0.0.0.0",
        long_help = "The host to run the application on"
    )]
    pub host: String,
    #[arg(
        short = 'P',
        long = "server-port",
        env = "SERVER_PORT",
        name = "SERVER_PORT",
        default_value_t = 3333,
        long_help = "The port to run the application on"
    )]
    pub port: u16,
}

impl Default for ServerArgs {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            host: "0.0.0.0".to_string(),
            port: 3333,
        }
    }
}
