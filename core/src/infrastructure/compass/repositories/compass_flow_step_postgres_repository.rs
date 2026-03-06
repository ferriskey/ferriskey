use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use ferriskey_compass::{entities::CompassFlowStep, ports::CompassFlowStepRepository};

use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::compass_flow_steps;

#[derive(Debug, Clone)]
pub struct PostgresCompassFlowStepRepository {
    pub db: DatabaseConnection,
}

impl PostgresCompassFlowStepRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl CompassFlowStepRepository for PostgresCompassFlowStepRepository {
    async fn create_step(&self, step: CompassFlowStep) -> Result<(), CoreError> {
        let active_model: compass_flow_steps::ActiveModel = step.into();

        compass_flow_steps::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create compass flow step: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_steps_for_flow(&self, flow_id: Uuid) -> Result<Vec<CompassFlowStep>, CoreError> {
        let models = compass_flow_steps::Entity::find()
            .filter(compass_flow_steps::Column::FlowId.eq(flow_id))
            .order_by_asc(compass_flow_steps::Column::StartedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get compass flow steps: {}", e);
                CoreError::InternalServerError
            })?;

        let steps = models.into_iter().map(|m| m.into()).collect();

        Ok(steps)
    }
}
