use std::sync::Arc;

use crate::{
    domain::{
        authentication::ports::auth_session::AuthSessionRepository,
        client::ports::{
            client_repository::ClientRepository, redirect_uri_repository::RedirectUriRepository,
        },
        credential::ports::credential_repository::CredentialRepository,
        crypto::ports::hasher_repository::HasherRepository,
        jwt::ports::jwt_repository::{JwtRepository, RefreshTokenRepository},
        realm::ports::realm_repository::RealmRepository,
        user::ports::user_repository::UserRepository,
    },
    env::Env,
    infrastructure::{
        db::postgres::Postgres,
        repositories::{
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            client_repository::PostgresClientRepository,
            credential_repository::PostgresCredentialRepository,
            jwt_repository::StaticJwtRepository, realm_repository::PostgresRealmRepository,
            redirect_uri_repository::PostgresRedirectUriRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
            user_repository::PostgresUserRepository,
        },
    },
};

pub struct AppServer<R, C, U, CR, H, J, AS, RR, RU>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    J: JwtRepository,
    AS: AuthSessionRepository,
    RR: RefreshTokenRepository,
    RU: RedirectUriRepository,
{
    pub realm_repository: R,
    pub client_repository: C,
    pub user_repository: U,
    pub credential_repository: CR,
    pub hasher_repository: H,
    pub jwt_repository: J,
    pub auth_session_repository: AS,
    pub refresh_token_repository: RR,
    pub redirect_uri_repository: RU,
}

impl
    AppServer<
        PostgresRealmRepository,
        PostgresClientRepository,
        PostgresUserRepository,
        PostgresCredentialRepository,
        Argon2HasherRepository,
        StaticJwtRepository,
        PostgresAuthSessionRepository,
        PostgresRefreshTokenRepository,
        PostgresRedirectUriRepository,
    >
{
    pub async fn new(env: Arc<Env>) -> Result<Self, anyhow::Error> {
        let postgres = Postgres::new(Arc::clone(&env)).await?;
        let realm_repository = PostgresRealmRepository::new(postgres.get_pool());
        let client_repository = PostgresClientRepository::new(postgres.get_pool());
        let user_repository = PostgresUserRepository::new(postgres.get_pool());
        let credential_repository = PostgresCredentialRepository::new(postgres.get_pool());
        let hasher_repository = Argon2HasherRepository::new();
        let jwt_repository = StaticJwtRepository::new(&env.private_key, &env.public_key)?;
        let auth_session_repository = PostgresAuthSessionRepository::new(postgres.get_pool());
        let refresh_token_repository = PostgresRefreshTokenRepository::new(postgres.get_pool());
        let redirect_uri_repository = PostgresRedirectUriRepository::new(postgres.get_pool());

        Ok(Self {
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            jwt_repository,
            auth_session_repository,
            refresh_token_repository,
            redirect_uri_repository,
        })
    }
}
