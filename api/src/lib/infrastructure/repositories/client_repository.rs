use crate::domain::{
    client::{
        entities::{
            dto::{CreateClientDto, UpdateClientDto},
            error::ClientError,
            model::Client,
        },
        ports::client_repository::ClientRepository,
    },
    utils::{generate_timestamp, generate_uuid_v7},
};
use chrono::{TimeZone, Utc};
use entity::clients::{ActiveModel, Entity as ClientEntity};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

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
    pub db: DatabaseConnection,
}

impl PostgresClientRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
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
            secret: Set(Some(data.secret)),
            enabled: Set(data.enabled),
            protocol: Set(data.protocol),
            public_client: Set(data.public_client),
            service_account_enabled: Set(data.service_account_enabled),
            client_type: Set(data.client_type),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_local()),
        };

        let client = payload
            .insert(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let client = client.into();

        Ok(client)
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
            .map(Client::from)
            .ok_or(ClientError::NotFound)?;

        Ok(client)
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(entity::clients::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .map(Client::from)
            .ok_or(ClientError::NotFound)?;

        Ok(client)
    }

    async fn get_by_realm_id(&self, realm_id: uuid::Uuid) -> Result<Vec<Client>, ClientError> {
        let clients = ClientEntity::find()
            .filter(entity::clients::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let clients: Vec<Client> = clients.into_iter().map(|c| c.into()).collect();

        Ok(clients)
    }

    async fn update_client(
        &self,
        client_id: Uuid,
        data: UpdateClientDto,
    ) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(entity::clients::Column::Id.eq(client_id))
            .one(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .ok_or(ClientError::NotFound)?;

        let mut client: ActiveModel = client.into();
        client.name = match data.name {
            Some(name) => Set(name),
            None => client.name,
        };

        client.client_id = match data.client_id {
            Some(client_id) => Set(client_id),
            None => client.client_id,
        };

        client.enabled = match data.enabled {
            Some(enabled) => Set(enabled),
            None => client.enabled,
        };

        client.updated_at = Set(Utc::now().naive_utc());

        let client = client
            .update(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        Ok(client.into())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), ClientError> {
        let result = ClientEntity::delete_many()
            .filter(entity::clients::Column::Id.eq(id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete client: {}", e);
                ClientError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(ClientError::InternalServerError);
        }

        Ok(())
    }
}
