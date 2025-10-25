use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::{
        entities::{CredentialOverview, GetCredentialsInput},
        ports::{CredentialRepository, CredentialService},
    },
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserPolicy, UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC> CredentialService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
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
        input: crate::domain::credential::entities::DeleteCredentialInput,
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
