use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{AuthenticationError, JwtToken},
        ports::AuthenticationService,
        value_objects::{AuthenticateRequest, AuthenticationResult},
    },
    client::ports::ClientRepository,
    credential::ports::CredentialRepository,
    realm::ports::RealmRepository,
    user::ports::UserRepository,
};

#[derive(Clone)]
pub struct AuthenticationServiceImpl<R, C, U, CR>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
{
    pub realm_repository: R,
    pub client_repository: C,
    pub user_repository: U,
    pub credential_repository: CR,
}

impl<R, C, U, CR> AuthenticationServiceImpl<R, C, U, CR>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
{
    pub fn new(
        realm_repository: R,
        client_repository: C,
        user_repository: U,
        credential_repository: CR,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
        }
    }
}

impl<R, C, U, CR> AuthenticationService for AuthenticationServiceImpl<R, C, U, CR>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
{
    async fn authenticate(
        &self,
        data: AuthenticateRequest,
        base_url: String,
    ) -> Result<JwtToken, AuthenticationError> {
        todo!()
    }

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?
            .ok_or(AuthenticationError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(client_id.clone(), realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient)?;

        let user = self
            .user_repository
            .get_by_username(username, realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidUser)?;

        let user_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| AuthenticationError::InvalidUser)?;

        let _: Vec<String> = user_credentials
            .iter()
            .map(|cred| cred.credential_type.clone())
            .collect();

        todo!()
    }
}
