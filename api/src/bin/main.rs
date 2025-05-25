use std::sync::Arc;

use clap::Parser;
use ferriskey::application::http::server::http_server::{HttpServer, HttpServerConfig};

use ferriskey::application::server::AppServer;
use ferriskey::domain::authentication::service::auth_session::DefaultAuthSessionService;
use ferriskey::domain::authentication::service::authentication::DefaultAuthenticationService;

use ferriskey::domain::client::services::client_service::DefaultClientService;
use ferriskey::domain::client::services::redirect_uri_service::DefaultRedirectUriService;
use ferriskey::domain::credential::services::credential_service::DefaultCredentialService;
use ferriskey::domain::crypto::services::crypto_service::DefaultCryptoService;

use ferriskey::domain::jwt::services::jwt_service::DefaultJwtService;
use ferriskey::domain::mediator::ports::mediator_service::MediatorService;
use ferriskey::domain::mediator::services::mediator_service::DefaultMediatorService;
use ferriskey::domain::realm::services::realm_service::DefaultRealmService;
use ferriskey::domain::role::services::DefaultRoleService;
use ferriskey::domain::user::services::user_role_service::DefaultUserRoleService;
use ferriskey::domain::user::services::user_service::DefaultUserService;
use ferriskey::env::{AppEnv, Env};

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

    let realm_service = Arc::new(DefaultRealmService::new(
        app_server.realm_repository.clone(),
    ));

    let client_service = Arc::new(DefaultClientService::new(
        app_server.client_repository,
        app_server.user_repository.clone(),
        Arc::clone(&realm_service),
    ));

    let redirect_uri_service = DefaultRedirectUriService::new(
        app_server.redirect_uri_repository,
        Arc::clone(&realm_service),
        Arc::clone(&client_service),
    );

    let user_service = Arc::new(DefaultUserService::new(
        app_server.user_repository.clone(),
        app_server.realm_repository.clone(),
    ));

    let crypto_service = Arc::new(DefaultCryptoService::new(app_server.hasher_repository));

    let credential_service = Arc::new(DefaultCredentialService::new(
        app_server.credential_repository,
        Arc::clone(&crypto_service),
    ));

    let jwt_service = Arc::new(DefaultJwtService::new(
        app_server.jwt_repository,
        app_server.refresh_token_repository,
    ));

    let auth_session_service = Arc::new(DefaultAuthSessionService::new(
        app_server.auth_session_repository,
    ));

    let role_service = DefaultRoleService::new(app_server.role_repository.clone());

    let authentication_service = Arc::new(DefaultAuthenticationService::new(
        Arc::clone(&realm_service),
        Arc::clone(&client_service),
        Arc::clone(&credential_service),
        Arc::clone(&user_service),
        Arc::clone(&jwt_service),
        Arc::clone(&auth_session_service),
    ));

    let user_role_service = DefaultUserRoleService::new(
        app_server.user_repository.clone(),
        app_server.role_repository.clone(),
        app_server.realm_repository.clone(),
    );

    let mediator_service = Arc::new(DefaultMediatorService::new(
        Arc::clone(&env),
        Arc::clone(&client_service),
        Arc::clone(&realm_service),
        Arc::clone(&user_service),
        Arc::clone(&credential_service),
        redirect_uri_service.clone(),
        role_service.clone(),
        user_role_service.clone(),
    ));

    mediator_service
        .initialize_master_realm()
        .await
        .expect("Failed to initialize master realm");

    mediator_service
        .initialize_admin_redirect_uris()
        .await
        .expect("Failed to initialize admin redirect uris");

    let server_config = HttpServerConfig::new(env.port.clone());

    let http_server = HttpServer::new(
        env.clone(),
        server_config,
        realm_service,
        client_service,
        credential_service,
        authentication_service,
        auth_session_service,
        user_service,
        jwt_service,
        redirect_uri_service,
        role_service,
        user_role_service,
    )
    .await?;

    http_server.run().await?;

    Ok(())
}
