use std::fmt::Display;

use clap::{Parser, ValueEnum};
use typeshare::typeshare;

#[derive(Debug, Clone, ValueEnum, Default)]
#[typeshare]
pub enum AppEnv {
    #[default]
    Development,
    Production,
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum LogLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<String> for AppEnv {
    fn from(value: String) -> Self {
        match value.as_str() {
            "development" => AppEnv::Development,
            "production" => AppEnv::Production,
            _ => AppEnv::Development, // Default to Development if unknown
        }
    }
}

impl Display for AppEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppEnv::Development => write!(f, "development"),
            AppEnv::Production => write!(f, "production"),
        }
    }
}

impl From<String> for LogLevel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            _ => LogLevel::Info, // Default to Info if unknown
        }
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

#[derive(Debug, Clone, Default, Parser)]
pub struct Env {
    #[clap(env)]
    pub port: String,

    #[clap(env)]
    pub database_url: String,

    #[clap(env)]
    pub portal_url: String,

    #[clap(env)]
    pub allowed_origins: String,

    #[clap(env)]
    pub admin_password: String,
    #[clap(env)]
    pub admin_username: String,
    #[clap(env)]
    pub admin_email: String,

    #[clap(env, default_value = "development", value_enum)]
    pub env: AppEnv,

    #[clap(env, default_value = "info", value_enum)]
    pub log_level: LogLevel,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}
