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
use ferriskey::application::http::server::app_state::AppState;
use ferriskey::application::http::server::http_server::{HttpServer, HttpServerConfig};

use ferriskey::env::{AppEnv, Env};
use ferriskey_core::application::common::factories::UseCaseBundle;
use ferriskey_core::application::orchestrators::startup_orchestrator::{
    StartupConfig, StartupOrchestrator, StartupOrchestratorBuilder,
};
use ferriskey_core::infrastructure::common::factories::service_factory::{
    ServiceFactory, ServiceFactoryConfig,
};

fn init_logger(env: Arc<Env>) {
    let filter: tracing::Level = env.log_level.clone().into();
    match env.env {
        AppEnv::Development => {
            tracing_subscriber::fmt()
                .with_max_level(filter)
                .with_writer(std::io::stdout)
                .init();
        }
        AppEnv::Production => {
            tracing_subscriber::fmt()
                .json()
                .with_max_level(filter)
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

    let service_bundle = ServiceFactory::create_all_services(ServiceFactoryConfig {
        database_url: env.database_url.clone(),
    })
    .await?;

    let use_case_bundle = UseCaseBundle::new(service_bundle.clone());

    let app_state = AppState::new(env.clone(), service_bundle.clone(), use_case_bundle);

    let startup_orchestrator = StartupOrchestrator::new(StartupOrchestratorBuilder {
        realm_service: service_bundle.realm_service.clone(),
        user_service: service_bundle.user_service.clone(),
        client_service: service_bundle.client_service.clone(),
        role_service: service_bundle.role_service.clone(),
        jwt_service: service_bundle.jwt_service.clone(),
        user_role_service: service_bundle.user_role_service.clone(),
        credential_service: service_bundle.credential_service.clone(),
        redirect_uri_service: service_bundle.redirect_uri_service.clone(),
    });

    startup_orchestrator
        .initialize_application(StartupConfig {
            admin_email: env.admin_email.clone(),
            admin_password: env.admin_password.clone(),
            admin_username: env.admin_username.clone(),
            default_client_id: "security-admin-console".to_string(),
            master_realm_name: "master".to_string(),
        })
        .await?;

    tracing::info!("FerrisKey API is starting...");

    let server_config = HttpServerConfig::new(env.port.clone());

    let http_server = HttpServer::new(env.clone(), server_config, app_state).await?;

    http_server.run().await?;

    tracing::info!("FerrisKey API is running on port {}", env.port);
    Ok(())
}
