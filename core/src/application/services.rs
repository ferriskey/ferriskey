use crate::{
    domain::{
        abyss::{BrokerServiceImpl, IdentityProviderServiceImpl},
        authentication::services::AuthServiceImpl,
        client::services::ClientServiceImpl,
        common::{
            entities::{InitializationResult, StartupConfig, app_errors::CoreError},
            ports::CoreService,
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

type RealmRepo = PostgresRealmRepository;
type ClientRepo = PostgresClientRepository;
type UserRepo = PostgresUserRepository;
type UserRoleRepo = PostgresUserRoleRepository;
type SecurityEventRepo = PostgresSecurityEventRepository;
type CredentialRepo = PostgresCredentialRepository;
type WebhookRepo = PostgresWebhookRepository;
type RedirectUriRepo = PostgresRedirectUriRepository;
type RoleRepo = PostgresRoleRepository;
type HealthCheckRepo = PostgresHealthCheckRepository;
type RecoveryCodeRepo = RandBytesRecoveryCodeRepository<10, Argon2HasherRepository>;
type AuthSessionRepo = PostgresAuthSessionRepository;
type HasherRepo = Argon2HasherRepository;
type UserRequiredActionRepo = PostgresUserRequiredActionRepository;
type KeystoreRepo = PostgresKeyStoreRepository;
type RefreshTokenRepo = PostgresRefreshTokenRepository;
type AccessTokenRepo = PostgresAccessTokenRepository;
type IdentityProviderRepo = PostgresIdentityProviderRepository;
type FederationRepo = FederationRepositoryImpl;
type BrokerAuthSessionRepo = PostgresBrokerAuthSessionRepository;
type IdentityProviderLinkRepo = PostgresIdentityProviderLinkRepository;
type OAuthClientImpl = ReqwestOAuthClient;

#[derive(Clone, Debug)]
pub struct ApplicationService {
    pub(crate) security_event_service:
        SecurityEventServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SecurityEventRepo>,
    pub(crate) credential_service:
        CredentialServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, CredentialRepo>,
    pub(crate) client_service: ClientServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        WebhookRepo,
        RedirectUriRepo,
        RoleRepo,
        SecurityEventRepo,
    >,
    pub(crate) realm_service: RealmServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        RoleRepo,
        WebhookRepo,
        IdentityProviderRepo,
    >,
    pub(crate) role_service: RoleServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        RoleRepo,
        SecurityEventRepo,
        WebhookRepo,
    >,
    pub(crate) trident_service: TridentServiceImpl<
        CredentialRepo,
        RecoveryCodeRepo,
        AuthSessionRepo,
        HasherRepo,
        UserRequiredActionRepo,
    >,
    pub(crate) user_service: UserServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        CredentialRepo,
        HasherRepo,
        RoleRepo,
        UserRequiredActionRepo,
        WebhookRepo,
        SecurityEventRepo,
    >,
    pub(crate) health_service: HealthServiceImpl<HealthCheckRepo>,
    pub(crate) webhook_service:
        WebhookServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, WebhookRepo>,

    pub(crate) auth_service: AuthServiceImpl<
        RealmRepo,
        ClientRepo,
        RedirectUriRepo,
        UserRepo,
        UserRoleRepo,
        RoleRepo,
        CredentialRepo,
        HasherRepo,
        AuthSessionRepo,
        KeystoreRepo,
        RefreshTokenRepo,
        AccessTokenRepo,
        FederationRepo,
    >,
    pub(crate) core_service: CoreServiceImpl<
        RealmRepo,
        KeystoreRepo,
        ClientRepo,
        UserRepo,
        RoleRepo,
        UserRoleRepo,
        HasherRepo,
        CredentialRepo,
        RedirectUriRepo,
    >,
    pub(crate) identity_provider_service: IdentityProviderServiceImpl<
        IdentityProviderRepo,
        crate::domain::common::policies::FerriskeyPolicy<UserRepo, ClientRepo, UserRoleRepo>,
        RealmRepo,
    >,
    pub(crate) federation_service:
        crate::domain::abyss::federation::services::FederationServiceImpl<
            RealmRepo,
            FederationRepo,
            crate::domain::common::policies::FerriskeyPolicy<UserRepo, ClientRepo, UserRoleRepo>,
            UserRepo,
            CredentialRepo,
        >,
    pub(crate) broker_service: BrokerServiceImpl<
        RealmRepo,
        IdentityProviderRepo,
        BrokerAuthSessionRepo,
        IdentityProviderLinkRepo,
        ClientRepo,
        RedirectUriRepo,
        UserRepo,
        AuthSessionRepo,
        OAuthClientImpl,
    >,
}

impl CoreService for ApplicationService {
    async fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> Result<InitializationResult, CoreError> {
        self.core_service.initialize_application(config).await
    }
}
