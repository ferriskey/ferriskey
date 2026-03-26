use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use tracing::error;
use uuid::Uuid;

use ferriskey_domain::realm::RealmId;
use ferriskey_organization::{
    CreateOrganizationParams, Organization, OrganizationId, OrganizationRepository,
    UpdateOrganizationParams,
};

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_timestamp;
use crate::entity::organizations::{
    ActiveModel as OrganizationActiveModel, Column as OrganizationColumn,
    Entity as OrganizationEntity, Model as OrganizationModel,
};

#[derive(Debug, Clone)]
pub struct PostgresOrganizationRepository {
    pub db: DatabaseConnection,
}

impl PostgresOrganizationRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

fn model_to_domain(model: OrganizationModel) -> Organization {
    Organization {
        id: OrganizationId::new(model.id),
        realm_id: RealmId::new(model.realm_id),
        name: model.name,
        alias: model.alias,
        domain: model.domain,
        redirect_url: model.redirect_url,
        description: model.description,
        enabled: model.enabled,
        created_at: model.created_at.with_timezone(&Utc),
        updated_at: model.updated_at.with_timezone(&Utc),
    }
}

impl OrganizationRepository for PostgresOrganizationRepository {
    async fn create_organization(
        &self,
        params: CreateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        let (_, timestamp) = generate_timestamp();
        let id = Uuid::new_v7(timestamp);
        let now = Utc::now().fixed_offset();

        let model = OrganizationEntity::insert(OrganizationActiveModel {
            id: Set(id),
            realm_id: Set(params.realm_id.into()),
            name: Set(params.name),
            alias: Set(params.alias),
            domain: Set(params.domain),
            redirect_url: Set(params.redirect_url),
            description: Set(params.description),
            enabled: Set(params.enabled),
            created_at: Set(now),
            updated_at: Set(now),
        })
        .exec_with_returning(&self.db)
        .await
        .map_err(|e| {
            error!("Failed to create organization: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model_to_domain(model))
    }

    async fn get_organization_by_id(
        &self,
        id: OrganizationId,
    ) -> Result<Option<Organization>, CoreError> {
        let model = OrganizationEntity::find_by_id(id.as_uuid())
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get organization by id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain))
    }

    async fn get_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<Option<Organization>, CoreError> {
        let model = OrganizationEntity::find()
            .filter(OrganizationColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(OrganizationColumn::Alias.eq(alias))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to get organization by realm and alias: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(model_to_domain))
    }

    async fn list_organizations_by_realm(
        &self,
        realm_id: RealmId,
    ) -> Result<Vec<Organization>, CoreError> {
        let models = OrganizationEntity::find()
            .filter(OrganizationColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to list organizations by realm: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(model_to_domain).collect())
    }

    async fn update_organization(
        &self,
        id: OrganizationId,
        params: UpdateOrganizationParams,
    ) -> Result<Organization, CoreError> {
        let now = Utc::now().fixed_offset();

        let mut active_model = OrganizationActiveModel {
            id: Set(id.as_uuid()),
            updated_at: Set(now),
            ..Default::default()
        };

        if let Some(name) = params.name {
            active_model.name = Set(name);
        }
        if let Some(alias) = params.alias {
            active_model.alias = Set(alias);
        }
        if params.domain.is_some() {
            active_model.domain = Set(params.domain);
        }
        if params.redirect_url.is_some() {
            active_model.redirect_url = Set(params.redirect_url);
        }
        if params.description.is_some() {
            active_model.description = Set(params.description);
        }
        if let Some(enabled) = params.enabled {
            active_model.enabled = Set(enabled);
        }

        let model = OrganizationEntity::update(active_model)
            .filter(OrganizationColumn::Id.eq(id.as_uuid()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to update organization: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model_to_domain(model))
    }

    async fn delete_organization(&self, id: OrganizationId) -> Result<(), CoreError> {
        OrganizationEntity::delete_by_id(id.as_uuid())
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to delete organization: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn exists_organization_by_realm_and_alias(
        &self,
        realm_id: RealmId,
        alias: &str,
    ) -> Result<bool, CoreError> {
        let count: u64 = OrganizationEntity::find()
            .filter(OrganizationColumn::RealmId.eq::<Uuid>(realm_id.into()))
            .filter(OrganizationColumn::Alias.eq(alias))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("Failed to check organization alias existence: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}
