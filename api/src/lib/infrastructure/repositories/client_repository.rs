use entity::clients::{ActiveModel, Entity as ClientEntity};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, QueryFilter};
use sqlx::{Executor, PgPool};

use crate::domain::{
    client::{
        entities::{dto::CreateClientDto, error::ClientError, model::Client},
        ports::client_repository::ClientRepository,
    },
    utils::{generate_timestamp, generate_uuid_v7},
};

impl From<entity::clients::Model> for Client {
    fn from(model: entity::clients::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        Client {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name,
            client_id: model.client_id,
            secret: model.secret,
            enabled: model.enabled,
            protocol: model.protocol,
            public_client: model.public_client,
            service_account_enabled: model.service_account_enabled,
            client_type: model.client_type,
            created_at,
            updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostgresClientRepository {
    pub pool: PgPool,
    pub db: DatabaseConnection,
}

impl PostgresClientRepository {
    pub fn new(pool: PgPool, db: DatabaseConnection) -> Self {
        Self { pool, db }
    }
}

impl ClientRepository for PostgresClientRepository {
    async fn create_client(&self, data: CreateClientDto) -> Result<Client, ClientError> {
        let (now, _) = generate_timestamp();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(data.realm_id),
            name: Set(data.name),
            client_id: Set(data.client_id),
            secret: Set(data.secret),
            enabled: Set(data.enabled),
            protocol: Set(data.protocol),
            public_client: Set(data.public_client),
            service_account_enabled: Set(data.service_account_enabled),
            client_type: Set(data.client_type),
            created_at: Set(now),
            updated_at: Set(now),
        };

        ClientEntity::insert(payload)
            .exec(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)
            .map(Client::from)
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: uuid::Uuid,
    ) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(entity::clients::Column::ClientId.eq(client_id))
            .filter(entity::clients::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .map(Client::from);

        Ok(client)
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(entity::clients::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .map(Client::from);

        Ok(client)
    }
}
