use crate::{
    application::common::permissions::FerriskeyPolicy,
    domain::common::{AppConfig, FerriskeyConfig},
    infrastructure::{
        auth_session::AuthSessionRepoAny,
        client::repositories::{ClientRepoAny, RedirectUriRepoAny},
        credential::CredentialRepoAny,
        hasher::HasherRepoAny,
        health::HealthCheckRepoAny,
        jwt::KeyStoreRepoAny,
        realm::repositories::RealmRepoAny,
        refresh_token::RefreshTokenRepoAny,
        repositories::build_repos_from_env,
        role::repositories::RoleRepoAny,
        user::{
            UserRepoAny,
            repositories::{
                user_required_action_repository::UserRequiredActionRepoAny,
                user_role_repository::UserRoleRepoAny,
            },
        },
        webhook::repositories::{
            webhook_notifier_repository::WebhookNotifierRepoAny, webhook_repository::WebhookRepoAny,
        },
    },
};

pub mod factories;
pub mod permissions;
pub mod policies;
pub mod services;

#[derive(Clone)]
pub struct FerriskeyService {
    pub config: FerriskeyConfig,
    pub realm_repository: RealmRepoAny,
    pub client_repository: ClientRepoAny,
    pub user_repository: UserRepoAny,
    pub credential_repository: CredentialRepoAny,
    pub hasher_repository: HasherRepoAny,
    pub auth_session_repository: AuthSessionRepoAny,
    pub refresh_token_repository: RefreshTokenRepoAny,
    pub redirect_uri_repository: RedirectUriRepoAny,
    pub role_repository: RoleRepoAny,
    pub keystore_repository: KeyStoreRepoAny,
    pub user_role_repository: UserRoleRepoAny,
    pub user_required_action_repository: UserRequiredActionRepoAny,
    pub health_check_repository: HealthCheckRepoAny,
    pub webhook_repository: WebhookRepoAny,
    pub policy: FerriskeyPolicy,
    pub webhook_notifier_repository: WebhookNotifierRepoAny,
}

impl FerriskeyService {
    pub async fn new(config: FerriskeyConfig) -> Result<Self, anyhow::Error> {
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.database.username,
            config.database.password,
            config.database.host,
            config.database.port,
            config.database.name
        );
        let repos = build_repos_from_env(AppConfig { database_url }).await?;

        let policy = FerriskeyPolicy::new(
            repos.user_repository.clone(),
            repos.client_repository.clone(),
            repos.user_role_repository.clone(),
        );

        Ok(FerriskeyService {
            realm_repository: repos.realm_repository,
            client_repository: repos.client_repository,
            user_repository: repos.user_repository,
            credential_repository: repos.credential_repository,
            hasher_repository: repos.hasher_repository,
            auth_session_repository: repos.auth_session_repository,
            refresh_token_repository: repos.refresh_token_repository,
            redirect_uri_repository: repos.redirect_uri_repository,
            role_repository: repos.role_repository,
            keystore_repository: repos.keystore_repository,
            user_role_repository: repos.user_role_repository,
            user_required_action_repository: repos.user_required_action_repository,
            health_check_repository: repos.health_check_repository,
            webhook_repository: repos.webhook_repository,
            webhook_notifier_repository: repos.webhook_notifier_repository,
            config,

            policy,
        })
    }
}
