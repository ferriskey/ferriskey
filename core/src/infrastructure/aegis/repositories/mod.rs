use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use tracing::error;
use uuid::Uuid;

use crate::domain::aegis::entities::ProtocolMapper;
use crate::domain::common::entities::app_errors::CoreError;
use crate::entity::client_scope_protocol_mappers;

pub mod client_scope_attribute_postgres_repository;
pub mod client_scope_postgres_repository;
pub mod protocol_mapper_postgres_repository;
pub mod scope_mapping_postgres_repository;

async fn protocol_mappers_by_scope(
    db: &DatabaseConnection,
    scope_ids: &[Uuid],
) -> Result<HashMap<Uuid, Vec<ProtocolMapper>>, CoreError> {
    let mut mappers: HashMap<Uuid, Vec<ProtocolMapper>> = HashMap::new();
    if scope_ids.is_empty() {
        return Ok(mappers);
    }

    let rows = client_scope_protocol_mappers::Entity::find()
        .filter(client_scope_protocol_mappers::Column::ClientScopeId.is_in(scope_ids.to_vec()))
        .order_by_asc(client_scope_protocol_mappers::Column::CreatedAt)
        .order_by_asc(client_scope_protocol_mappers::Column::Id)
        .all(db)
        .await
        .map_err(|e| {
            error!("error loading protocol mappers of client scopes: {:?}", e);
            CoreError::InternalServerError
        })?;
    for row in rows {
        mappers
            .entry(row.client_scope_id)
            .or_default()
            .push(ProtocolMapper::from(row));
    }

    Ok(mappers)
}
