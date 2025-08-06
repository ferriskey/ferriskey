use crate::application::common::policies::ensure_permissions;
use crate::application::common::services::{
    DefaultClientService, DefaultRealmService, DefaultUserService,
};
use crate::application::realm::policies::RealmPolicy;
use crate::domain::authentication::value_objects::Identity;
use crate::domain::realm::entities::{RealmError, RealmSetting};
use crate::domain::realm::ports::RealmService;

#[derive(Clone)]
pub struct UpdateRealmSettingsUseCase {
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub client_service: DefaultClientService,
}

pub struct UpdateRealmSettingsUseCaseParams {
    pub realm_name: String,
    pub algorithm: String,
}

impl UpdateRealmSettingsUseCase {
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
        params: UpdateRealmSettingsUseCaseParams,
    ) -> Result<RealmSetting, RealmError> {
        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| RealmError::Invalid)?;

        let realm_id = realm.id;
        ensure_permissions(
            RealmPolicy::update(
                identity,
                realm,
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to update realm settings",
        )
        .map_err(|_| RealmError::Forbidden)?;

        self.realm_service
            .update_realm_setting(realm_id, params.algorithm)
            .await
            .map_err(|_| RealmError::InternalServerError)
    }
}
