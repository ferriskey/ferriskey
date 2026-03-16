use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::{NoContext, Timestamp, Uuid};

use ferriskey_domain::realm::RealmId;

/// Unique identifier for an Organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, ToSchema)]
pub struct OrganizationId(Uuid);

impl OrganizationId {
    pub fn new(id: Uuid) -> Self {
        Self(id)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for OrganizationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for OrganizationId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<OrganizationId> for Uuid {
    fn from(id: OrganizationId) -> Self {
        id.0
    }
}

/// Organization domain entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Organization {
    pub id: OrganizationId,
    pub realm_id: RealmId,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for constructing a new Organization
pub struct OrganizationConfig {
    pub realm_id: RealmId,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

impl Organization {
    pub fn new(config: OrganizationConfig) -> Self {
        let now = Utc::now();
        let seconds = now.timestamp().try_into().unwrap_or(0);
        let timestamp = Timestamp::from_unix(NoContext, seconds, 0);

        Self {
            id: OrganizationId::new(Uuid::new_v7(timestamp)),
            realm_id: config.realm_id,
            name: config.name,
            display_name: config.display_name,
            description: config.description,
            enabled: config.enabled,
            created_at: now,
            updated_at: now,
        }
    }
}

/// A user's membership in an organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OrganizationMember {
    pub organization_id: OrganizationId,
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

impl OrganizationMember {
    pub fn new(organization_id: OrganizationId, user_id: Uuid) -> Self {
        Self {
            organization_id,
            user_id,
            joined_at: Utc::now(),
        }
    }
}

// --- Input structs ---

pub struct CreateOrganizationInput {
    pub realm_name: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

pub struct GetOrganizationInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
}

pub struct ListOrganizationsInput {
    pub realm_name: String,
}

pub struct UpdateOrganizationInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

pub struct DeleteOrganizationInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
}

pub struct AddOrganizationMemberInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
    pub user_id: Uuid,
}

pub struct RemoveOrganizationMemberInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
    pub user_id: Uuid,
}

pub struct ListOrganizationMembersInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
}
