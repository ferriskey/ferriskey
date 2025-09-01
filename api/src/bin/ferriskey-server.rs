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

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use ferriskey_core::application::orchestrators::startup_orchestrator::{
    StartupConfig, StartupOrchestrator, StartupOrchestratorBuilder,
};

use ferriskey_api::application::http::server::http_server::{router, state};
use ferriskey_api::args::{Args, LogArgs};
use tracing::{debug, info};
use tracing_subscriber::EnvFilter;

fn init_logger(args: &LogArgs) {
    let filter = EnvFilter::try_new(&args.filter).unwrap_or_else(|err| {
        eprint!("invalid log filter: {err}");
        eprint!("using default log filter: info");
        EnvFilter::new("info")
    });
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    if args.json {
        subscriber.json().init();
    } else {
        subscriber.init();
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let args = Arc::new(Args::parse());
    init_logger(&args.log);

    let app_state = state(args.clone()).await?;

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
            admin_email: args.admin.email.clone(),
            admin_password: args.admin.password.clone(),
            admin_username: args.admin.username.clone(),
            default_client_id: "security-admin-console".to_string(),
            master_realm_name: "master".to_string(),
        })
        .await?;

    let router = router(app_state)?;

    let addr = SocketAddr::from_str(&format!("{}:{}", args.server.host, args.server.port))?;
    if let Some(tls) = &args.server.tls {
        debug!("initializing crypto provider");
        rustls::crypto::aws_lc_rs::default_provider()
            .install_default()
            .expect("failed to install crypto provider");
        debug!("loading tls config");
        let tls_cfg = RustlsConfig::from_pem_file(tls.cert.clone(), tls.key.clone()).await?;
        info!("listening on {addr}");
        axum_server::bind_rustls(addr, tls_cfg)
            .serve(router.into_make_service())
            .await?;
    } else {
        info!("listening on {addr}");
        axum_server::bind(addr)
            .serve(router.into_make_service())
            .await?;
    }
    Ok(())
}
