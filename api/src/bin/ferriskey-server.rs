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

use anyhow::Context;
use clap::Parser;
use ferriskey_core::application::orchestrators::startup_orchestrator::{
    StartupConfig, StartupOrchestrator, StartupOrchestratorBuilder,
};

use ferriskey_api::application::http::server::http_server::{router, state};
use ferriskey_api::env::{AppEnv, Env};

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
    init_logger(env.clone());

    let app_state = state(env.clone()).await?;

    let startup_orchestrator = StartupOrchestrator::new(StartupOrchestratorBuilder {
        realm_service: app_state.service_bundle.realm_service.clone(),
        user_service: app_state.service_bundle.user_service.clone(),
        client_service: app_state.service_bundle.client_service.clone(),
        role_service: app_state.service_bundle.role_service.clone(),
        jwt_service: app_state.service_bundle.jwt_service.clone(),
        user_role_service: app_state.service_bundle.user_role_service.clone(),
        credential_service: app_state.service_bundle.credential_service.clone(),
        redirect_uri_service: app_state.service_bundle.redirect_uri_service.clone(),
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

    let router = router(app_state)?;

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", env.port))
        .await
        .with_context(|| format!("Failed to bind to port {}", env.port))?;

    tracing::info!(
        "FerrisKey API is running on {}:{}",
        listener.local_addr()?,
        env.port,
    );

    axum::serve(listener, router)
        .await
        .context("received error while running server")?;
    Ok(())
}
