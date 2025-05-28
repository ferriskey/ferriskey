use clap::{Parser, ValueEnum};
use typeshare::typeshare;

#[derive(Debug, Clone, ValueEnum, Default)]
#[typeshare]
pub enum AppEnv {
    #[default]
    Development,
    Production,
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

impl ToString for AppEnv {
    fn to_string(&self) -> String {
        match self {
            AppEnv::Development => "development".to_string(),
            AppEnv::Production => "production".to_string(),
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
    pub public_key: String,

    #[clap(env)]
    pub private_key: String,

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
}
