use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::seawatch::{
    entities::SecurityEvent, ports::SecurityEventRepository, value_objects::SecurityEventFilter,
};
use crate::entity::security_events;

#[derive(Debug, Clone)]
pub struct PostgresSecurityEventRepository {
    pub db: DatabaseConnection,
}

impl PostgresSecurityEventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl SecurityEventRepository for PostgresSecurityEventRepository {
    async fn store_event(&self, event: SecurityEvent) -> Result<(), CoreError> {
        let active_model: security_events::ActiveModel = event.into();

        security_events::Entity::insert(active_model)
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to store security event: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(())
    }

    async fn get_events(
        &self,
        realm_id: RealmId,
        filter: SecurityEventFilter,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        let mut query = security_events::Entity::find()
            .filter(security_events::Column::RealmId.eq::<Uuid>(realm_id.into()));

        if let Some(actor_id) = filter.actor_id {
            query = query.filter(security_events::Column::ActorId.eq(actor_id));
        }

        if let Some(client_id) = filter.client_id {
            query = query.filter(security_events::Column::TargetId.eq(client_id));
        }

        if let Some(event_types) = filter.event_types {
            let type_strings: Vec<String> =
                event_types.into_iter().map(|t| t.to_string()).collect();
            query = query.filter(security_events::Column::EventType.is_in(type_strings));
        }

        if let Some(from) = filter.from_timestamp {
            query = query.filter(security_events::Column::Timestamp.gte(from.naive_utc()));
        }

        if let Some(to) = filter.to_timestamp {
            query = query.filter(security_events::Column::Timestamp.lte(to.naive_utc()));
        }

        if let Some(ip) = filter.ip_address {
            query = query.filter(security_events::Column::IpAddress.eq(ip));
        }

        query = query.order_by_desc(security_events::Column::Timestamp);

        if let Some(limit) = filter.limit {
            query = query.limit(limit as u64);
        }

        if let Some(offset) = filter.offset {
            query = query.offset(offset as u64);
        }

        let models = query.all(&self.db).await.map_err(|e| {
            tracing::error!("Failed to get security events: {}", e);
            CoreError::InternalServerError
        })?;

        let events = models.into_iter().map(|model| model.into()).collect();

        Ok(events)
    }

    fn get_by_id(&self, id: Uuid) -> impl Future<Output = Result<SecurityEvent, CoreError>> + Send {
        let db = self.db.clone();
        async move {
            let model = security_events::Entity::find_by_id(id)
                .one(&db)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get security event by id: {}", e);
                    CoreError::InternalServerError
                })?
                .ok_or(CoreError::NotFound)?;

            Ok(model.into())
        }
    }

    fn count_events(
        &self,
        realm_id: Uuid,
        filter: SecurityEventFilter,
    ) -> impl Future<Output = Result<i64, CoreError>> + Send {
        let db = self.db.clone();
        async move {
            let mut query = security_events::Entity::find()
                .filter(security_events::Column::RealmId.eq(realm_id));

            if let Some(actor_id) = filter.actor_id {
                query = query.filter(security_events::Column::ActorId.eq(actor_id));
            }

            if let Some(client_id) = filter.client_id {
                query = query.filter(security_events::Column::TargetId.eq(client_id));
            }

            if let Some(event_types) = filter.event_types {
                let type_strings: Vec<String> =
                    event_types.into_iter().map(|t| t.to_string()).collect();
                query = query.filter(security_events::Column::EventType.is_in(type_strings));
            }

            if let Some(from) = filter.from_timestamp {
                query = query.filter(security_events::Column::Timestamp.gte(from.naive_utc()));
            }

            if let Some(to) = filter.to_timestamp {
                query = query.filter(security_events::Column::Timestamp.lte(to.naive_utc()));
            }

            if let Some(ip) = filter.ip_address {
                query = query.filter(security_events::Column::IpAddress.eq(ip));
            }

            let count = query.count(&db).await.map_err(|e| {
                tracing::error!("Failed to count security events: {}", e);
                CoreError::InternalServerError
            })?;

            Ok(count as i64)
        }
    }
}
