use uuid::Uuid;

use crate::{
    application::{
        client::policies::ClientPolicy,
        common::services::{DefaultClientService, DefaultRealmService, DefaultUserService},
    },
    domain::{
        authentication::value_objects::Identity,
        client::{entities::ClientError, ports::ClientService},
        realm::ports::RealmService,
    },
};

#[derive(Clone)]
pub struct DeleteClientUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct DeleteClientUseCaseParams {
    pub realm_name: String,
    pub client_id: Uuid,
}

impl DeleteClientUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        Self {
            realm_service,
            user_service,
            client_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: DeleteClientUseCaseParams,
    ) -> Result<(), ClientError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Self::ensure_permissions(
            ClientPolicy::delete(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await,
            "Insufficient permissions to delete client",
        )?;

        self.client_service.delete_by_id(params.client_id).await?;

        Ok(())
    }

    #[inline]
    fn ensure_permissions(
        result_has_permission: Result<bool, ClientError>,
        error_message: &str,
    ) -> Result<(), ClientError> {
        result_has_permission
            .map_err(|_| ClientError::Forbidden(error_message.to_string()))?
            .then_some(())
            .ok_or_else(|| ClientError::Forbidden(error_message.to_string()))
    }
}
