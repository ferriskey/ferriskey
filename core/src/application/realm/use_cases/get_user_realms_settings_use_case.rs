use tracing::info;

use crate::{
    application::{
        common::{
            policies::ensure_permissions,
            services::{DefaultClientService, DefaultRealmService, DefaultUserService},
        },
        realm::policies::RealmPolicy,
    },
    domain::{
        authentication::value_objects::Identity,
        realm::{
            entities::{RealmError, RealmSetting},
            ports::RealmService,
        },
        user::ports::UserService as _,
    },
};

#[derive(Clone)]
pub struct GetUserRealmSettingsUseCase {
    realm_service: DefaultRealmService,
    user_service: DefaultUserService,
    client_service: DefaultClientService,
}

pub struct GetUserRealmSettingsUseCaseParams {
    pub realm_name: String,
}

impl GetUserRealmSettingsUseCase {
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
        params: GetUserRealmSettingsUseCaseParams,
    ) -> Result<RealmSetting, RealmError> {
        info!(
            "Getting user realms settings for user: {}",
            params.realm_name
        );
        let user = match identity.clone() {
            Identity::User(user) => user,
            Identity::Client(client) => self
                .user_service
                .get_by_client_id(client.id)
                .await
                .map_err(|_| RealmError::Forbidden)?,
        };

        let realm = user.realm.ok_or(RealmError::Forbidden)?;

        ensure_permissions(
            RealmPolicy::view(
                identity,
                realm.clone(),
                self.user_service.clone(),
                self.client_service.clone(),
            )
            .await
            .map_err(anyhow::Error::new),
            "Insufficient permissions to view realm settings",
        )
        .map_err(|_| RealmError::Forbidden)?;

        let realm_setting = self
            .realm_service
            .get_realm_settings(realm.id)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        Ok(realm_setting)
    }
}
