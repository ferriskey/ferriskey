// Copyright 2025 FerrisKey Contributors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ferriskey")]
#[command(about = "🔒 FerrisKey")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub database: DatabaseConfig,
}

#[derive(Args)]
pub struct DatabaseConfig {
    #[arg(
        long = "database-host",
        env = "DATABASE_HOST",
        name = "DATABASE_HOST",
        default_value = "localhost"
    )]
    pub host: String,

    #[arg(
        long = "database-port",
        env = "DATABASE_PORT",
        name = "DATABASE_PORT",
        default_value_t = 5432
    )]
    pub port: u16,

    #[arg(
        long = "database-user",
        env = "DATABASE_USER",
        name = "DATABASE_USER",
        default_value = "ferriskey"
    )]
    pub user: String,

    #[arg(
        long = "database-password",
        env = "DATABASE_PASSWORD",
        name = "DATABASE_PASSWORD",
        default_value = "ferriskey"
    )]
    pub password: String,

    #[arg(
        long = "database-name",
        env = "DATABASE_NAME",
        name = "DATABASE_NAME",
        default_value = "ferriskey"
    )]
    pub name: String,
}

#[derive(Subcommand)]
pub enum Commands {
    Bootstrap(BootstrapCommand),
}

#[derive(Parser)]
pub struct BootstrapCommand {
    #[arg(
        long = "admin-username",
        env = "ADMIN_USERNAME",
        default_value = "admin",
        help = "Admin username"
    )]
    pub admin_username: String,
    #[arg(
        long = "admin-password",
        env = "ADMIN_PASSWORD",
        default_value = "admin",
        help = "Admin password"
    )]
    pub admin_password: String,

    #[arg(
        long = "admin-email",
        env = "ADMIN_EMAIL",
        default_value = "admin@local",
        help = "Admin email address"
    )]
    pub admin_email: String,
}
