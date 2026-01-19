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

use crate::application::http::server::http_server::{router, state};
use crate::args::{Args, LogArgs, ObservabilityArgs};
use ferriskey_core::domain::common::entities::StartupConfig;
use ferriskey_core::domain::common::ports::CoreService;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_otlp::{Protocol, SpanExporter};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{Resource, trace::RandomIdGenerator};
use tracing::{debug, error, info};
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Layer, Registry, fmt};

pub mod application;
pub mod args;

fn init_tracing_and_logging(
    log_args: &LogArgs,
    service_name: &str,
    observability_args: &ObservabilityArgs,
) -> Result<(), anyhow::Error> {
    let filter = EnvFilter::try_new(&log_args.filter).unwrap_or_else(|err| {
        eprint!("invalid log filter: {err}");
        eprint!("using default log filter: info");
        EnvFilter::new("info")
    });

    // Format layer for logging
    let fmt_layer = if log_args.json {
        fmt::layer().with_writer(std::io::stderr).json().boxed()
    } else {
        fmt::layer().with_writer(std::io::stderr).boxed()
    };

    if observability_args.active_observability {
        // Check if endpoints are provided
        let otlp_endpoint = observability_args.otlp_endpoint.as_ref().ok_or_else(|| {
            anyhow::anyhow!("OTLP endpoint is required when observability is active")
        })?;

        let metrics_endpoint = observability_args
            .metrics_endpoint
            .as_ref()
            .ok_or_else(|| {
                anyhow::anyhow!("Metrics endpoint is required when observability is active")
            })?;

        // Create the OTLP exporter
        let span_exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otlp_endpoint)
            .build()?;

        // Build the tracer provider with the exporter
        let tracer_provider = SdkTracerProvider::builder()
            .with_resource(
                Resource::builder()
                    .with_service_name(service_name.to_string())
                    .build(),
            )
            .with_id_generator(RandomIdGenerator::default())
            .with_batch_exporter(span_exporter)
            .build();

        let tracer = tracer_provider.tracer(service_name.to_string());

        // Prometheus natively supports accepting metrics via the OTLP protocol
        // Create the metric exporter
        let metric_exporter = MetricExporter::builder()
            .with_tonic()
            .with_protocol(Protocol::Grpc)
            .with_endpoint(metrics_endpoint)
            .build()?;

        let meter_provider = SdkMeterProvider::builder()
            .with_periodic_exporter(metric_exporter)
            .build();

        // Metrics layer for tracing
        let metrics_layer = MetricsLayer::new(meter_provider);

        // Trace layer for tracing
        let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Combine layers into a subscriber
        let subscriber = Registry::default()
            .with(fmt_layer)
            .with(trace_layer)
            .with(metrics_layer)
            .with(filter);

        subscriber.init();
    } else {
        let subscriber = Registry::default().with(fmt_layer);

        subscriber.init();
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let args = Arc::new(Args::parse());
    init_tracing_and_logging(&args.log, "ferriskey_server", &args.observability)?;

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
