use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::{Realm, RealmLoginSetting, RealmSetting},
    user::entities::User,
};

pub trait RealmService: Send + Sync {
    fn get_realms_by_user(
        &self,
        identity: Identity,
    ) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_realm_by_name(
        &self,
        identity: Identity,
        input: GetRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn get_realm_setting_by_name(
        &self,
        identity: Identity,
        input: GetRealmSettingInput,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn create_realm(
        &self,
        identity: Identity,
        input: CreateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn create_realm_with_user(
        &self,
        identity: Identity,
        input: CreateRealmWithUserInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        identity: Identity,
        input: UpdateRealmInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm_setting(
        &self,
        identity: Identity,
        input: UpdateRealmSettingInput,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn delete_realm(
        &self,
        identity: Identity,
        input: DeleteRealmInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
    fn get_login_settings(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<RealmLoginSetting, CoreError>> + Send;
}

pub trait RealmPolicy: Send + Sync {
    fn can_create_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_delete_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_view_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
    fn can_update_realm(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait RealmRepository: Send + Sync {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, CoreError>> + Send;

    fn get_by_name(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<Realm>, CoreError>> + Send;

    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, CoreError>> + Send;

    fn update_realm(
        &self,
        realm_name: String,
        name: String,
    ) -> impl Future<Output = Result<Realm, CoreError>> + Send;
    fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: Option<String>,
        user_registration_enabled: Option<bool>,
        forgot_password_enabled: Option<bool>,
        remember_me_enabled: Option<bool>,
    ) -> impl Future<Output = Result<RealmSetting, CoreError>> + Send;

    fn get_realm_settings(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<RealmSetting>, CoreError>> + Send;
}

pub struct GetRealmInput {
    pub realm_name: String,
}

pub struct GetRealmSettingInput {
    pub realm_name: String,
}

pub struct CreateRealmInput {
    pub realm_name: String,
}

pub struct CreateRealmWithUserInput {
    pub realm_name: String,
    pub user: User,
}

pub struct UpdateRealmInput {
    pub realm_name: String,
    pub name: String,
}

pub struct UpdateRealmSettingInput {
    pub realm_name: String,
    pub algorithm: Option<String>,

    pub user_registration_enabled: Option<bool>,
    pub forgot_password_enabled: Option<bool>,
    pub remember_me_enabled: Option<bool>,
}

pub struct DeleteRealmInput {
    pub realm_name: String,
}
