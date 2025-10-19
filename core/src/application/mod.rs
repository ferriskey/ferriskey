use crate::{
    domain::common::{FerriskeyConfig, entities::app_errors::CoreError, services::Service},
    infrastructure::{
        client::repositories::{
            client_postgres_repository::PostgresClientRepository,
            redirect_uri_postgres_repository::PostgresRedirectUriRepository,
        },
        db::postgres::{Postgres, PostgresConfig},
        health::repositories::PostgresHealthCheckRepository,
        realm::repositories::realm_postgres_repository::PostgresRealmRepository,
        repositories::{
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            random_bytes_recovery_code::RandBytesRecoveryCodeRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
        },
        role::repositories::role_postgres_repository::PostgresRoleRepository,
        user::{
            repositories::{
                user_required_action_repository::PostgresUserRequiredActionRepository,
                user_role_repository::PostgresUserRoleRepository,
            },
            repository::PostgresUserRepository,
        },
        webhook::repositories::{
            webhook_notifier_repository::PostgresWebhookNotifierRepository,
            webhook_repository::PostgresWebhookRepository,
        },
    },
};

pub type FerrisKeyService = Service<
    PostgresRealmRepository,
    PostgresClientRepository,
    PostgresUserRepository,
    PostgresCredentialRepository,
    Argon2HasherRepository,
    PostgresAuthSessionRepository,
    PostgresRedirectUriRepository,
    PostgresRoleRepository,
    PostgresKeyStoreRepository,
    PostgresUserRoleRepository,
    PostgresUserRequiredActionRepository,
    PostgresHealthCheckRepository,
    PostgresWebhookRepository,
    PostgresWebhookNotifierRepository,
    PostgresRefreshTokenRepository,
    RandBytesRecoveryCodeRepository<10, Argon2HasherRepository>,
>;

pub async fn create_service(config: FerriskeyConfig) -> Result<FerrisKeyService, CoreError> {
    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        config.database.username,
        config.database.password,
        config.database.host,
        config.database.port,
        config.database.name
    );

    let postgres = Postgres::new(PostgresConfig { database_url })
        .await
        .map_err(|e| CoreError::ServiceUnavailable(e.to_string()))?;

    let realm = PostgresRealmRepository::new(postgres.get_db());
    let client = PostgresClientRepository::new(postgres.get_db());
    let user = PostgresUserRepository::new(postgres.get_db());
    let credential = PostgresCredentialRepository::new(postgres.get_db());
    let hasher = Argon2HasherRepository::new();
    let auth_session = PostgresAuthSessionRepository::new(postgres.get_db());
    let redirect_uri = PostgresRedirectUriRepository::new(postgres.get_db());
    let role = PostgresRoleRepository::new(postgres.get_db());
    let keystore = PostgresKeyStoreRepository::new(postgres.get_db());
    let user_role = PostgresUserRoleRepository::new(postgres.get_db());
    let user_required_action = PostgresUserRequiredActionRepository::new(postgres.get_db());
    let health_check = PostgresHealthCheckRepository::new(postgres.get_db());
    let webhook = PostgresWebhookRepository::new(postgres.get_db());
    let webhook_notifier = PostgresWebhookNotifierRepository::new();
    let refresh_token = PostgresRefreshTokenRepository::new(postgres.get_db());
    let recovery_code = RandBytesRecoveryCodeRepository::new(hasher.clone());

    Ok(Service::new(
        realm,
        client,
        user,
        credential,
        hasher,
        auth_session,
        redirect_uri,
        role,
        keystore,
        user_role,
        user_required_action,
        health_check,
        webhook,
        webhook_notifier,
        refresh_token,
        recovery_code,
    ))
}
