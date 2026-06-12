use ferriskey_compass::recorder::FlowRecorder;
use ferriskey_migrate::{entities::MigrationReport, error::MigrationError};
use sea_orm::DatabaseConnection;

use crate::{
    application::migrate::{build_runner, context::MigrationContext},
    domain::{
        abyss::{BrokerServiceImpl, IdentityProviderServiceImpl},
        aegis::services::{
            ClientScopeServiceImpl, ProtocolMapperServiceImpl, ScopeMappingServiceImpl,
        },
        authentication::{
            device_flow::{
                error::DeviceFlowError,
                ports::{DeviceFlowService, DeviceTokenIssuer},
                services::DeviceFlowServiceImpl,
                value_objects::{
                    InitiateDeviceFlowInput, InitiateDeviceFlowOutput, InitiateDeviceFlowParams,
                    PollDeviceTokenParams,
                },
            },
            entities::{ExchangeTokenInput, JwtToken},
            ports::AuthService,
            services::AuthServiceImpl,
            value_objects::{GenerateTokensForUserInput, Identity},
        },
        client::{ports::ClientRepository, services::ClientServiceImpl},
        common::{
            entities::{InitializationResult, StartupConfig, app_errors::CoreError},
            ports::CoreService,
            services::CoreServiceImpl,
        },
        compass::services::CompassServiceImpl,
        credential::services::CredentialServiceImpl,
        email_template::services::EmailTemplateServiceImpl,
        email_verification::services::EmailVerificationServiceImpl,
        health::services::HealthServiceImpl,
        maintenance::services::MaintenanceServiceImpl,
        organization::services::OrganizationServiceImpl,
        password_policy::{
            entity::{PasswordPolicy, UpdatePasswordPolicy},
            service::PasswordPolicyService,
        },
        portal_layouts::services::PortalLayoutsServiceImpl,
        portal_theme::services::PortalThemeServiceImpl,
        realm::{
            ports::RealmRepository,
            services::{MailServiceImpl, RealmServiceImpl},
        },
        role::services::RoleServiceImpl,
        seawatch::services::SecurityEventServiceImpl,
        trident::services::TridentServiceImpl,
        user::services::UserServiceImpl,
        webhook::services::WebhookServiceImpl,
    },
    infrastructure::migrate::repository::PostgresMigrationRepository,
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
        email_template::{
            renderer::mjml_renderer::MjmlTemplateRenderer,
            repositories::email_template_repository::PostgresEmailTemplateRepository,
        },
        health::repositories::PostgresHealthCheckRepository,
        identity_provider::{
            PostgresBrokerAuthSessionRepository, PostgresIdentityProviderLinkRepository,
            PostgresIdentityProviderRepository, ReqwestOAuthClient,
        },
        organization::{
            organization_attribute_repository::PostgresOrganizationAttributeRepository,
            organization_member_repository::PostgresOrganizationMemberRepository,
            organization_repository::PostgresOrganizationRepository,
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
            device_auth_repository::PostgresDeviceAuthRepository,
            email_verification_token_repository::PostgresEmailVerificationTokenRepository,
            keystore_repository::PostgresKeyStoreRepository,
            magic_link_repository::PostgresMagicLinkRepository,
            password_reset_token_repository::PostgresPasswordResetTokenRepository,
            portal_layouts_repository::PostgresPortalLayoutsRepository,
            portal_theme_repository::PostgresPortalThemeRepository,
            random_bytes_recovery_code::RandBytesRecoveryCodeRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
        },
        role::repositories::role_postgres_repository::PostgresRoleRepository,
        seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository,
        user::{
            repositories::{
                user_attribute_repository::PostgresUserAttributeRepository,
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
type UserAttributeRepo = PostgresUserAttributeRepository;
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
type PasswordPolicyRepo = crate::infrastructure::repositories::password_policy_repository::PostgresPasswordPolicyRepository;
type EmailTemplateRepo = PostgresEmailTemplateRepository;
type MjmlRenderer = MjmlTemplateRenderer;
type PortalThemeRepo = PostgresPortalThemeRepository;
type PortalLayoutsRepo = PostgresPortalLayoutsRepository;
type OrganizationRepo = PostgresOrganizationRepository;
type OrganizationAttributeRepo = PostgresOrganizationAttributeRepository;
type OrganizationMemberRepo = PostgresOrganizationMemberRepository;
type EmailVerificationTokenRepo = PostgresEmailVerificationTokenRepository;

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
    EmailTemplateRepo,
    MjmlRenderer,
>;

type MaintenanceWhitelistRepo = crate::infrastructure::maintenance::repositories::maintenance_whitelist_repository::PostgresMaintenanceWhitelistRepository;
type RealmMaintenanceWhitelistRepo = crate::infrastructure::maintenance::repositories::realm_maintenance_whitelist_repository::PostgresRealmMaintenanceWhitelistRepository;
type ApplicationEmailVerificationService = EmailVerificationServiceImpl<
    EmailVerificationTokenRepo,
    UserRepo,
    RealmRepo,
    UserRequiredActionRepo,
    EmailPortImpl,
    SmtpConfigRepo,
    EmailTemplateRepo,
    MjmlRenderer,
    WebhookRepo,
    SecurityEventRepo,
>;

type ApplicationMaintenanceService = MaintenanceServiceImpl<
    RealmRepo,
    UserRepo,
    ClientRepo,
    UserRoleRepo,
    WebhookRepo,
    SecurityEventRepo,
    MaintenanceWhitelistRepo,
    RealmMaintenanceWhitelistRepo,
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
    OrganizationMemberRepo,
    OrganizationRepo,
    OrganizationAttributeRepo,
    UserRequiredActionRepo,
    MaintenanceWhitelistRepo,
    RealmMaintenanceWhitelistRepo,
    UserAttributeRepo,
    ApplicationEmailVerificationService,
    WebhookRepo,
    SecurityEventRepo,
>;

type DeviceAuthRepo = PostgresDeviceAuthRepository;

/// The auth service is the concrete token issuer for the device flow: an
/// approved session mints tokens through its existing issuance path.
impl DeviceTokenIssuer for ApplicationAuthService {
    async fn issue_tokens_for_user(
        &self,
        input: GenerateTokensForUserInput,
    ) -> Result<JwtToken, CoreError> {
        self.generate_tokens_for_user(input).await
    }
}

type ApplicationDeviceFlowService =
    DeviceFlowServiceImpl<DeviceAuthRepo, WebhookRepo, ApplicationAuthService>;

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
        PostLogoutRedirectUriRepo,
        RoleRepo,
        SecurityEventRepo,
        ClientScopeRepo,
        ScopeMappingRepo,
    >,
    pub(crate) realm_service: RealmServiceImpl<
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
        RedirectUriRepo,
    >,
    pub(crate) mail_service:
        MailServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SmtpConfigRepo>,
    pub(crate) role_service: RoleServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        RoleRepo,
        SecurityEventRepo,
        WebhookRepo,
    >,
    pub(crate) trident_service: ApplicationTridentService,
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
        UserAttributeRepo,
    >,
    pub(crate) health_service: HealthServiceImpl<HealthCheckRepo>,
    pub(crate) webhook_service:
        WebhookServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, WebhookRepo>,

    pub(crate) maintenance_service: ApplicationMaintenanceService,
    pub(crate) auth_service: ApplicationAuthService,
    pub(crate) device_flow_service: ApplicationDeviceFlowService,
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
    pub(crate) client_scope_service: ClientScopeServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ProtocolMapperRepo,
    >,
    pub(crate) protocol_mapper_service: ProtocolMapperServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ProtocolMapperRepo,
    >,
    pub(crate) scope_mapping_service: ScopeMappingServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        ClientScopeRepo,
        ScopeMappingRepo,
    >,
    pub(crate) compass_service: CompassServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        CompassFlowRepo,
        CompassFlowStepRepo,
    >,
    pub(crate) email_template_service: EmailTemplateServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        EmailTemplateRepo,
        MjmlRenderer,
    >,
    #[allow(dead_code)]
    pub(crate) portal_theme_service:
        PortalThemeServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, PortalThemeRepo>,
    #[allow(dead_code)]
    pub(crate) portal_layouts_service:
        PortalLayoutsServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, PortalLayoutsRepo>,
    pub(crate) password_policy_service:
        PasswordPolicyService<PasswordPolicyRepo, UserRepo, ClientRepo, UserRoleRepo>,
    pub(crate) organization_service: OrganizationServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        OrganizationRepo,
        OrganizationAttributeRepo,
        OrganizationMemberRepo,
    >,
    #[allow(dead_code)]
    pub(crate) flow_recorder: FlowRecorder,
    pub(crate) db: DatabaseConnection,
    pub email_verification_service: ApplicationEmailVerificationService,
}

impl CoreService for ApplicationService {
    async fn initialize_application(
        &self,
        config: StartupConfig,
    ) -> Result<InitializationResult, CoreError> {
        self.core_service.initialize_application(config).await
    }
}

impl ApplicationService {
    pub async fn get_password_policy(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<PasswordPolicy, CoreError> {
        // Get realm by name
        let realm = self
            .realm_service
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Authorization is handled inside the service
        self.password_policy_service
            .get_policy(identity, &realm)
            .await
    }

    pub async fn update_password_policy(
        &self,
        identity: Identity,
        realm_name: String,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        // Get realm by name
        let realm = self
            .realm_service
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Authorization is handled inside the service
        self.password_policy_service
            .update_policy(identity, &realm, update)
            .await
    }

    /// Device authorization endpoint (RFC 8628 §3.1).
    ///
    /// Resolves the realm and client, builds the realm-scoped verification URI
    /// from `base_url`, and delegates to the device flow service. `base_url`
    /// must already be root-path scoped by the caller.
    pub async fn initiate_device_authorization(
        &self,
        input: InitiateDeviceFlowInput,
        base_url: String,
    ) -> Result<InitiateDeviceFlowOutput, DeviceFlowError> {
        // An unresolvable realm/client at the device authorization endpoint
        // is `invalid_client`, matching the token endpoint's behaviour.
        let realm = self
            .realm_service
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| DeviceFlowError::InvalidClient)?
            .ok_or(DeviceFlowError::InvalidClient)?;

        let client = self
            .client_service
            .client_repository
            .get_by_client_id(input.client_id, realm.id)
            .await
            .map_err(|_| DeviceFlowError::InvalidClient)?;

        let verification_uri = format!("{base_url}/realms/{}/device", realm.name);

        self.device_flow_service
            .initiate(InitiateDeviceFlowParams {
                realm_id: realm.id,
                client_id: client.id,
                scope: input.scope,
                oauth_device_code_grant_enabled: client.oauth_device_code_grant_enabled,
                verification_uri,
            })
            .await
    }

    /// Token endpoint polling for the device_code grant (RFC 8628 §3.4).
    ///
    /// Resolves the realm and client, then advances the device flow state
    /// machine. Errors are returned as [`DeviceFlowError`] so the HTTP layer
    /// can render the RFC 6749 §5.2 error shape with the correct OAuth code.
    /// `base_url` must already be root-path scoped by the caller.
    pub async fn poll_device_token(
        &self,
        input: ExchangeTokenInput,
    ) -> Result<JwtToken, DeviceFlowError> {
        let device_code = input
            .device_code
            .as_deref()
            .and_then(|code| uuid::Uuid::parse_str(code).ok())
            .ok_or(DeviceFlowError::InvalidDeviceCode)?;

        // An unresolvable realm/client at the token endpoint is `invalid_client`.
        let realm = self
            .realm_service
            .realm_repository
            .get_by_name(&input.realm_name)
            .await
            .map_err(|_| DeviceFlowError::InvalidClient)?
            .ok_or(DeviceFlowError::InvalidClient)?;

        let client = self
            .client_service
            .client_repository
            .get_by_client_id(input.client_id, realm.id)
            .await
            .map_err(|_| DeviceFlowError::InvalidClient)?;

        self.device_flow_service
            .poll(PollDeviceTokenParams {
                device_code,
                client_id: client.id,
                base_url: input.base_url,
            })
            .await
    }

    /// Verification page: bind the authenticated user to the device session
    /// identified by `user_code` and mark it approved (RFC 8628 §3.3).
    pub async fn verify_device_user_code(
        &self,
        user_code: String,
        user_id: uuid::Uuid,
    ) -> Result<(), DeviceFlowError> {
        self.device_flow_service
            .verify_user_code(user_code, user_id)
            .await
    }

    /// Verification page: mark the device session identified by `user_code`
    /// as denied.
    pub async fn deny_device_user_code(
        &self,
        user_code: String,
        user_id: uuid::Uuid,
    ) -> Result<(), DeviceFlowError> {
        self.device_flow_service.deny(user_code, user_id).await
    }

    pub async fn run_data_migrations(&self) -> Result<MigrationReport, MigrationError> {
        let ctx = MigrationContext::new(
            self.realm_service.realm_repository.clone(),
            self.realm_service.client_repository.clone(),
            self.realm_service.client_scope_repository.clone(),
            self.realm_service.protocol_mapper_repository.clone(),
            self.realm_service.client_scope_mapping_repository.clone(),
        );

        build_runner(PostgresMigrationRepository::new(self.db.clone()))
            .run(&ctx)
            .await
    }
}
