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

use std::sync::Arc;

use clap::Parser;
use ferriskey::application::http::server::http_server::{HttpServer, HttpServerConfig};

use ferriskey::application::server::AppServer;

use ferriskey::domain::mediator::ports::mediator_service::MediatorService;

use ferriskey::env::{AppEnv, Env};

fn init_logger(env: Arc<Env>) {
    match env.env {
        AppEnv::Development => {
            tracing_subscriber::fmt::init();
        }
        AppEnv::Production => {
            tracing_subscriber::fmt()
                .json()
                .with_max_level(tracing::Level::INFO)
                .with_writer(std::io::stdout)
                .init();
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let env = Arc::new(Env::parse());
    init_logger(Arc::clone(&env));

    let app_server = AppServer::new(Arc::clone(&env)).await?;
    let app_state = app_server.create_app_state(env.clone());

    app_state
        .mediator_service
        .initialize_master_realm()
        .await
        .expect("Failed to initialize master realm");

    app_state
        .mediator_service
        .initialize_admin_redirect_uris()
        .await
        .expect("Failed to initialize admin redirect uris");

    let server_config = HttpServerConfig::new(env.port.clone());

    let http_server = HttpServer::new(env.clone(), server_config, app_state).await?;

    http_server.run().await?;

    Ok(())
}
