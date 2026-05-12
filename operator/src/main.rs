use ferriskey_operator::application::OperatorApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // rustls 0.23 requires an explicit crypto provider when multiple backends
    // are compiled in (aws-lc-rs and ring are both present via kube-client).
    // Install aws-lc-rs as the default before any TLS connection is attempted.
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("failed to install rustls crypto provider");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Ferriskey operator start-up");

    match OperatorApp::run().await {
        Ok(_) => tracing::info!("Operator successfully started"),
        Err(e) => {
            tracing::error!("Error during operator start-up: {:?}", e);
            return Err(e.into());
        }
    }

    tracing::info!("🔄 Contrôleur en cours d'exécution...");

    Ok(())
}
