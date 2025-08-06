use crate::application::common::services::{DefaultGrantTypeService, DefaultHealthCheckService};
use crate::domain::authentication::strategies::authorization_code_strategy::AuthorizationCodeStrategy;
use crate::domain::authentication::strategies::client_credentials_strategy::ClientCredentialsStrategy;
use crate::domain::authentication::strategies::password_strategy::PasswordStrategy;
use crate::domain::authentication::strategies::refresh_token_strategy::RefreshTokenStrategy;
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;
use crate::{
    application::common::services::{
        DefaultAuthSessionService, DefaultClientService, DefaultCredentialService,
        DefaultCryptoService, DefaultJwtService, DefaultRealmService, DefaultRedirectUriService,
        DefaultRoleService, DefaultUserRoleService, DefaultUserService,
    },
    domain::trident::services::OauthTotpService,
    infrastructure::{
        db::postgres::{Postgres, PostgresConfig},
        repositories::{
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            client_repository::PostgresClientRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            realm_repository::PostgresRealmRepository,
            redirect_uri_repository::PostgresRedirectUriRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
            role_repository::PostgresRoleRepository,
        },
        user::{
            repositories::{
                user_required_action_repository::PostgresUserRequiredActionRepository,
                user_role_repository::PostgresUserRoleRepository,
            },
            repository::PostgresUserRepository,
        },
    },
};

pub struct ServiceFactory;

pub struct ServiceFactoryConfig {
    pub database_url: String,
}

impl ServiceFactory {
    pub async fn create_all_services(
        config: ServiceFactoryConfig,
    ) -> Result<ServiceBundle, anyhow::Error> {
        let postgres = Postgres::new(PostgresConfig {
            database_url: config.database_url,
        })
        .await?;

        let realm_repository = PostgresRealmRepository::new(postgres.get_db());
        let client_repository = PostgresClientRepository::new(postgres.get_db());
        let user_repository = PostgresUserRepository::new(postgres.get_db());
        let credential_repository = PostgresCredentialRepository::new(postgres.get_db());
        let hasher_repository = Argon2HasherRepository::new();
        let auth_session_repository = PostgresAuthSessionRepository::new(postgres.get_db());
        let refresh_token_repository = PostgresRefreshTokenRepository::new(postgres.get_db());
        let redirect_uri_repository = PostgresRedirectUriRepository::new(postgres.get_db());
        let role_repository = PostgresRoleRepository::new(postgres.get_db());
        let keystore_repository = PostgresKeyStoreRepository::new(postgres.get_db());
        let user_role_repository = PostgresUserRoleRepository::new(postgres.get_db());
        let user_required_action_repository =
            PostgresUserRequiredActionRepository::new(postgres.get_db());
        let health_check_repository = PostgresHealthCheckRepository::new(postgres.get_db());

        let realm_service = DefaultRealmService::new(
            realm_repository.clone(),
            client_repository.clone(),
            role_repository.clone(),
            user_repository.clone(),
            user_role_repository.clone(),
        );

        let client_service = DefaultClientService::new(
            client_repository.clone(),
            user_repository.clone(),
            realm_repository.clone(),
        );

        let redirect_uri_service = DefaultRedirectUriService::new(
            realm_repository.clone(),
            redirect_uri_repository.clone(),
            client_repository.clone(),
        );

        let crypto_service = DefaultCryptoService::new(hasher_repository.clone());

        let credential_service =
            DefaultCredentialService::new(credential_repository.clone(), crypto_service.clone());

        let auth_session_service = DefaultAuthSessionService::new(auth_session_repository.clone());

        let user_service = DefaultUserService::new(
            user_repository.clone(),
            realm_repository.clone(),
            user_role_repository.clone(),
            user_required_action_repository.clone(),
        );

        let jwt_service = DefaultJwtService::new(
            refresh_token_repository.clone(),
            keystore_repository.clone(),
            realm_repository.clone(),
        );

        let role_service = DefaultRoleService::new(role_repository.clone());

        let user_role_service = DefaultUserRoleService::new(
            user_repository.clone(),
            role_repository.clone(),
            realm_repository.clone(),
            user_role_repository.clone(),
        );

        let totp_service = OauthTotpService::new();

        let grant_type_service = DefaultGrantTypeService::new(
            AuthorizationCodeStrategy::new(
                jwt_service.clone(),
                client_service.clone(),
                user_service.clone(),
                credential_service.clone(),
                auth_session_service.clone(),
            ),
            ClientCredentialsStrategy::new(
                client_service.clone(),
                user_service.clone(),
                jwt_service.clone(),
            ),
            PasswordStrategy::new(
                jwt_service.clone(),
                user_service.clone(),
                credential_service.clone(),
                client_service.clone(),
            ),
            RefreshTokenStrategy::new(
                jwt_service.clone(),
                client_service.clone(),
                user_service.clone(),
            ),
        );

        let health_check_service = DefaultHealthCheckService::new(health_check_repository.clone());

        Ok(ServiceBundle {
            realm_service,
            client_service,
            credential_service,
            auth_session_service,
            user_service,
            jwt_service,
            redirect_uri_service,
            role_service,
            user_role_service,
            totp_service,
            grant_type_service,
            health_check_service,
        })
    }
}

#[derive(Clone)]
pub struct ServiceBundle {
    pub realm_service: DefaultRealmService,
    pub client_service: DefaultClientService,
    pub credential_service: DefaultCredentialService,
    pub auth_session_service: DefaultAuthSessionService,
    pub user_service: DefaultUserService,
    pub jwt_service: DefaultJwtService,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub role_service: DefaultRoleService,
    pub user_role_service: DefaultUserRoleService,
    pub totp_service: OauthTotpService,
    pub grant_type_service: DefaultGrantTypeService,
    pub health_check_service: DefaultHealthCheckService,
}
