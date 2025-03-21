use super::{
    entities::{error::RealmError, model::Realm},
    ports::{RealmRepository, RealmService},
};
use tracing::{error, info};

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
    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        self.realm_repository.create_realm(name).await
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
}
