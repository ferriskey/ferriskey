use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::authentication::entities::Identity;
use crate::domain::realm::entities::Realm;
use crate::error::CoreError;

use super::entities::SecurityEvent;

#[async_trait]
pub trait SecurityEventRepository: Clone + Send + Sync + 'static {
    async fn store_event(&self, event: SecurityEvent) -> Result<(), CoreError>;
    async fn get_events(&self,realm_id: Uuid,filter: SecurityEventFilter) -> Result<Vec<SecurityEvent>, CoreError>;
    async fn count_events(&self,realm_id: Uuid,filter: SecurityEventFilter) -> Result<i64, CoreError>;
}

#[async_trait]
pub trait SecurityEventPolicy: Send + Sync + Clone {
    async fn can_view_events(&self, identity: Identity, realm: Realm) -> Result<bool, CoreError>;
}