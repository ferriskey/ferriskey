pub mod realm_postgres_repository;

use crate::{
    domain::common::entities::app_errors::CoreError,
    infrastructure::realm::repositories::realm_postgres_repository::PostgresRealmRepository,
};

use uuid::Uuid;

use crate::domain::realm::{
    entities::{Realm, RealmSetting},
    ports::RealmRepository,
};

#[cfg(test)]
use crate::domain::realm::ports::test::MockRealmRepository;

#[derive(Clone)]
pub enum RealmRepoAny {
    Postgres(PostgresRealmRepository),
    #[cfg(test)]
    Mock(MockRealmRepository),
}

impl RealmRepository for RealmRepoAny {
    async fn fetch_realm(&self) -> Result<Vec<Realm>, CoreError> {
        match self {
            Self::Postgres(r) => r.fetch_realm().await,
            #[cfg(test)]
            Self::Mock(m) => m.fetch_realm().await,
        }
    }

    async fn get_by_name(&self, name: String) -> Result<Option<Realm>, CoreError> {
        match self {
            Self::Postgres(r) => r.get_by_name(name).await,
            #[cfg(test)]
            Self::Mock(m) => m.get_by_name(name).await,
        }
    }

    async fn create_realm(&self, name: String) -> Result<Realm, CoreError> {
        match self {
            Self::Postgres(r) => r.create_realm(name).await,
            #[cfg(test)]
            Self::Mock(m) => m.create_realm(name).await,
        }
    }

    async fn update_realm(&self, realm_name: String, name: String) -> Result<Realm, CoreError> {
        match self {
            Self::Postgres(r) => r.update_realm(realm_name, name).await,
            #[cfg(test)]
            Self::Mock(m) => m.update_realm(realm_name, name).await,
        }
    }

    async fn delete_by_name(&self, name: String) -> Result<(), CoreError> {
        match self {
            Self::Postgres(r) => r.delete_by_name(name).await,
            #[cfg(test)]
            Self::Mock(m) => m.delete_by_name(name).await,
        }
    }

    async fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, CoreError> {
        match self {
            Self::Postgres(r) => r.create_realm_settings(realm_id, algorithm).await,
            #[cfg(test)]
            Self::Mock(m) => m.create_realm_settings(realm_id, algorithm).await,
        }
    }

    async fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, CoreError> {
        match self {
            Self::Postgres(r) => r.update_realm_setting(realm_id, algorithm).await,
            #[cfg(test)]
            Self::Mock(m) => m.update_realm_setting(realm_id, algorithm).await,
        }
    }

    async fn get_realm_settings(&self, realm_id: Uuid) -> Result<RealmSetting, CoreError> {
        match self {
            Self::Postgres(r) => r.get_realm_settings(realm_id).await,
            #[cfg(test)]
            Self::Mock(m) => m.get_realm_settings(realm_id).await,
        }
    }
}
