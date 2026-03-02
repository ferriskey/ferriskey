#![allow(deprecated)]

use std::{fmt::Display, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use ferriskey_core::domain::common::{DatabaseConfig, FerriskeyConfig};
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

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Generate the OpenAPI specification and print it to stdout (or a file).
    /// Does not require a running database.
    GenApi {
        /// Write the spec to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
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
    #[command(flatten)]
    pub observability: ObservabilityArgs,
    #[command(subcommand)]
    pub command: Option<Command>,
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
            observability: ObservabilityArgs::default(),
            command: None,
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
        default_value = "ferriskey",
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
        default_value = "ferriskey",
        name = "DATABASE_USER",
        long_help = "The database user to use"
    )]
    pub user: String,
    #[arg(
        long = "database-schema",
        env = "DATABASE_SCHEMA",
        default_value = "public",
        name = "DATABASE_SCHEMA",
        long_help = "The database schema to use"
    )]
    pub schema: String,
}

impl Default for DatabaseArgs {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            name: "ferriskey".to_string(),
            password: "postgres".to_string(),
            port: 5432,
            user: "postgres".to_string(),
            schema: "public".to_string(),
        }
    }
}

impl From<Url> for DatabaseArgs {
    fn from(value: Url) -> Self {
        // Parse schema from query parameters if available
        let schema = value
            .query_pairs()
            .find(|(key, _)| key == "schema")
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "public".to_string());

        Self {
            host: value
                .host()
                .unwrap_or(url::Host::Domain("localhost"))
                .to_string(),
            name: value.path().to_string(),
            password: value.password().unwrap_or("").to_string(),
            port: value.port().unwrap_or(5432),
            user: value.username().to_string(),
            schema,
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
    #[arg(
        long = "server-root-path",
        env = "SERVER_ROOT_PATH",
        name = "SERVER_ROOT_PATH",
        default_value = "",
        long_help = "The root path to run the application on",
        value_parser = parse_root_path,
    )]
    pub root_path: String,
    #[command(flatten)]
    pub tls: Option<ServerTlsArgs>,
}

impl Default for ServerArgs {
    fn default() -> Self {
        Self {
            allowed_origins: vec![],
            host: "0.0.0.0".into(),
            port: 3333,
            root_path: String::new(),
            tls: None,
        }
    }
}

#[derive(clap::Args, Debug, Clone)]
#[group(requires_all = ["SERVER_TLS_CERT", "SERVER_TLS_KEY"])]
pub struct ServerTlsArgs {
    #[arg(
        long = "server-tls-cert",
        env = "SERVER_TLS_CERT",
        name = "SERVER_TLS_CERT",
        long_help = "Path to the TLS cert file in PEM format",
        required = false
    )]
    pub cert: PathBuf,
    #[arg(
        long = "server-tls-key",
        env = "SERVER_TLS_KEY",
        name = "SERVER_TLS_KEY",
        long_help = "Path to the TLS key file in PEM format",
        required = false
    )]
    pub key: PathBuf,
}

#[derive(clap::Args, Debug, Clone)]
pub struct ObservabilityArgs {
    #[arg(
        long = "active-observability",
        env = "ACTIVE_OBSERVABILITY",
        name = "ACTIVE_OBSERVABILITY",
        default_value_t = false,
        long_help = "Whether to enable observability features like tracing and metrics",
        required = false
    )]
    pub active_observability: bool,
    #[arg(
        short = 'O',
        long = "otlp-endpoint",
        env = "OTLP_ENDPOINT",
        name = "OTLP_ENDPOINT",
        long_help = "The endpoint for the traces collector",
        required = false
    )]
    pub otlp_endpoint: Option<String>,
    #[arg(
        short = 'M',
        long = "metrics-endpoint",
        env = "METRICS_ENDPOINT",
        name = "METRICS_ENDPOINT",
        long_help = "The endpoint for the metrics collector",
        required = false
    )]
    pub metrics_endpoint: Option<String>,
}

impl Default for ObservabilityArgs {
    fn default() -> Self {
        Self {
            active_observability: false,
            otlp_endpoint: Some("http://localhost:4317".to_string()),
            metrics_endpoint: Some("http://localhost:4317".to_string()),
        }
    }
}

fn parse_root_path(value: &str) -> Result<String, String> {
    let value = value.trim_end_matches('/');
    if value.is_empty() || value.starts_with('/') {
        Ok(value.into())
    } else {
        Ok(format!("/{value}"))
    }
}

impl From<Args> for FerriskeyConfig {
    fn from(value: Args) -> Self {
        FerriskeyConfig {
            database: DatabaseConfig {
                host: value.db.host,
                name: value.db.name,
                password: value.db.password,
                port: value.db.port,
                username: value.db.user,
                schema: value.db.schema,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_root_path {
        use super::*;

        #[test]
        fn empty() {
            let path = parse_root_path("").unwrap();
            assert_eq!(path, "");
        }

        #[test]
        fn slash() {
            let path = parse_root_path("/").unwrap();
            assert_eq!(path, "");
        }

        #[test]
        fn api() {
            let path = parse_root_path("api").unwrap();
            assert_eq!(path, "/api");
        }

        #[test]
        fn api_slash() {
            let path = parse_root_path("api/").unwrap();
            assert_eq!(path, "/api");
        }

        #[test]
        fn slash_api() {
            let path = parse_root_path("/api").unwrap();
            assert_eq!(path, "/api");
        }

        #[test]
        fn slash_api_slash() {
            let path = parse_root_path("/api/").unwrap();
            assert_eq!(path, "/api");
        }
    }
}
