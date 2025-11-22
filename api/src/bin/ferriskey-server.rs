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

use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::Arc;

use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;

use ferriskey_api::application::http::server::http_server::{router, state};
use ferriskey_api::args::{Args, LogArgs};
use ferriskey_core::domain::common::entities::StartupConfig;
use ferriskey_core::domain::common::ports::CoreService;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{
    Resource,
    trace::RandomIdGenerator,
};
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Registry, fmt};

fn init_tracing_and_logging(args: &LogArgs, service_name: &str, otlp_endpoint: &str) {
    let filter = EnvFilter::try_new(&args.filter).unwrap_or_else(|err| {
        eprint!("invalid log filter: {err}");
        eprint!("using default log filter: info");
        EnvFilter::new("info")
    });
    // Create the OTLP exporter
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .expect("Failed to create span exporter");

    // Build the tracer provider with the exporter
    let provider = SdkTracerProvider::builder()
        .with_resource(
            Resource::builder()
                .with_service_name(service_name.to_string())
                .build(),
        )
        .with_id_generator(RandomIdGenerator::default())
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer(service_name.to_string());

    global::set_tracer_provider(provider);

    // Create tracing layers for logging and telemetry
    let fmt_layer = fmt::layer().with_writer(std::io::stderr).json(); // TODO Put a condition for json format
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Combine layers into a subscriber
    let subscriber = Registry::default()
        .with(fmt_layer)
        .with(telemetry_layer)
        .with(filter);

    subscriber.init();
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let args = Arc::new(Args::parse());
    init_tracing_and_logging(&args.log, "ferriskey_server", "http://localhost:4317"); // TODO extract endpoint from config

    let app_state = state(args.clone()).await?;

    app_state
        .service
        .initialize_application(StartupConfig {
            admin_email: args.admin.email.clone(),
            admin_password: args.admin.password.clone(),
            admin_username: args.admin.username.clone(),
            default_client_id: "security-admin-console".to_string(),
            master_realm_name: "master".to_string(),
        })
        .await?;

    let router = router(app_state)?;

    let addr = {
        let addrs = format!("{}:{}", args.server.host, args.server.port)
            .to_socket_addrs()?
            .collect::<Vec<SocketAddr>>();

        match addrs.first() {
            Some(addr) => *addr,
            None => {
                error!("At least one host and port must be provided.");
                return Err(anyhow::anyhow!(
                    "At least one host and port must be provided."
                ));
            }
        }
    };

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
