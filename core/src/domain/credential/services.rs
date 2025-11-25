use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    credential::{
        entities::{CredentialOverview, DeleteCredentialInput, GetCredentialsInput},
        ports::{CredentialRepository, CredentialService},
    },
    realm::ports::RealmRepository,
    user::ports::{UserPolicy, UserRepository, UserRoleRepository},
};

#[derive(Clone)]
pub struct CredentialServiceImpl<R, U, C, UR, CR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) credential_repository: Arc<CR>,

    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, CR> CredentialServiceImpl<R, U, C, UR, CR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        credential_repository: Arc<CR>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            credential_repository,
            policy,
        }
    }
}

impl<R, U, C, UR, CR> CredentialService for CredentialServiceImpl<R, U, C, UR, CR>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    CR: CredentialRepository,
{
    async fn get_credentials(
        &self,
        identity: Identity,
        input: GetCredentialsInput,
    ) -> Result<Vec<CredentialOverview>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(identity, realm).await,
            "insufficient permissions",
        )?;

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(input.user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        Ok(credentials
            .into_iter()
            .map(CredentialOverview::from)
            .collect())
    }

    async fn delete_credential(
        &self,
        identity: Identity,
        input: DeleteCredentialInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_user(identity, realm).await,
            "insufficient permissions",
        )?;

        self.credential_repository
            .delete_by_id(input.credential_id)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        // @TODO: implement webhook notifier

        Ok(())
    }
}
