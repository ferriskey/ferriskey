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

use clap::Parser;
use ferriskey_core::{
    application::common::FerriskeyService,
    domain::common::{DatabaseConfig, FerriskeyConfig},
};

use crate::cli::{BootstrapCommand, Cli, Commands};

pub mod cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();

    let cfg = FerriskeyConfig {
        database: DatabaseConfig {
            host: cli.database.host,
            name: cli.database.name,
            password: cli.database.password,
            username: cli.database.user,
            port: cli.database.port,
        },
    };
    println!("cfg: {:?}", cfg);
    let service = FerriskeyService::new(cfg).await?;

    match cli.command {
        Commands::Bootstrap(cmd) => {
            println!("init bootstrap");
        }
    }

    Ok(())
}
