use ferriskey_operator::{
    application::OperatorApp,
    domain::common::{Environment, OperatorConfig},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Ferriskey operator start-up");

    let config: OperatorConfig = OperatorConfig {
        env: Environment::Production,
    };

    match OperatorApp::run(&config).await {
        Ok(_) => tracing::info!("Operator successfully started"),
        Err(e) => {
            tracing::error!("Error during operator start-up: {:?}", e);
            return Err(e.into());
        }
    }

    tracing::info!("🔄 Contrôleur en cours d'exécution...");

    Ok(())
}
