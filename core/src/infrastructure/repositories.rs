use crate::domain::common::AppConfig;
use crate::infrastructure::client::ClientRepoAny;
use crate::infrastructure::db::postgres::{Postgres, PostgresConfig};
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;
use crate::infrastructure::realm::RealmRepoAny;
use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;
use crate::infrastructure::repositories::auth_session_repository::PostgresAuthSessionRepository;
use crate::infrastructure::repositories::client_repository::PostgresClientRepository;
use crate::infrastructure::repositories::credential_repository::PostgresCredentialRepository;
use crate::infrastructure::repositories::keystore_repository::PostgresKeyStoreRepository;
use crate::infrastructure::repositories::realm_repository::PostgresRealmRepository;
use crate::infrastructure::repositories::redirect_uri_repository::PostgresRedirectUriRepository;
use crate::infrastructure::repositories::refresh_token_repository::PostgresRefreshTokenRepository;
use crate::infrastructure::repositories::role_repository::PostgresRoleRepository;
use crate::infrastructure::user::UserRepoAny;
use crate::infrastructure::user::repositories::user_required_action_repository::PostgresUserRequiredActionRepository;
use crate::infrastructure::user::repositories::user_role_repository::PostgresUserRoleRepository;
use crate::infrastructure::user::repository::PostgresUserRepository;

pub mod argon2_hasher;
pub mod auth_session_repository;
pub mod client_repository;
pub mod credential_repository;
pub mod keystore_repository;
pub mod realm_repository;
pub mod redirect_uri_repository;
pub mod refresh_token_repository;
pub mod role_repository;
pub mod user_session_repository;

pub struct RepoBundle {
    pub realm_repository: RealmRepoAny,
    pub client_repository: ClientRepoAny,
    pub user_repository: UserRepoAny,
    pub credential_repository: PostgresCredentialRepository,
    pub hasher_repository: Argon2HasherRepository,
    pub auth_session_repository: PostgresAuthSessionRepository,
    pub refresh_token_repository: PostgresRefreshTokenRepository,
    pub redirect_uri_repository: PostgresRedirectUriRepository,
    pub role_repository: PostgresRoleRepository,
    pub keystore_repository: PostgresKeyStoreRepository,
    pub user_role_repository: PostgresUserRoleRepository,
    pub user_required_action_repository: PostgresUserRequiredActionRepository,
    pub health_check_repository: PostgresHealthCheckRepository,
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
    })
}
