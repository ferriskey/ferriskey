use super::entities::{error::RealmError, model::Realm};

pub trait RealmService: Clone + Send + Sync + 'static {
    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn get_by_name(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn create_realm_master(&self) -> impl Future<Output = Result<Realm, RealmError>> + Send;
}

pub trait RealmRepository: Clone + Send + Sync + 'static {
    fn create_realm(&self, name: String) -> impl Future<Output = Result<Realm, RealmError>> + Send;
    fn get_by_name(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Option<Realm>, RealmError>> + Send;
}
