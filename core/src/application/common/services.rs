use crate::domain::authentication::strategies::authorization_code_strategy::AuthorizationCodeStrategy;
use crate::domain::authentication::strategies::client_credentials_strategy::ClientCredentialsStrategy;
use crate::domain::authentication::strategies::password_strategy::PasswordStrategy;
use crate::domain::authentication::strategies::refresh_token_strategy::RefreshTokenStrategy;
use crate::domain::common::AppConfig;
use crate::domain::health::services::HealthCheckServiceImpl;
use crate::domain::trident::services::OauthTotpService;
use crate::domain::{
    authentication::services::{
        auth_session_service::AuthSessionServiceImpl, grant_type_service::GrantTypeServiceImpl,
    },
    client::services::{
        client_service::ClientServiceImpl, redirect_uri_service::RedirectUriServiceImpl,
    },
    credential::services::CredentialServiceImpl,
    crypto::services::CryptoServiceImpl,
    jwt::services::JwtServiceImpl,
    realm::services::RealmServiceImpl,
    role::services::RoleServiceImpl,
    user::services::{user_role_service::UserRoleServiceImpl, user_service::UserServiceImpl},
};
use crate::infrastructure::auth_session::AuthSessionRepoAny;
use crate::infrastructure::client::ClientRepoAny;
use crate::infrastructure::client::repositories::RedirectUriRepoAny;
use crate::infrastructure::credential::CredentialRepoAny;
use crate::infrastructure::hasher::HasherRepoAny;
use crate::infrastructure::health::HealthCheckRepoAny;
use crate::infrastructure::jwt::KeyStoreRepoAny;
use crate::infrastructure::realm::RealmRepoAny;
use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
use crate::infrastructure::repositories::build_repos_from_env;
use crate::infrastructure::role::RoleRepoAny;
use crate::infrastructure::user::UserRepoAny;
use crate::infrastructure::user::repositories::user_required_action_repository::UserRequiredActionRepoAny;
use crate::infrastructure::user::repositories::user_role_repository::UserRoleRepoAny;

pub type DefaultUserService =
    UserServiceImpl<UserRepoAny, RealmRepoAny, UserRoleRepoAny, UserRequiredActionRepoAny>;

pub type DefaultRealmService =
    RealmServiceImpl<RealmRepoAny, ClientRepoAny, RoleRepoAny, UserRepoAny, UserRoleRepoAny>;

pub type DefaultAuthSessionService = AuthSessionServiceImpl<AuthSessionRepoAny>;
pub type DefaultGrantTypeService = GrantTypeServiceImpl<
    RefreshTokenRepoAny,
    KeyStoreRepoAny,
    RealmRepoAny,
    ClientRepoAny,
    UserRepoAny,
    UserRoleRepoAny,
    UserRequiredActionRepoAny,
    CredentialRepoAny,
    HasherRepoAny,
    AuthSessionRepoAny,
>;

pub type DefaultClientService = ClientServiceImpl<ClientRepoAny, UserRepoAny, RealmRepoAny>;

pub type DefaultCredentialService = CredentialServiceImpl<CredentialRepoAny, HasherRepoAny>;

pub type DefaultCryptoService = CryptoServiceImpl<HasherRepoAny>;

pub type DefaultRoleService = RoleServiceImpl<RoleRepoAny>;

pub type DefaultUserRoleService =
    UserRoleServiceImpl<UserRepoAny, RoleRepoAny, RealmRepoAny, UserRoleRepoAny>;

pub type DefaultJwtService = JwtServiceImpl<RefreshTokenRepoAny, KeyStoreRepoAny, RealmRepoAny>;

pub type DefaultRedirectUriService =
    RedirectUriServiceImpl<RealmRepoAny, RedirectUriRepoAny, ClientRepoAny>;

pub type DefaultHealthCheckService = HealthCheckServiceImpl<HealthCheckRepoAny>;

pub struct ServiceFactory;

pub struct ServiceFactoryConfig {
    pub database_url: String,
}

impl ServiceFactory {
    pub async fn create_all_services(
        config: ServiceFactoryConfig,
    ) -> Result<ServiceBundle, anyhow::Error> {
        let repositories = build_repos_from_env(AppConfig {
            database_url: config.database_url,
        })
        .await?;

        let realm_service = DefaultRealmService::new(
            repositories.realm_repository.clone(),
            repositories.client_repository.clone(),
            repositories.role_repository.clone(),
            repositories.user_repository.clone(),
            repositories.user_role_repository.clone(),
        );

        let client_service = DefaultClientService::new(
            repositories.client_repository.clone(),
            repositories.user_repository.clone(),
            repositories.realm_repository.clone(),
        );

        let redirect_uri_service = DefaultRedirectUriService::new(
            repositories.realm_repository.clone(),
            repositories.redirect_uri_repository.clone(),
            repositories.client_repository.clone(),
        );

        let crypto_service = DefaultCryptoService::new(repositories.hasher_repository.clone());

        let credential_service = DefaultCredentialService::new(
            repositories.credential_repository.clone(),
            crypto_service.clone(),
        );

        let auth_session_service =
            DefaultAuthSessionService::new(repositories.auth_session_repository.clone());

        let user_service = DefaultUserService::new(
            repositories.user_repository.clone(),
            repositories.realm_repository.clone(),
            repositories.user_role_repository.clone(),
            repositories.user_required_action_repository.clone(),
        );

        let jwt_service = DefaultJwtService::new(
            repositories.refresh_token_repository.clone(),
            repositories.keystore_repository.clone(),
            repositories.realm_repository.clone(),
        );

        let role_service = DefaultRoleService::new(repositories.role_repository.clone());

        let user_role_service = DefaultUserRoleService::new(
            repositories.user_repository.clone(),
            repositories.role_repository.clone(),
            repositories.realm_repository.clone(),
            repositories.user_role_repository.clone(),
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

        let health_check_service =
            DefaultHealthCheckService::new(repositories.health_check_repository.clone());

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
