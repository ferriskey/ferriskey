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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use ferriskey_domain::realm::RealmId;

    use super::*;

    fn realm_id() -> RealmId {
        RealmId::new(Uuid::new_v4())
    }

    fn org_config(name: &str) -> OrganizationConfig {
        OrganizationConfig {
            realm_id: realm_id(),
            name: name.to_string(),
            display_name: None,
            description: None,
            enabled: true,
        }
    }

    // --- OrganizationId ---

    #[test]
    fn organization_id_roundtrip_uuid() {
        let uuid = Uuid::new_v4();
        let id = OrganizationId::from(uuid);

        assert_eq!(Uuid::from(id), uuid);
    }

    #[test]
    fn organization_id_as_uuid_matches_inner() {
        let uuid = Uuid::new_v4();
        let id = OrganizationId::new(uuid);

        assert_eq!(id.as_uuid(), uuid);
    }

    #[test]
    fn organization_id_display_matches_uuid_string() {
        let uuid = Uuid::new_v4();
        let id = OrganizationId::new(uuid);

        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn organization_id_equality() {
        let uuid = Uuid::new_v4();
        assert_eq!(OrganizationId::new(uuid), OrganizationId::new(uuid));
        assert_ne!(
            OrganizationId::new(uuid),
            OrganizationId::new(Uuid::new_v4())
        );
    }

    // --- Organization::new ---

    #[test]
    fn organization_new_sets_fields_from_config() {
        let realm = realm_id();
        let config = OrganizationConfig {
            realm_id: realm,
            name: "acme".to_string(),
            display_name: Some("Acme Corp".to_string()),
            description: Some("A test org".to_string()),
            enabled: true,
        };

        let org = Organization::new(config);

        assert_eq!(org.name, "acme");
        assert_eq!(org.display_name.as_deref(), Some("Acme Corp"));
        assert_eq!(org.description.as_deref(), Some("A test org"));
        assert!(org.enabled);
        assert_eq!(org.realm_id, realm);
    }

    #[test]
    fn organization_new_generates_unique_ids() {
        let org1 = Organization::new(org_config("org1"));
        let org2 = Organization::new(org_config("org2"));

        assert_ne!(org1.id, org2.id);
    }

    #[test]
    fn organization_new_created_at_equals_updated_at() {
        let org = Organization::new(org_config("org"));

        assert_eq!(org.created_at, org.updated_at);
    }

    #[test]
    fn organization_new_without_optional_fields() {
        let org = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "minimal".to_string(),
            display_name: None,
            description: None,
            enabled: false,
        });

        assert!(org.display_name.is_none());
        assert!(org.description.is_none());
        assert!(!org.enabled);
    }

    // --- OrganizationMember::new ---

    #[test]
    fn organization_member_new_sets_fields() {
        let org_id = OrganizationId::new(Uuid::new_v4());
        let user_id = Uuid::new_v4();

        let member = OrganizationMember::new(org_id, user_id);

        assert_eq!(member.organization_id, org_id);
        assert_eq!(member.user_id, user_id);
    }

    #[test]
    fn organization_member_new_generates_recent_joined_at() {
        let before = chrono::Utc::now();
        let member = OrganizationMember::new(OrganizationId::new(Uuid::new_v4()), Uuid::new_v4());
        let after = chrono::Utc::now();

        assert!(member.joined_at >= before);
        assert!(member.joined_at <= after);
    }
}
