use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRedirectUriService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::client::ports::RedirectUriService;
use crate::domain::realm::ports::RealmService;
use uuid::Uuid;

#[derive(Clone)]
pub struct DeleteRedirectUriUseCase {
    redirect_uri_service: DefaultRedirectUriService,
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct DeleteRedirectUriUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

impl DeleteRedirectUriUseCase {
    pub fn new(
        redirect_uri_service: DefaultRedirectUriService,
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        Self {
            redirect_uri_service,
            realm_service,
            user_service,
            client_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: DeleteRedirectUriUseCaseParams,
    ) -> Result<(), ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        ClientPolicy::delete(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await
        .map_err(|_| {
            ClientError::Forbidden("Insufficient permissions to delete redirect URI".to_string())
        })?
        .then_some(())
        .ok_or_else(|| {
            ClientError::Forbidden("Insufficient permissions to delete redirect URI".to_string())
        })?;

        self.redirect_uri_service
            .delete(params.uri_id)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }
}
