use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRedirectUriService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::client::entities::redirect_uri::RedirectUri;
use crate::domain::client::ports::RedirectUriService;
use crate::domain::realm::ports::RealmService;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetRedirectUrisUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    redirect_uri_service: DefaultRedirectUriService,
}

pub struct GetRedirectUrisUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
}

impl GetRedirectUrisUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
        redirect_uri_service: DefaultRedirectUriService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
            redirect_uri_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: GetRedirectUrisUseCaseParams,
    ) -> Result<Vec<RedirectUri>, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let can_view = ClientPolicy::view(
            identity,
            realm,
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_view {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to view redirect URIs".to_string(),
            ));
        }

        self.redirect_uri_service
            .get_by_client_id(params.client_id)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }
}
