use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, TransactionTrait,
};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::seawatch::{
    entities::SecurityEvent,
    hashing::{GENESIS_PREV_HASH, compute_event_hash},
    pii::{AuditPiiMode, PiiConfig, apply_to_event},
    ports::SecurityEventRepository,
    value_objects::SecurityEventFilter,
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

    async fn load_pii_config(&self, realm_id: Uuid) -> PiiConfig {
        let result = crate::entity::realm_settings::Entity::find()
            .filter(crate::entity::realm_settings::Column::RealmId.eq(realm_id))
            .one(&self.db)
            .await;

        match result {
            Ok(Some(model)) => {
                let mode = model
                    .seawatch_pii_mode
                    .parse::<AuditPiiMode>()
                    .unwrap_or_default();
                PiiConfig {
                    mode,
                    pseudo_key: model.seawatch_pseudo_key,
                }
            }
            _ => PiiConfig::default(),
        }
    }
}

impl SecurityEventRepository for PostgresSecurityEventRepository {
    async fn store_event(&self, event: SecurityEvent) -> Result<(), CoreError> {
        let realm_id: Uuid = event.realm_id.into();
        let pii_cfg = self.load_pii_config(realm_id).await;
        let event = apply_to_event(event, &pii_cfg);

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

    /// Atomically compute and persist the chained event.
    ///
    /// The implementation opens a transaction, fetches the current chain head
    /// (latest event for the realm ordered by `created_at DESC`), computes
    /// `event_hash = SHA-256(preimage || prev_hash)`, and inserts in the same
    /// transaction to keep the chain linear even under concurrent writes from
    /// multiple API workers (Postgres serialises the writes within the txn).
    async fn store_event_chained(
        &self,
        mut event: SecurityEvent,
    ) -> Result<SecurityEvent, CoreError> {
        let db = &self.db;
        let realm_uuid: Uuid = event.realm_id.into();

        let txn = db.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction for chained event: {}", e);
            CoreError::InternalServerError
        })?;

        // Lock the latest row for this realm so concurrent inserts are serialised.
        let head = security_events::Entity::find()
            .filter(security_events::Column::RealmId.eq(realm_uuid))
            .order_by_desc(security_events::Column::CreatedAt)
            .lock_exclusive()
            .one(&txn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch chain head: {}", e);
                CoreError::InternalServerError
            })?;

        let prev_hash: [u8; 32] = head
            .and_then(|m| {
                m.event_hash
                    .and_then(|h| hex::decode(&h).ok().and_then(|b| b.try_into().ok()))
            })
            .unwrap_or(GENESIS_PREV_HASH);

        let hash = compute_event_hash(&event, &prev_hash);
        event.prev_hash = Some(prev_hash);
        event.event_hash = Some(hash);

        let active_model: security_events::ActiveModel = event.clone().into();
        security_events::Entity::insert(active_model)
            .exec(&txn)
            .await
            .map_err(|e| {
                tracing::error!("Failed to store chained security event: {}", e);
                CoreError::InternalServerError
            })?;

        txn.commit().await.map_err(|e| {
            tracing::error!("Failed to commit chained event transaction: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(event)
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

    async fn get_events_ordered_for_verification(
        &self,
        realm_id: RealmId,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        let models = security_events::Entity::find()
            .filter(security_events::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .order_by_asc(security_events::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch events for chain verification: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }
}
