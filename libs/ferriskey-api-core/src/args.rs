#![allow(deprecated)]

use std::{fmt::Display, num::NonZeroUsize, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use ferriskey_core::domain::common::{DatabaseConfig, FerriskeyConfig, PasswordHashingCost};
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

    #[arg(
        long,
        env = "WEBAPP_URL_FROM_REQUEST",
        default_value_t = false,
        long_help = "Build the webapp links (login and consent pages, emailed links, WebAuthn origin) from the scheme and Host of each request, like the issuer, instead of WEBAPP_URL. For one deployment that serves several hostnames, each with the webapp at / and the API under SERVER_ROOT_PATH. The scheme is https when X-Forwarded-Proto is https, so the proxy must set or overwrite that header. Requires WEBAPP_ALLOWED_HOSTS: the server refuses to start without it, and a request whose Host is missing, repeated, not ASCII or not allowed is refused with 400. A passkey stays bound to the host it was registered on. WEBAPP_URL is still used to seed the admin console at start-up."
    )]
    pub webapp_url_from_request: bool,

    #[arg(
        long,
        env = "WEBAPP_ALLOWED_HOSTS",
        num_args = 0..,
        value_delimiter = ',',
        long_help = "Hosts the webapp is served on when WEBAPP_URL_FROM_REQUEST is on, comma-separated. Each entry is host, host:port, or *.domain where * stands for exactly one label (*.example.com matches acme.example.com, not example.com nor a.b.example.com). Case and one trailing dot are ignored; an entry without a port only matches the scheme's default port. Required when WEBAPP_URL_FROM_REQUEST is on: links are built from the Host header, so an unlisted host could otherwise put an attacker's address in password reset and magic link emails."
    )]
    pub webapp_allowed_hosts: Vec<String>,

    #[arg(
        long,
        env = "WEBHOOK_ALLOW_PRIVATE_ENDPOINTS",
        default_value_t = false,
        long_help = "Allow webhook endpoints to resolve to loopback or private addresses. For local development only — never enable this in production. Link-local addresses stay refused regardless."
    )]
    pub webhook_allow_private_endpoints: bool,

    #[arg(
        long,
        env = "CLIENT_METADATA_ALLOW_PRIVATE_ENDPOINTS",
        default_value_t = false,
        long_help = "Allow client metadata documents to be fetched over http from loopback or private addresses. For tests only — never enable this in production. Link-local addresses stay refused regardless."
    )]
    pub client_metadata_allow_private_endpoints: bool,
    #[arg(
        long = "config-cache-ttl-seconds",
        env = "CONFIG_CACHE_TTL_SECONDS",
        default_value_t = 5,
        long_help = "How long an instance keeps a realm, its settings and a client in memory before reading them again (seconds). Writes made through the API invalidate the entry immediately on the instance that handled them; other instances see them after this delay. It also bounds how long the client credentials grant keeps using a service account's state, roles and groups: a change to them, on any instance, reaches new tokens only after this delay. 0 disables the cache."
    )]
    pub config_cache_ttl_seconds: u64,
    #[arg(
        long = "password-hash-memory-kib",
        env = "PASSWORD_HASH_MEMORY_KIB",
        default_value_t = 7168,
        long_help = "Memory cost of a new Argon2id password hash, in KiB. Verification reads the cost from the stored hash, so this only changes new hashes; a login whose stored hash used another cost is rehashed with this one. Refused below the OWASP floor at startup."
    )]
    pub password_hash_memory_kib: u32,
    #[arg(
        long = "password-hash-iterations",
        env = "PASSWORD_HASH_ITERATIONS",
        default_value_t = 5,
        long_help = "Number of Argon2id iterations of a new password hash. Refused below the OWASP floor at startup."
    )]
    pub password_hash_iterations: u32,
    #[arg(
        long = "password-hash-parallelism",
        env = "PASSWORD_HASH_PARALLELISM",
        default_value_t = 1,
        long_help = "Parallelism degree of a new Argon2id password hash."
    )]
    pub password_hash_parallelism: u32,
    #[arg(
        long = "password-hash-memory-budget-mib",
        env = "PASSWORD_HASH_MEMORY_BUDGET_MIB",
        default_value_t = 256,
        long_help = "Memory the instance lets Argon2 use at once across every password hash and verification in flight, in MiB. A request beyond the budget waits for memory to free up instead of allocating; one hash dearer than the budget runs alone. Must hold at least one hash of PASSWORD_HASH_MEMORY_KIB."
    )]
    pub password_hash_memory_budget_mib: u32,
    #[arg(
        long = "secrets-encryption-keys",
        env = "SECRETS_ENCRYPTION_KEYS",
        hide_env_values = true,
        default_value = "",
        long_help = "Keys that seal the secrets kept in the database (realm signing keys, TOTP seeds, SMTP passwords), written id:base64 and separated by commas. Each key is 32 random bytes in base64. The first key seals, every key opens, so a key is rotated by putting the new one first. Secrets already stored in clear are sealed at the next start. Leave it empty to keep them in clear."
    )]
    pub secrets_encryption_keys: String,
    #[arg(
        long,
        env = "AUTHZ_ENABLED",
        default_value_t = false,
        long_help = "Expose the AuthZEN evaluation endpoint and its metadata document. A realm still has to enable tenant authorization for itself."
    )]
    pub authz_enabled: bool,
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
            webhook_allow_private_endpoints: false,
            client_metadata_allow_private_endpoints: false,
            config_cache_ttl_seconds: 5,
            password_hash_memory_kib: 7168,
            password_hash_iterations: 5,
            password_hash_parallelism: 1,
            password_hash_memory_budget_mib: 256,
            authz_enabled: false,
            secrets_encryption_keys: String::new(),
            webapp_url: "http://localhost:5555".to_string(),
            webapp_url_from_request: false,
            webapp_allowed_hosts: vec![],
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
    #[arg(
        long = "database-max-connections",
        env = "DATABASE_MAX_CONNECTIONS",
        default_value_t = 100,
        name = "DATABASE_MAX_CONNECTIONS",
        long_help = "Maximum connections one instance opens to the database. The total across instances must stay under the server's max_connections. Each connection costs the database a few MiB, so a smaller pool is lighter on the database."
    )]
    pub max_connections: u32,
    #[arg(
        long = "database-min-connections",
        env = "DATABASE_MIN_CONNECTIONS",
        default_value_t = 5,
        name = "DATABASE_MIN_CONNECTIONS",
        long_help = "Connections one instance keeps open when idle. Must not exceed the maximum."
    )]
    pub min_connections: u32,
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
            max_connections: 100,
            min_connections: 5,
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
            max_connections: 100,
            min_connections: 5,
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
        long_help = "Origins always allowed to call the API from a browser, comma-separated, on every route. Per-client web origins only cover realm-scoped routes, so a console served from a different origin than the API must keep its origin here: /config, the health probes and the API docs carry no realm and can be allowed by nothing else. Each entry must be a serialized origin (scheme://host[:port]), with no path and no wildcard.",
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
    #[arg(
        long = "server-public-url",
        env = "SERVER_PUBLIC_URL",
        name = "SERVER_PUBLIC_URL",
        long_help = "The origin browsers and service providers reach this deployment at, as scheme://host[:port]. Leave unset to derive it from each request's Host header, which is the historical behaviour. Setting it is required for SAML: an entity id is signed into every assertion, so it must not change when a request arrives through a different hostname."
    )]
    pub public_url: Option<String>,
    #[arg(
        long = "server-trust-forwarded-for",
        env = "SERVER_TRUST_FORWARDED_FOR",
        name = "SERVER_TRUST_FORWARDED_FOR",
        default_value_t = false,
        long_help = "Read the caller's address from the X-Forwarded-For header instead of the peer of the TCP connection. Leave it off unless a proxy you control is the only way in: when it is on, a client can put any address in that header and the recorded address is whatever it claims. With it off, a deployment behind a proxy records the proxy's address, which is honest but of little use."
    )]
    pub trust_forwarded_for: bool,
    #[arg(
        long = "server-token-max-in-flight",
        env = "SERVER_TOKEN_MAX_IN_FLIGHT",
        name = "SERVER_TOKEN_MAX_IN_FLIGHT",
        long_help = "The most client_credentials token requests served at once. Past it, requests get 503 with a Retry-After header instead of queuing behind the ones in flight. Must be at least 1. Defaults to 32 per CPU the process may use."
    )]
    pub token_max_in_flight: Option<NonZeroUsize>,
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
            public_url: None,
            trust_forwarded_for: false,
            token_max_in_flight: None,
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
                max_connections: value.db.max_connections,
                min_connections: value.db.min_connections,
            },
            webapp_url: value.webapp_url,
            webhook_allow_private_endpoints: value.webhook_allow_private_endpoints,
            client_metadata_allow_private_endpoints: value.client_metadata_allow_private_endpoints,
            config_cache_ttl_seconds: value.config_cache_ttl_seconds,
            password_hashing: PasswordHashingCost {
                memory_kib: value.password_hash_memory_kib,
                iterations: value.password_hash_iterations,
                parallelism: value.password_hash_parallelism,
                memory_budget_kib: value.password_hash_memory_budget_mib.saturating_mul(1024),
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
