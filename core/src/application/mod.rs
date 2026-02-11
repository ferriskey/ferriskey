use std::sync::Arc;

use crate::{
    domain::{
        abyss::federation::services::FederationServiceImpl,
        abyss::{BrokerServiceImpl, IdentityProviderServiceImpl},
        authentication::services::AuthServiceImpl,
        client::services::ClientServiceImpl,
        common::{
            FerriskeyConfig, entities::app_errors::CoreError, policies::FerriskeyPolicy,
            services::CoreServiceImpl,
        },
        credential::services::CredentialServiceImpl,
        health::services::HealthServiceImpl,
        realm::services::RealmServiceImpl,
        role::services::RoleServiceImpl,
        seawatch::services::SecurityEventServiceImpl,
        trident::services::TridentServiceImpl,
        user::services::UserServiceImpl,
        webhook::services::WebhookServiceImpl,
    },
    infrastructure::{
        abyss::federation::repository::FederationRepositoryImpl,
        client::repositories::{
            client_postgres_repository::PostgresClientRepository,
            redirect_uri_postgres_repository::PostgresRedirectUriRepository,
        },
        db::postgres::{Postgres, PostgresConfig},
        health::repositories::PostgresHealthCheckRepository,
        identity_provider::{
            PostgresBrokerAuthSessionRepository, PostgresIdentityProviderLinkRepository,
            PostgresIdentityProviderRepository, ReqwestOAuthClient,
        },
        realm::repositories::realm_postgres_repository::PostgresRealmRepository,
        repositories::{
            access_token_repository::PostgresAccessTokenRepository,
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            random_bytes_recovery_code::RandBytesRecoveryCodeRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
        },
        role::repositories::role_postgres_repository::PostgresRoleRepository,
        seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository,
        user::{
            repositories::{
                user_required_action_repository::PostgresUserRequiredActionRepository,
                user_role_repository::PostgresUserRoleRepository,
            },
            repository::PostgresUserRepository,
        },
        webhook::repositories::webhook_repository::PostgresWebhookRepository,
    },
};

pub mod services;

pub mod abyss;
pub mod auth;
pub mod broker;
pub mod client;
pub mod credential;
pub mod health;
pub mod identity_provider;
pub mod realm;
pub mod role;
pub mod seawatch;
pub mod trident;
pub mod user;
pub mod webhook;

pub use services::ApplicationService;

pub async fn create_service(config: FerriskeyConfig) -> Result<ApplicationService, CoreError> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}?options=-c search_path={}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name,
        urlencoding::encode(&config.database.schema)
    );

    let postgres = Postgres::new(PostgresConfig { database_url })
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let realm = Arc::new(PostgresRealmRepository::new(postgres.get_db()));
    let client = Arc::new(PostgresClientRepository::new(postgres.get_db()));
    let user = Arc::new(PostgresUserRepository::new(postgres.get_db()));
    let credential = Arc::new(PostgresCredentialRepository::new(postgres.get_db()));
    let hasher = Arc::new(Argon2HasherRepository::new());
    let auth_session = Arc::new(PostgresAuthSessionRepository::new(postgres.get_db()));
    let redirect_uri = Arc::new(PostgresRedirectUriRepository::new(postgres.get_db()));
    let role = Arc::new(PostgresRoleRepository::new(postgres.get_db()));
    let keystore = Arc::new(PostgresKeyStoreRepository::new(postgres.get_db()));
    let user_role = Arc::new(PostgresUserRoleRepository::new(postgres.get_db()));
    let user_required_action =
        Arc::new(PostgresUserRequiredActionRepository::new(postgres.get_db()));
    let health_check = Arc::new(PostgresHealthCheckRepository::new(postgres.get_db()));
    let webhook = Arc::new(PostgresWebhookRepository::new(postgres.get_db()));
    let refresh_token = Arc::new(PostgresRefreshTokenRepository::new(postgres.get_db()));
    let access_token = Arc::new(PostgresAccessTokenRepository::new(postgres.get_db()));
    let recovery_code = Arc::new(RandBytesRecoveryCodeRepository::new(hasher.clone()));
    let security_event = Arc::new(PostgresSecurityEventRepository::new(postgres.get_db()));
    let identity_provider = Arc::new(PostgresIdentityProviderRepository::new(postgres.get_db()));
    let federation = Arc::new(FederationRepositoryImpl::new(postgres.get_db()));
    let broker_auth_session = Arc::new(PostgresBrokerAuthSessionRepository::new(postgres.get_db()));
    let identity_provider_link = Arc::new(PostgresIdentityProviderLinkRepository::new(
        postgres.get_db(),
    ));
    let oauth_client = Arc::new(ReqwestOAuthClient::new());

    let policy = Arc::new(FerriskeyPolicy::new(
        user.clone(),
        client.clone(),
        user_role.clone(),
    ));

    let app = ApplicationService {
        auth_service: AuthServiceImpl::new(
            realm.clone(),
            client.clone(),
            redirect_uri.clone(),
            user.clone(),
            user_role.clone(),
            role.clone(),
            credential.clone(),
            hasher.clone(),
            auth_session.clone(),
            keystore.clone(),
            refresh_token.clone(),
            access_token.clone(),
            federation.clone(),
        ),
        client_service: ClientServiceImpl::new(
            realm.clone(),
            user.clone(),
            client.clone(),
            webhook.clone(),
            redirect_uri.clone(),
            role.clone(),
            security_event.clone(),
            policy.clone(),
        ),
        credential_service: CredentialServiceImpl::new(
            realm.clone(),
            credential.clone(),
            policy.clone(),
        ),
        health_service: HealthServiceImpl::new(health_check.clone()),
        realm_service: RealmServiceImpl::new(
            realm.clone(),
            user.clone(),
            user_role.clone(),
            role.clone(),
            client.clone(),
            webhook.clone(),
            identity_provider.clone(),
            policy.clone(),
        ),
        role_service: RoleServiceImpl::new(
            realm.clone(),
            role.clone(),
            security_event.clone(),
            webhook.clone(),
            user_role.clone(),
            policy.clone(),
        ),
        security_event_service: SecurityEventServiceImpl::new(
            realm.clone(),
            security_event.clone(),
            policy.clone(),
        ),
        trident_service: TridentServiceImpl::new(
            credential.clone(),
            recovery_code.clone(),
            auth_session.clone(),
            hasher.clone(),
            user_required_action.clone(),
        ),
        user_service: UserServiceImpl::new(
            realm.clone(),
            user.clone(),
            credential.clone(),
            hasher.clone(),
            user_role.clone(),
            role.clone(),
            user_required_action.clone(),
            webhook.clone(),
            security_event.clone(),
            policy.clone(),
        ),
        webhook_service: WebhookServiceImpl::new(realm.clone(), webhook.clone(), policy.clone()),
        core_service: CoreServiceImpl::new(
            realm.clone(),
            keystore.clone(),
            client.clone(),
            user.clone(),
            role.clone(),
            user_role.clone(),
            hasher.clone(),
            credential.clone(),
            redirect_uri.clone(),
        ),
        identity_provider_service: IdentityProviderServiceImpl::new(
            identity_provider.clone(),
            policy.clone(),
            realm.clone(),
        ),
        federation_service: FederationServiceImpl::new(
            realm.clone(),
            federation.clone(),
            user.clone(),
            credential.clone(),
            policy.clone(),
        ),
        broker_service: BrokerServiceImpl::new(
            realm.clone(),
            identity_provider.clone(),
            broker_auth_session.clone(),
            identity_provider_link.clone(),
            client.clone(),
            redirect_uri.clone(),
            user.clone(),
            auth_session.clone(),
            oauth_client.clone(),
        ),
    };

    Ok(app)
}
