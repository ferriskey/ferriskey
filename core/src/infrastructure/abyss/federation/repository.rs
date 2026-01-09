use chrono::Utc;
use sea_orm::*;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{
    FederationMapping, FederationProvider, FederationType, SyncMode,
};
use crate::domain::abyss::federation::ports::FederationRepository;
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, UpdateProviderRequest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{user_federation_mappings, user_federation_providers};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct FederationRepositoryImpl {
    db: DatabaseConnection,
}

#[derive(Deserialize)]
struct SyncSettings {
    enabled: bool,
    mode: SyncMode,
    interval_minutes: Option<i32>,
}

impl FederationRepositoryImpl {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

// Transformation Logic
impl TryFrom<user_federation_providers::Model> for FederationProvider {
    type Error = CoreError;

    fn try_from(model: user_federation_providers::Model) -> Result<Self, Self::Error> {
        let provider_type = match model.provider_type.as_str() {
            "Ldap" => FederationType::Ldap,
            "Kerberos" => FederationType::Kerberos,
            "ActiveDirectory" => FederationType::ActiveDirectory,
            s => FederationType::Custom(s.to_string()),
        };

        Ok(FederationProvider {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name,
            provider_type,
            enabled: model.enabled,
            priority: model.priority,
            config: model.config,
            sync_settings: serde_json::json!({
                "enabled": model.sync_enabled,
                "mode": model.sync_mode,
                "interval_minutes": model.sync_interval_minutes,
                "last_sync_at": model.last_sync_at,
                "last_sync_status": model.last_sync_status,
                "last_sync_result": model.last_sync_result
            }),
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        })
    }
}

impl TryFrom<CreateProviderRequest> for user_federation_providers::ActiveModel {
    type Error = CoreError;

    fn try_from(request: CreateProviderRequest) -> Result<Self, Self::Error> {
        let id = Uuid::now_v7();
        let now = Utc::now();

        let sync_settings: SyncSettings = serde_json::from_value(request.sync_settings)
            .map_err(|e| CoreError::Configuration(format!("Invalid sync settings: {}", e)))?;

        Ok(user_federation_providers::ActiveModel {
            id: Set(id),
            realm_id: Set(request.realm_id),
            name: Set(request.name),
            provider_type: Set(request.provider_type.to_string()),
            enabled: Set(request.enabled),
            priority: Set(request.priority),
            config: Set(request.config),
            sync_enabled: Set(sync_settings.enabled),
            sync_mode: Set(sync_settings.mode.to_string()),
            sync_interval_minutes: Set(sync_settings.interval_minutes),
            last_sync_at: Set(None),
            last_sync_status: Set(None),
            last_sync_result: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        })
    }
}

impl TryFrom<user_federation_mappings::Model> for FederationMapping {
    type Error = CoreError;

    fn try_from(model: user_federation_mappings::Model) -> Result<Self, Self::Error> {
        Ok(FederationMapping {
            id: model.id,
            provider_id: model.provider_id,
            user_id: model.user_id,
            external_id: model.external_id,
            external_username: model.external_username,
            mapping_metadata: model.mapping_metadata.unwrap_or(serde_json::Value::Null),
            last_synced_at: model.last_synced_at.into(),
        })
    }
}

impl TryFrom<FederationMapping> for user_federation_mappings::ActiveModel {
    type Error = CoreError;

    fn try_from(mapping: FederationMapping) -> Result<Self, Self::Error> {
        Ok(user_federation_mappings::ActiveModel {
            id: Set(mapping.id),
            provider_id: Set(mapping.provider_id),
            user_id: Set(mapping.user_id),
            external_id: Set(mapping.external_id),
            external_username: Set(mapping.external_username),
            mapping_metadata: Set(Some(mapping.mapping_metadata)),
            last_synced_at: Set(mapping.last_synced_at.into()),
        })
    }
}

impl FederationRepository for FederationRepositoryImpl {
    async fn create(
        &self,
        request: CreateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let active_model: user_federation_providers::ActiveModel = request.try_into()?;

        let model = active_model.insert(&self.db).await.map_err(|e| {
            CoreError::Database(format!("Failed to create federation provider: {}", e))
        })?;

        model.try_into()
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<FederationProvider>, CoreError> {
        let model = user_federation_providers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to get federation provider: {}", e))
            })?;

        match model {
            Some(m) => Ok(Some(m.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        id: Uuid,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let model = user_federation_providers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to find federation provider: {}", e)))?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: user_federation_providers::ActiveModel = model.into();
        let now = Utc::now();

        if let Some(name) = request.name {
            active_model.name = Set(name);
        }
        if let Some(provider_type) = request.provider_type {
            active_model.provider_type = Set(provider_type.to_string());
        }
        if let Some(enabled) = request.enabled {
            active_model.enabled = Set(enabled);
        }
        if let Some(priority) = request.priority {
            active_model.priority = Set(priority);
        }
        if let Some(config) = request.config {
            active_model.config = Set(config);
        }

        if let Some(sync_settings_json) = request.sync_settings {
            let sync_settings: SyncSettings = serde_json::from_value(sync_settings_json)
                .map_err(|e| CoreError::Configuration(format!("Invalid sync settings: {}", e)))?;

            active_model.sync_enabled = Set(sync_settings.enabled);
            active_model.sync_mode = Set(sync_settings.mode.to_string());
            active_model.sync_interval_minutes = Set(sync_settings.interval_minutes);
        }

        active_model.updated_at = Set(now.into());

        let updated_model = active_model.update(&self.db).await.map_err(|e| {
            CoreError::Database(format!("Failed to update federation provider: {}", e))
        })?;

        updated_model.try_into()
    }

    async fn delete(&self, id: Uuid) -> Result<(), CoreError> {
        user_federation_providers::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete federation provider: {}", e);
                CoreError::Database(format!("Failed to delete federation provider: {}", e))
            })?;
        Ok(())
    }

    async fn list_by_realm(&self, realm_id: Uuid) -> Result<Vec<FederationProvider>, CoreError> {
        let models = user_federation_providers::Entity::find()
            .filter(user_federation_providers::Column::RealmId.eq(realm_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to list federation providers: {}", e))
            })?;

        models.into_iter().map(|m| m.try_into()).collect()
    }

    async fn create_mapping(
        &self,
        mapping: FederationMapping,
    ) -> Result<FederationMapping, CoreError> {
        let active_model: user_federation_mappings::ActiveModel = mapping.try_into()?;

        let model = active_model.insert(&self.db).await.map_err(|e| {
            CoreError::Database(format!("Failed to create federation mapping: {}", e))
        })?;

        model.try_into()
    }

    async fn get_mapping(
        &self,
        provider_id: Uuid,
        external_id: &str,
    ) -> Result<Option<FederationMapping>, CoreError> {
        let model = user_federation_mappings::Entity::find()
            .filter(
                user_federation_mappings::Column::ProviderId
                    .eq(provider_id)
                    .and(user_federation_mappings::Column::ExternalId.eq(external_id)),
            )
            .one(&self.db)
            .await
            .map_err(|e| CoreError::Database(format!("Failed to get federation mapping: {}", e)))?;

        match model {
            Some(m) => Ok(Some(m.try_into()?)),
            None => Ok(None),
        }
    }

    async fn list_mappings_by_provider(
        &self,
        provider_id: Uuid,
    ) -> Result<Vec<FederationMapping>, CoreError> {
        let models = user_federation_mappings::Entity::find()
            .filter(user_federation_mappings::Column::ProviderId.eq(provider_id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to list federation mappings: {}", e))
            })?;

        models.into_iter().map(|m| m.try_into()).collect()
    }

    async fn update_mapping(
        &self,
        mapping: FederationMapping,
    ) -> Result<FederationMapping, CoreError> {
        let model = user_federation_mappings::Entity::find_by_id(mapping.id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!(
                    "Failed to find federation mapping for update: {}",
                    e
                ))
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: user_federation_mappings::ActiveModel = model.into();

        // Update fields
        active_model.mapping_metadata = Set(Some(mapping.mapping_metadata));
        active_model.last_synced_at = Set(mapping.last_synced_at.into());
        active_model.external_username = Set(mapping.external_username);

        let updated_model = active_model.update(&self.db).await.map_err(|e| {
            CoreError::Database(format!("Failed to update federation mapping: {}", e))
        })?;

        updated_model.try_into()
    }

    async fn delete_mapping(&self, id: Uuid) -> Result<(), CoreError> {
        user_federation_mappings::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to delete federation mapping: {}", e))
            })?;
        Ok(())
    }
}
