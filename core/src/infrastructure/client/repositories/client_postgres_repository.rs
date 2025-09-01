use crate::entity::clients::{ActiveModel, Entity as ClientEntity};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::{
    client::{
        entities::{Client, ClientError, redirect_uri::RedirectUri},
        ports::ClientRepository,
        value_objects::{CreateClientRequest, UpdateClientRequest},
    },
    common::{generate_timestamp, generate_uuid_v7},
};

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
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, ClientError> {
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
            direct_access_grants_enabled: Set(Some(data.direct_access_grants_enabled)),
            client_type: Set(data.client_type),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_local()),
        };

        let client = payload.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert client: {}", e);
            ClientError::InternalServerError
        })?;

        let client = client.into();

        Ok(client)
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: uuid::Uuid,
    ) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(crate::entity::clients::Column::ClientId.eq(client_id))
            .filter(crate::entity::clients::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?
            .map(Client::from)
            .ok_or(ClientError::NotFound)?;

        Ok(client)
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Client, ClientError> {
        let clients_model = ClientEntity::find()
            .filter(crate::entity::clients::Column::Id.eq(id))
            .find_with_related(crate::entity::redirect_uris::Entity)
            .all(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        if clients_model.is_empty() {
            return Err(ClientError::NotFound);
        }

        let (client_model, uri_models) = &clients_model[0];

        let mut client: Client = client_model.clone().into();

        let redirect_uris: Vec<RedirectUri> = uri_models
            .iter()
            .map(|uri_model| uri_model.clone().into())
            .collect();

        client.redirect_uris = Some(redirect_uris);

        Ok(client)
    }

    async fn get_by_realm_id(&self, realm_id: uuid::Uuid) -> Result<Vec<Client>, ClientError> {
        let clients = ClientEntity::find()
            .filter(crate::entity::clients::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|_| ClientError::InternalServerError)?;

        let clients: Vec<Client> = clients.into_iter().map(|c| c.into()).collect();

        Ok(clients)
    }

    async fn update_client(
        &self,
        client_id: Uuid,
        data: UpdateClientRequest,
    ) -> Result<Client, ClientError> {
        let client = ClientEntity::find()
            .filter(crate::entity::clients::Column::Id.eq(client_id))
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

        client.direct_access_grants_enabled = match data.direct_access_grants_enabled {
            Some(enabled) => Set(Some(enabled)),
            None => client.direct_access_grants_enabled,
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
            .filter(crate::entity::clients::Column::Id.eq(id))
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
