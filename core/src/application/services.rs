use ferriskey_compass::recorder::FlowRecorder;

use crate::{
    domain::{
        abyss::{BrokerServiceImpl, IdentityProviderServiceImpl},
        aegis::services::{
            ClientScopeServiceImpl, ProtocolMapperServiceImpl, ScopeMappingServiceImpl,
        },
        authentication::services::AuthServiceImpl,
        client::services::ClientServiceImpl,
        common::{
            entities::{InitializationResult, StartupConfig, app_errors::CoreError},
            ports::CoreService,
            services::CoreServiceImpl,
        },
        compass::services::CompassServiceImpl,
        credential::services::CredentialServiceImpl,
        health::services::HealthServiceImpl,
        password_policy::services::PasswordPolicyServiceImpl,
        realm::services::{MailServiceImpl, RealmServiceImpl},
        role::services::RoleServiceImpl,
        seawatch::services::SecurityEventServiceImpl,
        trident::services::TridentServiceImpl,
        user::services::UserServiceImpl,
        webhook::services::WebhookServiceImpl,
    },
    infrastructure::{
        abyss::federation::repository::FederationRepositoryImpl,
        aegis::repositories::{
            client_scope_postgres_repository::PostgresClientScopeRepository,
            protocol_mapper_postgres_repository::PostgresProtocolMapperRepository,
            scope_mapping_postgres_repository::PostgresScopeMappingRepository,
        },
        client::repositories::{
            client_postgres_repository::PostgresClientRepository,
            post_logout_redirect_uri_postgres_repository::PostgresPostLogoutRedirectUriRepository,
            redirect_uri_postgres_repository::PostgresRedirectUriRepository,
        },
        compass::repositories::{PostgresCompassFlowRepository, PostgresCompassFlowStepRepository},
        email::SmtpEmailPort,
        health::repositories::PostgresHealthCheckRepository,
        identity_provider::{
            PostgresBrokerAuthSessionRepository, PostgresIdentityProviderLinkRepository,
            PostgresIdentityProviderRepository, ReqwestOAuthClient,
        },
        realm::repositories::{
            realm_postgres_repository::PostgresRealmRepository,
            smtp_config_postgres_repository::PostgresSmtpConfigRepository,
        },
        repositories::{
            access_token_repository::PostgresAccessTokenRepository,
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            magic_link_repository::PostgresMagicLinkRepository,
            password_policy_repository::PostgresPasswordPolicyRepository,
            password_reset_token_repository::PostgresPasswordResetTokenRepository,
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
type PostLogoutRedirectUriRepo = PostgresPostLogoutRedirectUriRepository;
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
type ClientScopeRepo = PostgresClientScopeRepository;
type ProtocolMapperRepo = PostgresProtocolMapperRepository;
type ScopeMappingRepo = PostgresScopeMappingRepository;
type MagicLinkRepo = PostgresMagicLinkRepository;
type CompassFlowRepo = PostgresCompassFlowRepository;
type CompassFlowStepRepo = PostgresCompassFlowStepRepository;
type SmtpConfigRepo = PostgresSmtpConfigRepository;
type EmailPortImpl = SmtpEmailPort;
type PasswordResetTokenRepo = PostgresPasswordResetTokenRepository;
type PasswordPolicyRepo = PostgresPasswordPolicyRepository;

type ApplicationTridentService = TridentServiceImpl<
    CredentialRepo,
    RecoveryCodeRepo,
    AuthSessionRepo,
    HasherRepo,
    UserRequiredActionRepo,
    MagicLinkRepo,
    UserRepo,
    RealmRepo,
    EmailPortImpl,
    SmtpConfigRepo,
    PasswordResetTokenRepo,
    SecurityEventRepo,
    WebhookRepo,
>;

type ApplicationAuthService = AuthServiceImpl<
    RealmRepo,
    ClientRepo,
    RedirectUriRepo,
    PostLogoutRedirectUriRepo,
    UserRepo,
    UserRoleRepo,
    CredentialRepo,
    HasherRepo,
    AuthSessionRepo,
    KeystoreRepo,
    RefreshTokenRepo,
    AccessTokenRepo,
    FederationRepo,
    ScopeMappingRepo,
    ProtocolMapperRepo,
>;

#[derive(Clone, Debug)]
pub struct ApplicationService {
    pub security_event_service:
        SecurityEventServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SecurityEventRepo>,
    pub credential_service:
        CredentialServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, CredentialRepo>,
    pub client_service: ClientServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        WebhookRepo,
        RedirectUriRepo,
        PostLogoutRedirectUriRepo,
        RoleRepo,
        SecurityEventRepo,
        ClientScopeRepo,
        ScopeMappingRepo,
    >,
    pub realm_service: RealmServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        RoleRepo,
        WebhookRepo,
        IdentityProviderRepo,
        ClientScopeRepo,
        ProtocolMapperRepo,
        ScopeMappingRepo,
        PasswordPolicyRepo,
    >,
    pub mail_service:
        MailServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SmtpConfigRepo>,
    pub role_service: RoleServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        RoleRepo,
        SecurityEventRepo,
        WebhookRepo,
    >,
    pub trident_service: ApplicationTridentService,
    pub user_service: UserServiceImpl<
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
        PasswordPolicyServiceImpl<PasswordPolicyRepo>,
    >,
    pub health_service: HealthServiceImpl<HealthCheckRepo>,
    pub webhook_service:
        WebhookServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, WebhookRepo>,
    pub password_policy_service: PasswordPolicyServiceImpl<PasswordPolicyRepo>,

    pub auth_service: ApplicationAuthService,
    pub core_service: CoreServiceImpl<
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
    pub identity_provider_service: IdentityProviderServiceImpl<
        IdentityProviderRepo,
        crate::domain::common::policies::FerriskeyPolicy<UserRepo, ClientRepo, UserRoleRepo>,
        RealmRepo,
    >,
    pub federation_service:
        crate::domain::abyss::federation::services::FederationServiceImpl<
            RealmRepo,
            FederationRepo,
            crate::domain::common::policies::FerriskeyPolicy<UserRepo, ClientRepo, UserRoleRepo>,
            UserRepo,
            CredentialRepo,
        >,
    pub broker_service: BrokerServiceImpl<
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
    pub client_scope_service: ClientScopeServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ProtocolMapperRepo,
    >,
    pub protocol_mapper_service: ProtocolMapperServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ProtocolMapperRepo,
    >,
    pub scope_mapping_service: ScopeMappingServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ScopeMappingRepo,
    >,
    pub compass_service: CompassServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        CompassFlowRepo,
        CompassFlowStepRepo,
    >,
    #[allow(dead_code)]
    pub(crate) flow_recorder: FlowRecorder,
}

impl CoreService for ApplicationService {
    async fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> Result<InitializationResult, CoreError> {
        self.core_service.initialize_application(config).await
    }
}
