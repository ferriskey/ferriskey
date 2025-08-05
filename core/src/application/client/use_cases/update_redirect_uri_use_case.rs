use crate::application::client::policies::ClientPolicy;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultRedirectUriService, DefaultUserService,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::ClientError;
use crate::domain::client::entities::redirect_uri::RedirectUri;
use crate::domain::client::ports::RedirectUriService;
use crate::domain::realm::ports::RealmService;

#[derive(Clone)]
pub struct UpdateRedirectUriUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
    redirect_uri_service: DefaultRedirectUriService,
}

pub struct UpdateRedirectUriUseCaseParams {
    pub realm_name: String,
    pub client_id: uuid::Uuid,
    pub redirect_uri_id: uuid::Uuid,
    pub enabled: bool,
}

impl UpdateRedirectUriUseCase {
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
        params: UpdateRedirectUriUseCaseParams,
    ) -> Result<RedirectUri, ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let can_update = ClientPolicy::update(
            identity,
            realm.clone(),
            self.user_service.clone(),
            self.client_service.clone(),
        )
        .await?;

        if !can_update {
            return Err(ClientError::Forbidden(
                "Insufficient permissions to update redirect URI".to_string(),
            ));
        }

        self.redirect_uri_service
            .update_enabled(params.client_id, params.enabled)
            .await
            .map_err(|_| ClientError::InternalServerError)
    }
}
