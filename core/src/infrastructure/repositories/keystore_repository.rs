use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    common::generate_uuid_v7,
    jwt::{
        entities::{JwtError, JwtKeyPair},
        ports::KeyStoreRepository,
    },
};

impl TryFrom<crate::entity::jwt_keys::Model> for JwtKeyPair {
    type Error = JwtError;

    fn try_from(value: crate::entity::jwt_keys::Model) -> Result<Self, Self::Error> {
        let jwt_key_pair = JwtKeyPair::from_pem(
            &value.private_key,
            &value.public_key,
            value.realm_id,
            value.id,
        )?;

        Ok(jwt_key_pair)
    }
}

#[derive(Debug, Clone)]
pub struct PostgresKeyStoreRepository {
    pub db: DatabaseConnection,
}

impl PostgresKeyStoreRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl KeyStoreRepository for PostgresKeyStoreRepository {
    async fn get_or_generate_key(&self, realm_id: Uuid) -> Result<JwtKeyPair, JwtError> {
        let key = crate::entity::jwt_keys::Entity::find()
            .filter(crate::entity::jwt_keys::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| JwtError::RealmKeyNotFound)?;

        if let Some(key) = key {
            return key.try_into();
        }

        let id = generate_uuid_v7();

        // Generate a new key pair
        let (private_key, public_key) = JwtKeyPair::generate()?;

        let new_key = crate::entity::jwt_keys::ActiveModel {
            id: Set(id),
            realm_id: Set(realm_id),
            public_key: Set(public_key),
            private_key: Set(private_key),
            created_at: Set(chrono::Utc::now().naive_utc()),
        };

        // Insert the new key into the database
        let result = new_key
            .insert(&self.db)
            .await
            .map_err(|e| JwtError::GenerationError(e.to_string()))?;

        result.try_into()
    }
}
