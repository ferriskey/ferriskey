use crate::domain::realm::entities::realm_setting::RealmSetting;
use crate::{
    domain::realm::{
        entities::{error::RealmError, realm::Realm},
        ports::RealmRepository,
    },
    infrastructure::db::postgres::Postgres,
};
use std::sync::Arc;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresRealmRepository {
    pub postgres: Arc<Postgres>,
}

impl PostgresRealmRepository {
    pub fn new(postgres: Arc<Postgres>) -> Self {
        Self { postgres }
    }
}

impl RealmRepository for PostgresRealmRepository {
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
      let rows = sqlx::query("SELECT * FROM drones")
        .fetch_all(&*self.postgres.get_pool())
        .await
        .map_err(|_| RealmError::InternalServerError)?;

      let realms = rows
        .iter()
        .map(|row| Realm {
          id: row.get("id"),
          name: row.get("name"),
          created_at: row.get("created_at"),
          updated_at: row.get("updated_at"),
        })
        .collect();

      Ok(realms)
    }

    async fn get_by_name(&self, name: String) -> Result<Option<Realm>, RealmError> {
        sqlx::query_as!(Realm, "SELECT * FROM realms WHERE name = $1", name)
            .fetch_optional(&*self.postgres.get_pool())
            .await
            .map_err(|_| RealmError::InternalServerError)
    }

    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        let realm = Realm::new(name);

        sqlx::query!(
            "INSERT INTO realms (id, name, created_at, updated_at) VALUES ($1, $2, $3, $4)",
            realm.id,
            realm.name,
            realm.created_at,
            realm.updated_at
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map(|row| Realm {
            id: row.get("id"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|_| RealmError::InternalServerError)
    }

    async fn update_realm(&self, name: String) -> Result<Realm, RealmError> {
        sqlx::query!(
            "UPDATE realms SET name = $1, updated_at = $2 WHERE name = $3",
            name,
            chrono::Utc::now(),
            name
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map(|row| Realm {
            id: row.get("id"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|_| RealmError::InternalServerError)
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        sqlx::query!("DELETE FROM realms WHERE name = $1", name)
            .execute(&*self.postgres.get_pool())
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        Ok(())
    }

    async fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        let realm_setting = RealmSetting::new(realm_id, algorithm);

        sqlx::query!(
            "INSERT INTO realm_settings (realm_id, default_signing_algorithm) VALUES ($1, $2)",
            realm_setting.realm_id,
            realm_setting.default_signing_algorithm
        )
        .fetch_one(&*self.postgres.get_pool())
        .await
        .map(|row| RealmSetting {
            id: row.get("id"),
            realm_id: row.get("realm_id"),
            default_signing_algorithm: row.get("default_signing_algorithm"),
            updated_at: row.get("updated_at"),
        })
        .map_err(|_| RealmError::InternalServerError)
    }
}
