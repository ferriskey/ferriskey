use crate::domain::realm::entities::{
    error::RealmError, realm::Realm, realm_setting::RealmSetting,
};
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

    fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> impl Future<Output = Result<RealmSetting, RealmError>> + Send;
}
