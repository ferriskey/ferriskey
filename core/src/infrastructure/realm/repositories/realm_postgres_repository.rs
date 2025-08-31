use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::entity::realms::{ActiveModel, Entity as RealmEntity};

use chrono::Utc;
use uuid::Uuid;

use crate::domain::realm::{
    entities::{Realm, RealmError, RealmSetting},
    ports::RealmRepository,
};

#[derive(Debug, Clone)]
pub struct PostgresRealmRepository {
    pub db: DatabaseConnection,
}

impl PostgresRealmRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl RealmRepository for PostgresRealmRepository {
    async fn fetch_realm(&self) -> Result<Vec<Realm>, RealmError> {
        let realms = RealmEntity::find()
            .all(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .iter()
            .map(Realm::from)
            .collect::<Vec<Realm>>();

        Ok(realms)
    }

    async fn get_by_name(&self, name: String) -> Result<Option<Realm>, RealmError> {
        let realm = RealmEntity::find()
            .filter(crate::entity::realms::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .map(Realm::from);

        Ok(realm)
    }

    async fn create_realm(&self, name: String) -> Result<Realm, RealmError> {
        let realm = Realm::new(name);

        let new_realm = ActiveModel {
            id: Set(realm.id),
            name: Set(realm.name),
            created_at: Set(realm.created_at.naive_utc()),
            updated_at: Set(realm.updated_at.naive_utc()),
        };

        let result_insert = new_realm.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert realm: {:?}", e);
            RealmError::InternalServerError
        })?;

        let realm = result_insert.into();

        Ok(realm)
    }

    async fn update_realm(&self, realm_name: String, name: String) -> Result<Realm, RealmError> {
        let realm = RealmEntity::find()
            .filter(crate::entity::realms::Column::Name.eq(realm_name))
            .one(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .ok_or(RealmError::NotFound)?;

        let mut realm: ActiveModel = realm.into();
        realm.name = Set(name.clone());
        realm.updated_at = Set(Utc::now().naive_utc());
        realm
            .update(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        let updated_realm = RealmEntity::find()
            .filter(crate::entity::realms::Column::Name.eq(name))
            .one(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .map(Realm::from);
        let updated_realm = updated_realm.ok_or(RealmError::InternalServerError)?;
        Ok(updated_realm)
    }

    async fn delete_by_name(&self, name: String) -> Result<(), RealmError> {
        let res = RealmEntity::delete_many()
            .filter(crate::entity::realms::Column::Name.eq(name))
            .exec(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?;

        if res.rows_affected == 0 {
            return Err(RealmError::InternalServerError);
        }

        Ok(())
    }

    async fn create_realm_settings(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        let realm_setting = RealmSetting::new(realm_id, Some(algorithm));

        let active_model = crate::entity::realm_settings::ActiveModel {
            id: Set(realm_setting.id),
            realm_id: Set(realm_setting.realm_id),
            default_signing_algorithm: Set(realm_setting.default_signing_algorithm),
            updated_at: Set(realm_setting.updated_at.naive_utc()),
        };

        let model: RealmSetting = active_model
            .insert(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to insert realm setting: {:?}", e);
                RealmError::InternalServerError
            })?
            .into();

        Ok(model)
    }

    async fn update_realm_setting(
        &self,
        realm_id: Uuid,
        algorithm: String,
    ) -> Result<RealmSetting, RealmError> {
        let realm_setting = crate::entity::realm_settings::Entity::find()
            .filter(crate::entity::realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .ok_or(RealmError::NotFound)?;

        let mut realm_setting: crate::entity::realm_settings::ActiveModel = realm_setting.into();

        realm_setting.default_signing_algorithm = Set(Some(algorithm));

        let realm_setting = realm_setting
            .update(&self.db)
            .await
            .map_err(|_| RealmError::InternalServerError)?
            .into();

        Ok(realm_setting)
    }
}
