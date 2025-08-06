use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{Realm, RealmError};
use crate::domain::realm::ports::RealmService;

#[derive(Clone)]
pub struct GetRealmUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct GetRealmUseCaseParams {
    pub realm_name: String,
}

impl GetRealmUseCase {
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
        params: GetRealmUseCaseParams,
    ) -> Result<Realm, RealmError> {
        let realm = self.realm_service.get_by_name(params.realm_name).await?;

        ensure_permissions(
            RealmPolicy::view(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to view realm",
        )
        .map_err(|_| RealmError::Forbidden)?;

        Ok(realm)
    }
}
