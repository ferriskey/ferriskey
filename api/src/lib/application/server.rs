use std::sync::Arc;

use crate::{
    domain::{
        authentication::ports::auth_session::AuthSessionRepository,
        client::ports::ClientRepository, credential::ports::CredentialRepository,
        crypto::ports::HasherRepository, jwt::ports::JwtRepository, realm::ports::RealmRepository,
        user::ports::UserRepository,
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
            user_repository::PostgresUserRepository,
        },
    },
};

pub struct AppServer<R, C, U, CR, H>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
{
    pub postgres: Arc<Postgres>,
    pub realm_repository: R,
    pub client_repository: C,
    pub user_repository: U,
    pub credential_repository: CR,
    pub hasher_repository: H,
    pub jwt_repository: Box<dyn JwtRepository>,
    pub auth_session_repository: Box<dyn AuthSessionRepository>,
}

impl
    AppServer<
        PostgresRealmRepository,
        PostgresClientRepository,
        PostgresUserRepository,
        PostgresCredentialRepository,
        Argon2HasherRepository,
    >
{
    pub async fn new(env: Arc<Env>) -> Result<Self, anyhow::Error> {
        let postgres = Arc::new(Postgres::new(Arc::clone(&env)).await?);
        let realm_repository = PostgresRealmRepository::new(Arc::clone(&postgres));
        let client_repository = PostgresClientRepository::new(Arc::clone(&postgres));
        let user_repository = PostgresUserRepository::new(Arc::clone(&postgres));
        let credential_repository = PostgresCredentialRepository::new(Arc::clone(&postgres));
        let hasher_repository = Argon2HasherRepository::new();
        let jwt_repository = Box::new(StaticJwtRepository::new(&env.private_key, &env.public_key)?);
        let auth_session_repository =
            Box::new(PostgresAuthSessionRepository::new(Arc::clone(&postgres)));

        Ok(Self {
            postgres,
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            jwt_repository,
            auth_session_repository,
        })
    }
}
