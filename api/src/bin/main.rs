use std::sync::Arc;

use clap::Parser;
use ferriskey::application::http::server::http_server::{HttpServer, HttpServerConfig};
use ferriskey::domain::authentication::service::AuthenticationServiceImpl;
use ferriskey::domain::credential::service::CredentialServiceImpl;
use ferriskey::domain::crypto::service::CryptoServiceImpl;
use ferriskey::domain::mediator::ports::MediatorService;
use ferriskey::domain::mediator::service::MediatorServiceImpl;
use ferriskey::domain::user::service::UserServiceImpl;
use ferriskey::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;
use ferriskey::infrastructure::repositories::credential_repository::PostgresCredentialRepository;
use ferriskey::infrastructure::repositories::user_repository::PostgresUserRepository;
use ferriskey::{
    domain::{client::service::ClientServiceImpl, realm::service::RealmServiceImpl},
    env::{AppEnv, Env},
    infrastructure::{
        db::postgres::Postgres,
        repositories::{
            client_repository::PostgresClientRepository, realm_repository::PostgresRealmRepository,
        },
    },
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

    let postgres = Arc::new(Postgres::new(Arc::clone(&env)).await?);

    let realm_repository = PostgresRealmRepository::new(Arc::clone(&postgres));
    let client_repository = PostgresClientRepository::new(Arc::clone(&postgres));
    let user_repository = PostgresUserRepository::new(Arc::clone(&postgres));
    let credential_repository = PostgresCredentialRepository::new(Arc::clone(&postgres));
    let hasher_repository = Argon2HasherRepository::new();

    let realm_service = Arc::new(RealmServiceImpl::new(realm_repository));

    let client_service = Arc::new(ClientServiceImpl::new(
        client_repository,
        Arc::clone(&realm_service),
    ));

    let user_service = Arc::new(UserServiceImpl::new(user_repository));

    let crypto_service = Arc::new(CryptoServiceImpl::new(hasher_repository));

    let credential_service = Arc::new(CredentialServiceImpl::new(
        credential_repository,
        Arc::clone(&crypto_service),
    ));

    let authentication_service = Arc::new(AuthenticationServiceImpl::new(
        Arc::clone(&realm_service),
        Arc::clone(&client_service),
        Arc::clone(&credential_service),
        Arc::clone(&user_service),
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
