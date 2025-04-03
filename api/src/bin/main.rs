use std::sync::Arc;

use clap::Parser;
use ferriskey::application::http::server::http_server::{HttpServer, HttpServerConfig};

use ferriskey::application::server::AppServer;
use ferriskey::domain::authentication::service::AuthenticationServiceImpl;

use ferriskey::domain::credential::service::CredentialServiceImpl;

use ferriskey::domain::crypto::service::CryptoServiceImpl;

use ferriskey::domain::jwt::ports::JwtService;
use ferriskey::domain::jwt::service::JwtServiceImpl;
use ferriskey::domain::mediator::ports::MediatorService;
use ferriskey::domain::mediator::service::MediatorServiceImpl;
use ferriskey::domain::user::service::UserServiceImpl;
use ferriskey::{
    domain::{client::service::ClientServiceImpl, realm::service::RealmServiceImpl},
    env::{AppEnv, Env},
};

fn init_logger(env: Arc<Env>) {
    match env.env {
        AppEnv::Development => {
            tracing_subscriber::fmt::init();
        }
        AppEnv::Production => {
            tracing_subscriber::fmt()
                .json()
                .with_max_level(tracing::Level::INFO)
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

    let app_server = AppServer::new(Arc::clone(&env)).await?;

    let realm_service = Arc::new(RealmServiceImpl::new(app_server.realm_repository));

    let client_service = Arc::new(ClientServiceImpl::new(
        app_server.client_repository,
        Arc::clone(&realm_service),
    ));

    let user_service = Arc::new(UserServiceImpl::new(app_server.user_repository));

    let crypto_service = Arc::new(CryptoServiceImpl::new(app_server.hasher_repository));

    let credential_service = Arc::new(CredentialServiceImpl::new(
        app_server.credential_repository,
        Arc::clone(&crypto_service),
    ));

    let jwt_service: Arc<dyn JwtService> = Arc::new(JwtServiceImpl::new(app_server.jwt_repository));
    
    let authentication_service = Arc::new(AuthenticationServiceImpl::new(
        Arc::clone(&realm_service),
        Arc::clone(&client_service),
        Arc::clone(&credential_service),
        Arc::clone(&user_service),
        Arc::clone(&jwt_service),
    ));

    let mediator_service = Arc::new(MediatorServiceImpl::new(
        Arc::clone(&client_service),
        Arc::clone(&realm_service),
        Arc::clone(&user_service),
        Arc::clone(&credential_service),
    ));

    mediator_service
        .initialize_master_realm()
        .await
        .expect("Failed to initialize master realm");

    let server_config = HttpServerConfig::new(env.port.clone());

    let http_server = HttpServer::new(
        server_config,
        realm_service,
        client_service,
        credential_service,
        authentication_service,
    )
    .await?;

    http_server.run().await?;

    Ok(())
}
