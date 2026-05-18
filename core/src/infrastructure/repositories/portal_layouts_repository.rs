use chrono::{TimeZone, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};
use tracing::error;
use uuid::Uuid;

use crate::{
    domain::{
        common::{entities::app_errors::CoreError, generate_uuid_v7},
        portal_layouts::{entities::PortalLayout, ports::PortalLayoutsRepository},
    },
    entity::portal_layouts::{ActiveModel, Column, Entity, Model},
};

#[derive(Debug, Clone)]
pub struct PostgresPortalLayoutsRepository {
    pub db: DatabaseConnection,
}

impl PostgresPortalLayoutsRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl From<Model> for PortalLayout {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            realm_id: model.realm_id.into(),
            name: model.name,
            tree: model.tree,
            is_default: model.is_default,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        }
    }
}

impl PortalLayoutsRepository for PostgresPortalLayoutsRepository {
    async fn list_by_realm(&self, realm_id: Uuid) -> Result<Vec<PortalLayout>, CoreError> {
        let models = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .order_by_asc(Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                error!("failed to list portal layouts: {e}");
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(PortalLayout::from).collect())
    }

    async fn get_by_id(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> Result<Option<PortalLayout>, CoreError> {
        let model = Entity::find_by_id(layout_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(model.map(PortalLayout::from))
    }

    async fn get_default(&self, realm_id: Uuid) -> Result<Option<PortalLayout>, CoreError> {
        let model = Entity::find()
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::IsDefault.eq(true))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch default portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(model.map(PortalLayout::from))
    }

    async fn create(
        &self,
        realm_id: Uuid,
        name: String,
        tree: serde_json::Value,
        is_default: bool,
    ) -> Result<PortalLayout, CoreError> {
        let now = Utc::now().naive_utc();
        let model = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(realm_id),
            name: Set(name),
            tree: Set(tree),
            is_default: Set(is_default),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let inserted = Entity::insert(model)
            .exec_with_returning(&self.db)
            .await
            .map_err(|e| {
                error!("failed to create portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(inserted.into())
    }

    async fn update(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
        name: String,
        tree: serde_json::Value,
    ) -> Result<PortalLayout, CoreError> {
        let existing = Entity::find_by_id(layout_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await
            .map_err(|e| {
                error!("failed to fetch portal layout for update: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: ActiveModel = existing.into();
        active.name = Set(name);
        active.tree = Set(tree);
        active.updated_at = Set(Utc::now().naive_utc());

        let updated = Entity::update(active).exec(&self.db).await.map_err(|e| {
            error!("failed to update portal layout: {e}");
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    async fn set_default(
        &self,
        realm_id: Uuid,
        layout_id: Uuid,
    ) -> Result<PortalLayout, CoreError> {
        let txn = self.db.begin().await.map_err(|e| {
            error!("failed to begin set_default transaction: {e}");
            CoreError::InternalServerError
        })?;

        // Confirm the target layout exists in this realm.
        let target = Entity::find_by_id(layout_id)
            .filter(Column::RealmId.eq(realm_id))
            .one(&txn)
            .await
            .map_err(|e| {
                error!("failed to fetch target layout: {e}");
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now().naive_utc();

        // Clear any current default in the realm (no-op if none).
        Entity::update_many()
            .col_expr(Column::IsDefault, sea_orm::sea_query::Expr::value(false))
            .col_expr(Column::UpdatedAt, sea_orm::sea_query::Expr::value(now))
            .filter(Column::RealmId.eq(realm_id))
            .filter(Column::IsDefault.eq(true))
            .exec(&txn)
            .await
            .map_err(|e| {
                error!("failed to clear existing default layout: {e}");
                CoreError::InternalServerError
            })?;

        let mut active: ActiveModel = target.into();
        active.is_default = Set(true);
        active.updated_at = Set(now);

        let updated = Entity::update(active).exec(&txn).await.map_err(|e| {
            error!("failed to mark layout as default: {e}");
            CoreError::InternalServerError
        })?;

        txn.commit().await.map_err(|e| {
            error!("failed to commit set_default transaction: {e}");
            CoreError::InternalServerError
        })?;

        Ok(updated.into())
    }

    async fn delete(&self, realm_id: Uuid, layout_id: Uuid) -> Result<(), CoreError> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(layout_id))
            .filter(Column::RealmId.eq(realm_id))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to delete portal layout: {e}");
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn is_used_by_themes(&self, realm_id: Uuid, layout_id: Uuid) -> Result<bool, CoreError> {
        use crate::entity::portal_themes;

        let count = portal_themes::Entity::find()
            .filter(portal_themes::Column::RealmId.eq(realm_id))
            .filter(portal_themes::Column::LayoutId.eq(layout_id))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("failed to count themes using portal layout: {e}");
                CoreError::InternalServerError
            })?;

        Ok(count > 0)
    }
}
