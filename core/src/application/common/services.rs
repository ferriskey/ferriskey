use crate::domain::common::AppConfig;
use crate::domain::trident::services::OauthTotpService;
use crate::domain::webhook::services::webhook_notifier_service::WebhookNotifierServiceImpl;
use crate::domain::{
    authentication::services::auth_session_service::AuthSessionServiceImpl,
    client::services::{
        client_service::ClientServiceImpl, redirect_uri_service::RedirectUriServiceImpl,
    },
    jwt::services::JwtServiceImpl,
    realm::services::RealmServiceImpl,
    user::services::{user_role_service::UserRoleServiceImpl, user_service::UserServiceImpl},
};
use crate::infrastructure::auth_session::AuthSessionRepoAny;
use crate::infrastructure::client::repositories::{ClientRepoAny, RedirectUriRepoAny};
use crate::infrastructure::jwt::KeyStoreRepoAny;
use crate::infrastructure::realm::repositories::RealmRepoAny;
use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
use crate::infrastructure::repositories::build_repos_from_env;
use crate::infrastructure::role::repositories::RoleRepoAny;
use crate::infrastructure::user::UserRepoAny;
use crate::infrastructure::user::repositories::user_required_action_repository::UserRequiredActionRepoAny;
use crate::infrastructure::user::repositories::user_role_repository::UserRoleRepoAny;
use crate::infrastructure::webhook::repositories::webhook_repository::WebhookRepoAny;

pub type DefaultUserService =
    UserServiceImpl<UserRepoAny, RealmRepoAny, UserRoleRepoAny, UserRequiredActionRepoAny>;

pub type DefaultRealmService =
    RealmServiceImpl<RealmRepoAny, ClientRepoAny, RoleRepoAny, UserRepoAny, UserRoleRepoAny>;

pub type DefaultAuthSessionService = AuthSessionServiceImpl<AuthSessionRepoAny>;

pub type DefaultClientService = ClientServiceImpl<ClientRepoAny, UserRepoAny, RealmRepoAny>;

pub type DefaultUserRoleService =
    UserRoleServiceImpl<UserRepoAny, RoleRepoAny, RealmRepoAny, UserRoleRepoAny>;

pub type DefaultJwtService = JwtServiceImpl<RefreshTokenRepoAny, KeyStoreRepoAny, RealmRepoAny>;

pub type DefaultRedirectUriService =
    RedirectUriServiceImpl<RealmRepoAny, RedirectUriRepoAny, ClientRepoAny>;

pub type DefaultWebhookNotifierService = WebhookNotifierServiceImpl<WebhookRepoAny>;

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

        let user_role_service = DefaultUserRoleService::new(
            repositories.user_repository.clone(),
            repositories.role_repository.clone(),
            repositories.realm_repository.clone(),
            repositories.user_role_repository.clone(),
        );

        let totp_service = OauthTotpService::new();

        let webhook_notifier_service =
            WebhookNotifierServiceImpl::new(repositories.webhook_repository.clone());

        Ok(ServiceBundle {
            realm_service,
            client_service,
            auth_session_service,
            user_service,
            jwt_service,
            redirect_uri_service,
            user_role_service,
            totp_service,
            webhook_notifier_service,
        })
    }
}

#[derive(Clone)]
pub struct ServiceBundle {
    pub realm_service: DefaultRealmService,
    pub client_service: DefaultClientService,
    pub auth_session_service: DefaultAuthSessionService,
    pub user_service: DefaultUserService,
    pub jwt_service: DefaultJwtService,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub user_role_service: DefaultUserRoleService,
    pub totp_service: OauthTotpService,
    pub webhook_notifier_service: DefaultWebhookNotifierService,
}
