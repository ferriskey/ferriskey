use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
};
use uuid::Uuid;

use ferriskey_compass::{
    entities::{CompassFlow, FlowStatus},
    ports::CompassFlowRepository,
    value_objects::{FlowFilter, FlowStats},
};
use ferriskey_domain::realm::RealmId;

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::{compass_flow_steps, compass_flows};

#[derive(Debug, Clone)]
pub struct PostgresCompassFlowRepository {
    pub db: DatabaseConnection,
}

impl PostgresCompassFlowRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn apply_filter(
        query: sea_orm::Select<compass_flows::Entity>,
        realm_id: RealmId,
        filter: &FlowFilter,
    ) -> sea_orm::Select<compass_flows::Entity> {
        let mut query = query.filter(compass_flows::Column::RealmId.eq::<Uuid>(realm_id.into()));

        if let Some(ref client_id) = filter.client_id {
            query = query.filter(compass_flows::Column::ClientId.eq(client_id.clone()));
        }

        if let Some(user_id) = filter.user_id {
            query = query.filter(compass_flows::Column::UserId.eq(user_id));
        }

        if let Some(ref grant_type) = filter.grant_type {
            query = query.filter(compass_flows::Column::GrantType.eq(grant_type.clone()));
        }

        if let Some(ref status) = filter.status {
            query = query.filter(compass_flows::Column::Status.eq(status.clone()));
        }

        if let Some(from) = filter.from_timestamp {
            query = query.filter(compass_flows::Column::StartedAt.gte(from.naive_utc()));
        }

        if let Some(to) = filter.to_timestamp {
            query = query.filter(compass_flows::Column::StartedAt.lte(to.naive_utc()));
        }

        query
    }
}

impl CompassFlowRepository for PostgresCompassFlowRepository {
    async fn create_flow(&self, flow: CompassFlow) -> Result<(), CoreError> {
        let active_model: compass_flows::ActiveModel = flow.into();

        compass_flows::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create compass flow: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn update_flow_status(
        &self,
        flow_id: Uuid,
        status: FlowStatus,
        completed_at: DateTime<Utc>,
        duration_ms: Option<i64>,
        user_id: Option<Uuid>,
    ) -> Result<(), CoreError> {
        let model = compass_flows::Entity::find_by_id(flow_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find compass flow: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active_model: compass_flows::ActiveModel = model.into();
        active_model.status = Set(status.to_string());
        active_model.completed_at = Set(Some(completed_at.naive_utc()));
        active_model.duration_ms = Set(duration_ms);
        active_model.user_id = Set(user_id);

        compass_flows::Entity::update(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update compass flow status: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_flows(
        &self,
        realm_id: RealmId,
        filter: FlowFilter,
    ) -> Result<Vec<CompassFlow>, CoreError> {
        let query = Self::apply_filter(compass_flows::Entity::find(), realm_id, &filter)
            .order_by_desc(compass_flows::Column::StartedAt);

        let query = if let Some(limit) = filter.limit {
            query.limit(limit as u64)
        } else {
            query
        };

        let query = if let Some(offset) = filter.offset {
            query.offset(offset as u64)
        } else {
            query
        };

        let results = query
            .find_with_related(compass_flow_steps::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get compass flows: {}", e);
                CoreError::InternalServerError
            })?;

        let flows = results
            .into_iter()
            .map(|(flow_model, step_models)| {
                let mut flow: CompassFlow = flow_model.into();
                flow.steps = step_models.into_iter().map(|s| s.into()).collect();
                flow
            })
            .collect();

        Ok(flows)
    }

    async fn get_flow_by_id(&self, flow_id: Uuid) -> Result<Option<CompassFlow>, CoreError> {
        let result = compass_flows::Entity::find_by_id(flow_id)
            .find_with_related(compass_flow_steps::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get compass flow by id: {}", e);
                CoreError::InternalServerError
            })?;

        let flow = result.into_iter().next().map(|(flow_model, step_models)| {
            let mut flow: CompassFlow = flow_model.into();
            flow.steps = step_models.into_iter().map(|s| s.into()).collect();
            flow
        });

        Ok(flow)
    }

    async fn count_flows(&self, realm_id: RealmId, filter: FlowFilter) -> Result<i64, CoreError> {
        let query = Self::apply_filter(compass_flows::Entity::find(), realm_id, &filter);

        let count = query.count(&self.db).await.map_err(|e| {
            tracing::error!("Failed to count compass flows: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(count as i64)
    }

    async fn purge_old_flows(&self, older_than: DateTime<Utc>) -> Result<u64, CoreError> {
        let result = compass_flows::Entity::delete_many()
            .filter(compass_flows::Column::StartedAt.lt(older_than.naive_utc()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to purge old compass flows: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected)
    }

    async fn get_stats(&self, realm_id: RealmId) -> Result<FlowStats, CoreError> {
        let realm_uuid: Uuid = realm_id.into();

        let total = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count total flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let success_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("success"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count success flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let failure_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("failure"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count failure flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let pending_count = compass_flows::Entity::find()
            .filter(compass_flows::Column::RealmId.eq(realm_uuid))
            .filter(compass_flows::Column::Status.eq("pending"))
            .count(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to count pending flows: {}", e);
                CoreError::InternalServerError
            })? as i64;

        let avg_duration_ms = self
            .db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "SELECT AVG(duration_ms)::float8 as avg_duration FROM compass_flows WHERE realm_id = $1 AND duration_ms IS NOT NULL",
                [realm_uuid.into()],
            ))
            .await
            .map_err(|e| {
                tracing::error!("Failed to get avg duration: {}", e);
                CoreError::InternalServerError
            })?
            .and_then(|row| {
                use sea_orm::TryGetable;
                Option::<f64>::try_get_by(&row, "avg_duration").ok().flatten()
            });

        Ok(FlowStats {
            total,
            success_count,
            failure_count,
            pending_count,
            avg_duration_ms,
        })
    }
}
