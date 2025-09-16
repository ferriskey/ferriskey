mod crd;
mod macros;

use ferriskey_operator::application::OperatorApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    OperatorApp::run().await?;

    Ok(())
}
