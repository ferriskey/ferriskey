use chrono::Utc;
use sea_orm::*;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use crate::domain::abyss::federation::entities::{
    FederationMapping, FederationProvider, FederationProviderFilter, FederationProviderSortField,
    FederationType, SyncMode,
};
use crate::domain::abyss::federation::ports::FederationRepository;
use crate::domain::abyss::federation::value_objects::{
    CreateProviderRequest, UpdateProviderRequest,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{RealmScope, Scoped, Unscoped};
use crate::entity::{user_federation_mappings, user_federation_providers};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within};

impl SortColumn<user_federation_providers::Entity> for FederationProviderSortField {
    fn column(&self) -> user_federation_providers::Column {
        match self {
            FederationProviderSortField::Name => user_federation_providers::Column::Name,
            FederationProviderSortField::Priority => user_federation_providers::Column::Priority,
            FederationProviderSortField::Enabled => user_federation_providers::Column::Enabled,
            FederationProviderSortField::LastSyncAt => {
                user_federation_providers::Column::LastSyncAt
            }
            FederationProviderSortField::CreatedAt => user_federation_providers::Column::CreatedAt,
            FederationProviderSortField::UpdatedAt => user_federation_providers::Column::UpdatedAt,
        }
    }
}

fn listing_select(
    realm_id: Uuid,
    filter: &FederationProviderFilter,
) -> Select<user_federation_providers::Entity> {
    use user_federation_providers::Column;

    user_federation_providers::Entity::find()
        .filter(Column::RealmId.eq(realm_id))
        .filter(within(Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(contains(Column::Name, value))
        })
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(Column::Name, value))
        })
        .apply_if(filter.provider_type.as_deref(), |select, value| {
            select.filter(Column::ProviderType.eq(value))
        })
        .apply_if(filter.provider_family, |select, family| {
            select.filter(
                Column::ProviderType.is_in(
                    family
                        .provider_types()
                        .iter()
                        .map(FederationType::to_string),
                ),
            )
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(Column::Enabled.eq(value))
        })
        .apply_if(filter.sync_enabled, |select, value| {
            select.filter(Column::SyncEnabled.eq(value))
        })
        .apply_if(filter.sync_mode, |select, value| {
            select.filter(Column::SyncMode.eq(value.to_string()))
        })
        .apply_if(filter.synced, |select, value| {
            select.filter(if value {
                Column::LastSyncAt.is_not_null()
            } else {
                Column::LastSyncAt.is_null()
            })
        })
}

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

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Unscoped<FederationProvider>>, CoreError> {
        let model = user_federation_providers::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to get federation provider: {}", e))
            })?;

        match model {
            Some(m) => Ok(Some(Unscoped::new(m.try_into()?))),
            None => Ok(None),
        }
    }

    async fn update(
        &self,
        provider: &Scoped<FederationProvider>,
        request: UpdateProviderRequest,
    ) -> Result<FederationProvider, CoreError> {
        let model = user_federation_providers::Entity::find_by_id(provider.get().id)
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

    async fn delete(&self, provider: &Scoped<FederationProvider>) -> Result<(), CoreError> {
        user_federation_providers::Entity::delete_by_id(provider.get().id)
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete federation provider: {}", e);
                CoreError::Database(format!("Failed to delete federation provider: {}", e))
            })?;
        Ok(())
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<FederationProviderFilter, FederationProviderSortField>,
    ) -> Result<Page<FederationProvider>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            CoreError::Database(format!("Failed to list federation providers: {}", e))
        })?;

        let providers = models
            .into_iter()
            .map(FederationProvider::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Page::new(providers, total, request.page, request.limit))
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
        provider: &Scoped<FederationProvider>,
        external_id: &str,
    ) -> Result<Option<FederationMapping>, CoreError> {
        let model = user_federation_mappings::Entity::find()
            .filter(
                user_federation_mappings::Column::ProviderId
                    .eq(provider.get().id)
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
        provider: &Scoped<FederationProvider>,
    ) -> Result<Vec<FederationMapping>, CoreError> {
        let models = user_federation_mappings::Entity::find()
            .filter(user_federation_mappings::Column::ProviderId.eq(provider.get().id))
            .all(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to list federation mappings: {}", e))
            })?;

        models.into_iter().map(|m| m.try_into()).collect()
    }

    async fn get_mapping_by_user_id(
        &self,
        user_id: Uuid,
    ) -> Result<Option<FederationMapping>, CoreError> {
        let model = user_federation_mappings::Entity::find()
            .filter(user_federation_mappings::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!(
                    "Failed to get federation mapping by user_id: {}",
                    e
                ))
            })?;

        match model {
            Some(m) => Ok(Some(m.try_into()?)),
            None => Ok(None),
        }
    }

    async fn update_mapping(
        &self,
        provider: &Scoped<FederationProvider>,
        mapping: FederationMapping,
    ) -> Result<FederationMapping, CoreError> {
        let model = user_federation_mappings::Entity::find()
            .filter(user_federation_mappings::Column::Id.eq(mapping.id))
            .filter(user_federation_mappings::Column::ProviderId.eq(provider.get().id))
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

    async fn delete_mapping(
        &self,
        provider: &Scoped<FederationProvider>,
        id: Uuid,
    ) -> Result<(), CoreError> {
        user_federation_mappings::Entity::delete_many()
            .filter(user_federation_mappings::Column::Id.eq(id))
            .filter(user_federation_mappings::Column::ProviderId.eq(provider.get().id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                CoreError::Database(format!("Failed to delete federation mapping: {}", e))
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::abyss::federation::entities::{
        FederationProviderFamily, FederationProviderFilter, SyncMode,
    };
    use crate::domain::common::pagination::DateRange;

    #[test]
    fn search_is_an_escaped_contains_match_on_the_name() {
        let sql = sql(&FederationProviderFilter {
            search: Some("b_".to_string()),
            ..FederationProviderFilter::default()
        });
        assert!(
            sql.contains(r#""user_federation_providers"."name" ILIKE E'%b\\_%'"#),
            "{sql}"
        );
    }

    #[test]
    fn sync_mode_is_an_exact_match_on_the_stored_value() {
        for (mode, stored) in [
            (SyncMode::Import, "Import"),
            (SyncMode::Force, "Force"),
            (SyncMode::LinkOnly, "LinkOnly"),
        ] {
            let sql = sql(&FederationProviderFilter {
                sync_mode: Some(mode),
                ..FederationProviderFilter::default()
            });
            assert!(
                sql.contains(&format!(
                    r#""user_federation_providers"."sync_mode" = '{stored}'"#
                )),
                "{sql}"
            );
        }
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&FederationProviderFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..FederationProviderFilter::default()
        });
        assert!(
            sql.contains(
                r#""user_federation_providers"."created_at" >= '2026-01-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
        assert!(
            sql.contains(
                r#""user_federation_providers"."created_at" < '2026-02-01 00:00:00.000000 +00:00'"#
            ),
            "{sql}"
        );
    }

    fn sql(filter: &FederationProviderFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&FederationProviderFilter::default());
        assert!(
            sql.contains(
                r#""user_federation_providers"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
            "{sql}"
        );
        assert!(!sql.contains("ILIKE"), "{sql}");
        assert!(!sql.contains("last_sync_at\" IS"), "{sql}");
    }

    #[test]
    fn name_is_an_escaped_contains_match() {
        let sql = sql(&FederationProviderFilter {
            name: Some("a%".to_string()),
            ..FederationProviderFilter::default()
        });
        assert!(
            sql.contains(r#""user_federation_providers"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
    }

    #[test]
    fn type_and_flags_are_exact_matches() {
        let sql = sql(&FederationProviderFilter {
            provider_type: Some("Ldap".to_string()),
            enabled: Some(false),
            sync_enabled: Some(true),
            ..FederationProviderFilter::default()
        });
        assert!(
            sql.contains(r#""user_federation_providers"."provider_type" = 'Ldap'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""user_federation_providers"."enabled" = FALSE"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""user_federation_providers"."sync_enabled" = TRUE"#),
            "{sql}"
        );
    }

    #[test]
    fn provider_family_lists_its_stored_types() {
        let ldap = sql(&FederationProviderFilter {
            provider_family: Some(FederationProviderFamily::Ldap),
            ..FederationProviderFilter::default()
        });
        assert!(
            ldap.contains(
                r#""user_federation_providers"."provider_type" IN ('Ldap', 'ActiveDirectory')"#
            ),
            "{ldap}"
        );

        let kerberos = sql(&FederationProviderFilter {
            provider_family: Some(FederationProviderFamily::Kerberos),
            ..FederationProviderFilter::default()
        });
        assert!(
            kerberos.contains(r#""user_federation_providers"."provider_type" IN ('Kerberos')"#),
            "{kerberos}"
        );
    }

    #[test]
    fn synced_tests_the_last_sync_timestamp() {
        let synced = sql(&FederationProviderFilter {
            synced: Some(true),
            ..FederationProviderFilter::default()
        });
        assert!(
            synced.contains(r#""user_federation_providers"."last_sync_at" IS NOT NULL"#),
            "{synced}"
        );

        let never = sql(&FederationProviderFilter {
            synced: Some(false),
            ..FederationProviderFilter::default()
        });
        assert!(
            never.contains(r#""user_federation_providers"."last_sync_at" IS NULL"#),
            "{never}"
        );
    }
}
