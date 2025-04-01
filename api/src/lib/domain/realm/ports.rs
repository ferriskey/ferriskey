use super::entities::{error::RealmError, realm::Realm};
use crate::domain::realm::entities::realm_setting::RealmSetting;
use uuid::Uuid;

pub trait RealmService: Clone + Send + Sync + 'static {
    fn fetch_realm(&self) -> impl Future<Output = Result<Vec<Realm>, RealmError>> + Send;
    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn update_realm(
        &self,
        realm_name: String,
        name: String,
    ) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn delete_by_name(&self, name: String) -> impl Future<Output = Result<(), RealmError>> + Send;
    fn get_by_name(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    //fn create_realm_master(&self) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;
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
}
