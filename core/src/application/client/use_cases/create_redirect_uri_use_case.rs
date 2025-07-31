use uuid::Uuid;

use crate::{
    application::common::services::DefaultRedirectUriService,
    domain::client::{
        entities::{ClientError, redirect_uri::RedirectUri},
        ports::RedirectUriService,
        value_objects::CreateRedirectUriRequest,
    },
};

#[derive(Clone)]
pub struct CreateRedirectUriUseCase {
    redirect_uri_service: DefaultRedirectUriService,
}

pub struct CreateRedirectUriUseCaseParams {
    pub client_id: Uuid,
    pub realm_name: String,
    pub payload: CreateRedirectUriRequest,
}

impl CreateRedirectUriUseCase {
    pub fn new(redirect_uri_service: DefaultRedirectUriService) -> Self {
        Self {
            redirect_uri_service,
        }
    }

    pub async fn execute(
        &self,
        params: CreateRedirectUriUseCaseParams,
    ) -> Result<RedirectUri, ClientError> {
        self.redirect_uri_service
            .add_redirect_uri(params.payload, params.realm_name, params.client_id)
            .await
    }
}
