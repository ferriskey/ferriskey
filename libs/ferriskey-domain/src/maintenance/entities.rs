use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{generate_timestamp, realm::RealmId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MaintenanceWhitelistEntry {
    pub id: Uuid,
    pub client_id: Uuid,
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl MaintenanceWhitelistEntry {
    pub fn new_user(client_id: Uuid, user_id: Uuid) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            client_id,
            user_id: Some(user_id),
            role_id: None,
            created_at: now,
        }
    }

    pub fn new_role(client_id: Uuid, role_id: Uuid) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            client_id,
            user_id: None,
            role_id: Some(role_id),
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct RealmMaintenanceWhitelistEntry {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub user_id: Option<Uuid>,
    pub role_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl RealmMaintenanceWhitelistEntry {
    pub fn new_user(realm_id: RealmId, user_id: Uuid) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            user_id: Some(user_id),
            role_id: None,
            created_at: now,
        }
    }

    pub fn new_role(realm_id: RealmId, role_id: Uuid) -> Self {
        let (now, timestamp) = generate_timestamp();
        Self {
            id: Uuid::new_v7(timestamp),
            realm_id,
            user_id: None,
            role_id: Some(role_id),
            created_at: now,
        }
    }
}
