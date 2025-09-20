mod crd;
mod macros;

use ferriskey_operator::application::OperatorApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("🚀 Démarrage de l'opérateur ferriskey");

    match OperatorApp::run().await {
        Ok(_) => tracing::info!("✅ Opérateur démarré avec succès"),
        Err(e) => {
            tracing::error!("❌ Erreur lors du démarrage de l'opérateur: {:?}", e);
            return Err(e.into());
        }
    }

    tracing::info!("🔄 Contrôleur en cours d'exécution...");

    Ok(())
}
