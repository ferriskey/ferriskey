pub mod realm_postgres_repository;

use crate::infrastructure::realm::repositories::realm_postgres_repository::PostgresRealmRepository;

use uuid::Uuid;

use crate::domain::realm::{
    entities::{Realm, RealmError, RealmSetting},
    ports::RealmRepository,
};

#[derive(Clone)]
pub enum RealmRepoAny {
    Postgres(PostgresRealmRepository),
}

impl RealmRepository for RealmRepoAny {
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
        match self {
            Self::Postgres(r) => r.fetch_realm().await,
        }
    }

    async fn get_by_name(&self, name: String) -> Result<Option<Realm>, RealmError> {
        match self {
            Self::Postgres(r) => r.get_by_name(name).await,
        }
    }

    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        match self {
            Self::Postgres(r) => r.create_realm(name).await,
        }
    }

    async fn update_realm(&self, realm_name: String, name: String) -> Result<Realm, RealmError> {
        match self {
            Self::Postgres(r) => r.update_realm(realm_name, name).await,
        }
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        match self {
            Self::Postgres(r) => r.delete_by_name(name).await,
        }
    }

    async fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        match self {
            Self::Postgres(r) => r.create_realm_settings(realm_id, algorithm).await,
        }
    }

    async fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        match self {
            Self::Postgres(r) => r.update_realm_setting(realm_id, algorithm).await,
        }
    }

    async fn get_realm_settings(&self, realm_id: Uuid) -> Result<RealmSetting, RealmError> {
        match self {
            Self::Postgres(r) => r.get_realm_settings(realm_id).await,
        }
    }
}
