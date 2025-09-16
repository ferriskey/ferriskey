mod crd;
mod macros;

use ferriskey_operator::application::OperatorApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tracing::info!("start ferriskey operator");

    OperatorApp::run().await?;

    tracing::info!("controller running...");

    Ok(())
}
