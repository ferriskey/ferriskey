use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::domain::client::entities::redirect_uri::RedirectUri;
use crate::domain::client::ports::PostLogoutRedirectUriRepository;
use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone)]
pub struct PostgresPostLogoutRedirectUriRepository {
    pub db: DatabaseConnection,
}

impl PostgresPostLogoutRedirectUriRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl PostLogoutRedirectUriRepository for PostgresPostLogoutRedirectUriRepository {
    async fn create_redirect_uri(
        &self,
        client_id: Uuid,
        value: String,
        enabled: bool,
    ) -> Result<RedirectUri, CoreError> {
        let redirect_uri = RedirectUri::new(client_id, value, enabled);

        let payload = crate::entity::post_logout_redirect_uris::ActiveModel {
            id: Set(redirect_uri.id),
            client_id: Set(redirect_uri.client_id),
            value: Set(redirect_uri.value),
            enabled: Set(redirect_uri.enabled),
            created_at: Set(redirect_uri.created_at.naive_utc()),
            updated_at: Set(redirect_uri.updated_at.naive_utc()),
        };

        let model = payload
            .insert(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(model.into())
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<RedirectUri>, CoreError> {
        let redirect_uris = crate::entity::post_logout_redirect_uris::Entity::find()
            .filter(crate::entity::post_logout_redirect_uris::Column::ClientId.eq(client_id))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn get_enabled_by_client_id(
        &self,
        client_id: Uuid,
    ) -> Result<Vec<RedirectUri>, CoreError> {
        let redirect_uris = crate::entity::post_logout_redirect_uris::Entity::find()
            .filter(crate::entity::post_logout_redirect_uris::Column::ClientId.eq(client_id))
            .filter(crate::entity::post_logout_redirect_uris::Column::Enabled.eq(true))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?
            .into_iter()
            .map(RedirectUri::from)
            .collect::<Vec<RedirectUri>>();

        Ok(redirect_uris)
    }

    async fn update_enabled(&self, id: Uuid, enabled: bool) -> Result<RedirectUri, CoreError> {
        let redirect_uri = crate::entity::post_logout_redirect_uris::Entity::find()
            .filter(crate::entity::post_logout_redirect_uris::Column::Id.eq(id))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::RedirectUriNotFound)?;

        if let Some(redirect_uri) = redirect_uri {
            let mut redirect_uri: crate::entity::post_logout_redirect_uris::ActiveModel =
                redirect_uri.into();
            redirect_uri.enabled = Set(enabled);
            redirect_uri.updated_at = Set(Utc::now().naive_utc());

            let redirect_uri = redirect_uri
                .update(&self.db)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            Ok(redirect_uri.into())
        } else {
            Err(CoreError::RedirectUriNotFound)
        }
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        crate::entity::post_logout_redirect_uris::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }
}

impl From<crate::entity::post_logout_redirect_uris::Model> for RedirectUri {
    fn from(model: crate::entity::post_logout_redirect_uris::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        RedirectUri {
            id: model.id,
            client_id: model.client_id,
            value: model.value,
            enabled: model.enabled,
            created_at,
            updated_at,
        }
    }
}
