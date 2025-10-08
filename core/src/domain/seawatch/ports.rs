use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::authentication::entities::Identity;
use crate::domain::realm::entities::Realm;
use crate::error::CoreError;

use super::entities::SecurityEvent;
use super::value_objects::SecurityEventFilter;

type Result<T> = std::result::Result<T, CoreError>;

#[async_trait]
pub trait SecurityEventRepository: Clone + Send + Sync + 'static {
    async fn store_event(&self, event: SecurityEvent) -> Result<()>;
    async fn get_events(&self, realm_id: Uuid, filter: SecurityEventFilter) -> Result<Vec<SecurityEvent>>;
    async fn get_by_id(&self, id: Uuid) -> Result<SecurityEvent>;
    async fn count_events(&self, realm_id: Uuid, filter: SecurityEventFilter) -> Result<i64>;
}

#[async_trait]
pub trait SecurityEventPolicy: Send + Sync + Clone {
    async fn can_view_events(&self, identity: Identity, realm: Realm) -> Result<bool>;
    async fn can_export_events(&self, identity: Identity, realm: Realm) -> Result<bool>;
}