use super::{
    entities::{error::RealmError, realm::Realm},
    ports::{RealmRepository, RealmService},
};
use crate::domain::realm::entities::realm_setting::RealmSetting;
use tracing::{error, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RealmServiceImpl<R>
where
    R: RealmRepository,
{
    pub realm_repository: R,
}

impl<R> RealmServiceImpl<R>
where
    R: RealmRepository,
{
    pub fn new(realm_repository: R) -> Self {
        Self { realm_repository }
    }
}

impl<R> RealmService for RealmServiceImpl<R>
where
    R: RealmRepository,
{
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
        self.realm_repository.fetch_realm().await
    }

    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        let realm = self.realm_repository.create_realm(name.clone()).await?;
        println!("Created realm: {:?}", realm);
        self.realm_repository
            .create_realm_settings(realm.id, "RS256".to_string())
            .await?;

        Ok(realm)
    }

    async fn update_realm(&self, realm_name: String, name: String) -> Result<Realm, RealmError> {
        self.realm_repository.update_realm(realm_name, name).await
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        let realm = self.get_by_name(name.clone()).await.map_err(|_| {
            error!("realm {} not found", name);
            RealmError::Forbidden
        })?;

        if !realm.can_delete() {
            error!("try to delete master realm");
            return Err(RealmError::CannotDeleteMaster);
        }
        self.realm_repository.delete_by_name(name).await
    }

    async fn get_by_name(&self, name: String) -> Result<Realm, RealmError> {
        self.realm_repository
            .get_by_name(name)
            .await?
            .ok_or(RealmError::NotFound)
    }

    async fn create_realm_master(&self) -> Result<Realm, RealmError> {
        info!("Introspecting realm master");
        let realm = self.get_by_name("master".to_string()).await;

        if let Ok(realm) = realm {
            info!("Realm master already exists");
            return Ok(realm);
        }

        info!("Creating realm master");
        self.create_realm("master".to_string()).await
    }

    async fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        self.realm_repository
            .update_realm_setting(realm_id, algorithm)
            .await
    }
}
