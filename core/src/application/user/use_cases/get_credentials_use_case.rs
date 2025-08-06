use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultCredentialService, DefaultRealmService, DefaultUserService,
};
use crate::application::user::policies::user_policy::UserPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::credential::entities::CredentialOverview;
use crate::domain::credential::ports::CredentialService;
use crate::domain::realm::ports::RealmService;
use crate::domain::user::entities::UserError;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetCredentialsUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    credential_service: DefaultCredentialService,
}

pub struct GetCredentialsUseCaseParams {
    pub realm_name: String,
    pub user_id: Uuid,
}

impl GetCredentialsUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        credential_service: DefaultCredentialService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            credential_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: GetCredentialsUseCaseParams,
    ) -> Result<Vec<CredentialOverview>, UserError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        ensure_permissions(
            UserPolicy::view(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to view credentials",
        )
        .map_err(|e| UserError::Forbidden(e.to_string()))?;

        let credentials = self
            .credential_service
            .get_credentials_by_user_id(params.user_id)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        Ok(credentials
            .into_iter()
            .map(CredentialOverview::from)
            .collect())
    }
}
