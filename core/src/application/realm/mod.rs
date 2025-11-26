use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        realm::{
            entities::{Realm, RealmLoginSetting, RealmSetting},
            ports::{
                CreateRealmInput, CreateRealmWithUserInput, DeleteRealmInput, GetRealmInput,
                GetRealmSettingInput, RealmService, UpdateRealmInput, UpdateRealmSettingInput,
            },
        },
    },
};
use tracing::instrument;

impl RealmService for ApplicationService {
    async fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.create_realm(identity, input).await
    }

    async fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service
            .create_realm_with_user(identity, input)
            .await
    }

    async fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> Result<(), CoreError> {
        self.realm_service.delete_realm(identity, input).await
    }

    async fn get_login_settings(&self, realm_name: String) -> Result<RealmLoginSetting, CoreError> {
        self.realm_service.get_login_settings(realm_name).await
    }

    #[instrument]
    async fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.get_realm_by_name(identity, input).await
    }

    async fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> Result<RealmSetting, CoreError> {
        self.realm_service
            .get_realm_setting_by_name(identity, input)
            .await
    }

    async fn get_realms_by_user(&self, identity: Identity) -> Result<Vec<Realm>, CoreError> {
        self.realm_service.get_realms_by_user(identity).await
    }

    async fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service.update_realm(identity, input).await
    }

    async fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> Result<Realm, CoreError> {
        self.realm_service
            .update_realm_setting(identity, input)
            .await
    }
}
