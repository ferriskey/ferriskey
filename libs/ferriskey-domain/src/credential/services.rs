use std::sync::Arc;

use tracing::warn;

use crate::auth::Identity;
use crate::common::app_errors::CoreError;
use crate::common::policies::ensure_policy;
use crate::credential::entities::{CredentialOverview, DeleteCredentialInput, GetCredentialsInput};
use crate::credential::ports::{CredentialRepository, CredentialService};
use crate::realm::ports::RealmRepository;
use crate::realm::scope::RealmScope;
use crate::user::ports::{UserPolicy, UserRepository};

/// Generic over the policy rather than bound to a concrete engine, mirroring
/// `UserSessionManagementServiceImpl`. The kernel states which contract it
/// needs; the engine satisfying it lives in `ferriskey-authz`, which depends on
/// the kernel — so naming it here would close a cycle.
#[derive(Clone, Debug)]
pub struct CredentialServiceImpl<R, U, CR, P>
where
    R: RealmRepository,
    U: UserRepository,
    CR: CredentialRepository,
    P: UserPolicy,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) credential_repository: Arc<CR>,
    pub(crate) user_repository: Arc<U>,

    pub(crate) policy: Arc<P>,
}

impl<R, U, CR, P> CredentialServiceImpl<R, U, CR, P>
where
    R: RealmRepository,
    U: UserRepository,
    CR: CredentialRepository,
    P: UserPolicy,
{
    pub fn new(
        realm_repository: Arc<R>,
        credential_repository: Arc<CR>,
        user_repository: Arc<U>,
        policy: Arc<P>,
    ) -> Self {
        Self {
            realm_repository,
            credential_repository,
            user_repository,
            policy,
        }
    }
}

impl<R, U, CR, P> CredentialService for CredentialServiceImpl<R, U, CR, P>
where
    R: RealmRepository,
    U: UserRepository,
    CR: CredentialRepository,
    P: UserPolicy,
{
    async fn get_credentials(
        &self,
        identity: Identity,
        input: GetCredentialsInput,
    ) -> Result<Vec<CredentialOverview>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_view_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        self.user_repository
            .get_by_id(input.user_id)
            .await?
            .in_realm(&scope)?;

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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;
        let realm = scope.realm().clone();

        ensure_policy(
            self.policy.can_delete_user(&identity, &realm).await,
            "insufficient permissions",
        )?;

        let user = self
            .user_repository
            .get_by_id(input.user_id)
            .await?
            .in_realm(&scope)?;

        let owns_credential = self
            .credential_repository
            .get_credentials_by_user_id(input.user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?
            .iter()
            .any(|credential| credential.id == input.credential_id);

        if !owns_credential {
            warn!(
                user_id = %input.user_id,
                credential_id = %input.credential_id,
                "Refused deletion of a credential that does not belong to the target user"
            );
            return Err(CoreError::NotFound);
        }

        self.credential_repository
            .delete_by_id(&user, input.credential_id)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        // @TODO: implement webhook notifier

        Ok(())
    }
}
