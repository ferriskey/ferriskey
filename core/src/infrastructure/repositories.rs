use crate::domain::common::AppConfig;
use crate::infrastructure::auth_session::AuthSessionRepoAny;
use crate::infrastructure::client::repositories::client_postgres_repository::PostgresClientRepository;
use crate::infrastructure::client::repositories::redirect_uri_postgres_repository::PostgresRedirectUriRepository;
use crate::infrastructure::client::repositories::{ClientRepoAny, RedirectUriRepoAny};
use crate::infrastructure::credential::CredentialRepoAny;
use crate::infrastructure::db::postgres::{Postgres, PostgresConfig};
use crate::infrastructure::hasher::HasherRepoAny;
use crate::infrastructure::health::HealthCheckRepoAny;
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;
use crate::infrastructure::jwt::KeyStoreRepoAny;
use crate::infrastructure::realm::repositories::RealmRepoAny;
use crate::infrastructure::realm::repositories::realm_postgres_repository::PostgresRealmRepository;
use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;
use crate::infrastructure::repositories::auth_session_repository::PostgresAuthSessionRepository;
use crate::infrastructure::repositories::credential_repository::PostgresCredentialRepository;
use crate::infrastructure::repositories::keystore_repository::PostgresKeyStoreRepository;
use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;
use crate::infrastructure::role::repositories::RoleRepoAny;
use crate::infrastructure::role::repositories::role_postgres_repository::PostgresRoleRepository;
use crate::infrastructure::user::UserRepoAny;
use crate::infrastructure::user::repositories::user_required_action_repository::{
    PostgresUserRequiredActionRepository, UserRequiredActionRepoAny,
};
use crate::infrastructure::user::repositories::user_role_repository::{
    PostgresUserRoleRepository, UserRoleRepoAny,
};
use crate::infrastructure::user::repository::PostgresUserRepository;
use crate::infrastructure::webhook::repository::{PostgresWebhookRepository, WebhookRepoAny};

pub mod argon2_hasher;
pub mod auth_session_repository;
pub mod credential_repository;
pub mod keystore_repository;
pub mod refresh_token_repository;

pub struct RepoBundle {
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
}

pub async fn build_repos_from_env(cfg: AppConfig) -> Result<RepoBundle, anyhow::Error> {
    let postgres = Postgres::new(PostgresConfig {
        database_url: cfg.database_url,
    })
    .await?;

    let realm_repository = RealmRepoAny::Postgres(PostgresRealmRepository::new(postgres.get_db()));
    let client_repository =
        ClientRepoAny::Postgres(PostgresClientRepository::new(postgres.get_db()));
    let user_repository = UserRepoAny::Postgres(PostgresUserRepository::new(postgres.get_db()));
    let credential_repository =
        CredentialRepoAny::Postgres(PostgresCredentialRepository::new(postgres.get_db()));
    let hasher_repository = HasherRepoAny::Argon2(Argon2HasherRepository::new());
    let auth_session_repository =
        AuthSessionRepoAny::Postgres(PostgresAuthSessionRepository::new(postgres.get_db()));
    let refresh_token_repository =
        RefreshTokenRepoAny::Postgres(PostgresRefreshTokenRepository::new(postgres.get_db()));
    let redirect_uri_repository =
        RedirectUriRepoAny::Postgres(PostgresRedirectUriRepository::new(postgres.get_db()));
    let role_repository = RoleRepoAny::Postgres(PostgresRoleRepository::new(postgres.get_db()));
    let keystore_repository =
        KeyStoreRepoAny::Postgres(PostgresKeyStoreRepository::new(postgres.get_db()));
    let user_role_repository =
        UserRoleRepoAny::Postgres(PostgresUserRoleRepository::new(postgres.get_db()));
    let user_required_action_repository = UserRequiredActionRepoAny::Postgres(
        PostgresUserRequiredActionRepository::new(postgres.get_db()),
    );
    let health_check_repository =
        HealthCheckRepoAny::Postgres(PostgresHealthCheckRepository::new(postgres.get_db()));

    let webhook_repository =
        WebhookRepoAny::Postgres(PostgresWebhookRepository::new(postgres.get_db()));

    Ok(RepoBundle {
        realm_repository,
        client_repository,
        user_repository,
        credential_repository,
        hasher_repository,
        auth_session_repository,
        refresh_token_repository,
        redirect_uri_repository,
        role_repository,
        keystore_repository,
        user_role_repository,
        user_required_action_repository,
        health_check_repository,
        webhook_repository,
    })
}
