use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::{Realm, RealmError, RealmSetting},
    user::entities::User,
};

pub trait RealmService: Clone + Send + Sync {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, RealmError>> + Send;

    fn create_realm_with_user(
        &self,
        name: String,
        user: &User,
    ) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;

    fn update_realm(
        &self,
        realm_name: String,
        name: String,
    ) -> impl Future<Output = Result<Realm, RealmError>> + Send;

    fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), RealmError>> + Send;

    fn get_by_name(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;

    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;

    fn get_realm_settings(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;
}

pub trait RealmPolicy: Send + Sync + Clone {
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

pub trait RealmRepository: Clone + Send + Sync + 'static {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, RealmError>> + Send;

    fn get_by_name(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<Realm>, RealmError>> + Send;

    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;

    fn update_realm(
        &self,
        realm_name: String,
        name: String,
    ) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), RealmError>> + Send;

    fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;

    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;

    fn get_realm_settings(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;
}
