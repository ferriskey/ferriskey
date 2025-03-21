use std::sync::Arc;

use clap::Parser;
use ferriskey::{
    application::http::{HttpServer, HttpServerConfig},
    domain::{
        client::service::ClientServiceImpl,
        realm::{ports::RealmService, service::RealmServiceImpl},
    },
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
    let realm_service = Arc::new(RealmServiceImpl::new(realm_repository));

    let client_repository = PostgresClientRepository::new(Arc::clone(&postgres));
    let client_service = Arc::new(ClientServiceImpl::new(
        client_repository,
        Arc::clone(&realm_service),
    ));

    realm_service.create_realm_master().await?;

    let server_config = HttpServerConfig::new(env.port.clone());

    let http_server = HttpServer::new(server_config, realm_service, client_service).await?;
    http_server.run().await?;
    println!("Hello AuthCrux");

    Ok(())
}
