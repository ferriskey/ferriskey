use std::fmt::Display;

use chrono::{DateTime, Utc};
use ferriskey_domain::{generate_timestamp, realm::RealmId};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

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

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OrganizationValidationError {
    #[error("organization name is required")]
    EmptyName,
    #[error("organization name must be at most 255 characters")]
    NameTooLong,
    #[error("organization alias is required")]
    EmptyAlias,
    #[error("organization alias must be at most 255 characters")]
    AliasTooLong,
    #[error(
        "organization alias must contain only lowercase letters, digits, hyphen, or underscore"
    )]
    InvalidAlias,
    #[error("organization domain must be at most 255 characters")]
    DomainTooLong,
    #[error("organization redirect_url cannot be empty when provided")]
    EmptyRedirectUrl,
    #[error("organization attribute key is required")]
    EmptyAttributeKey,
    #[error("organization attribute key must be at most 255 characters")]
    AttributeKeyTooLong,
    #[error("organization attribute value is required")]
    EmptyAttributeValue,
    #[error("organization members must belong to the same realm")]
    CrossRealmMembership,
}

/// Organization domain entity
///
/// `name` is the primary human-readable label shown in admin or end-user surfaces.
/// `alias` is the stable URL-safe identifier used for lookups and routing within a realm.
/// The first organizations release does not persist a separate `display_name`, so callers
/// should rely on `name` as the display label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Organization {
    pub id: OrganizationId,
    pub realm_id: RealmId,
    pub name: String,
    pub alias: String,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for constructing a new Organization
pub struct OrganizationConfig {
    pub realm_id: RealmId,
    pub name: String,
    pub alias: String,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CreateOrganizationParams {
    pub realm_id: RealmId,
    pub name: String,
    pub alias: String,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UpdateOrganizationParams {
    pub name: Option<String>,
    pub alias: Option<String>,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

impl Organization {
    pub fn new(config: OrganizationConfig) -> Result<Self, OrganizationValidationError> {
        let (now, timestamp) = generate_timestamp();

        Ok(Self {
            id: OrganizationId::new(Uuid::new_v7(timestamp)),
            realm_id: config.realm_id,
            name: validate_required_name(config.name)?,
            alias: validate_alias(config.alias)?,
            domain: validate_optional_domain(config.domain)?,
            redirect_url: validate_optional_redirect_url(config.redirect_url)?,
            description: normalize_optional_text(config.description),
            enabled: config.enabled,
            created_at: now,
            updated_at: now,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OrganizationAttribute {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub key: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
}

impl OrganizationAttribute {
    pub fn new(
        organization_id: OrganizationId,
        key: String,
        value: String,
    ) -> Result<Self, OrganizationValidationError> {
        let (now, timestamp) = generate_timestamp();

        Ok(Self {
            id: Uuid::new_v7(timestamp),
            organization_id,
            key: validate_attribute_key(key)?,
            value: validate_attribute_value(value)?,
            created_at: now,
        })
    }
}

/// A user's membership in an organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct OrganizationMember {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl OrganizationMember {
    pub fn new(organization_id: OrganizationId, user_id: Uuid) -> Self {
        let (now, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            organization_id,
            user_id,
            created_at: now,
        }
    }
}

pub fn validate_membership_realms(
    organization_realm_id: RealmId,
    user_realm_id: RealmId,
) -> Result<(), OrganizationValidationError> {
    if organization_realm_id == user_realm_id {
        Ok(())
    } else {
        Err(OrganizationValidationError::CrossRealmMembership)
    }
}

// --- Input structs ---

pub struct CreateOrganizationInput {
    pub realm_name: String,
    pub name: String,
    pub alias: String,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
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
    pub name: Option<String>,
    pub alias: Option<String>,
    pub domain: Option<String>,
    pub redirect_url: Option<String>,
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

pub struct ListUserOrganizationsInput {
    pub realm_name: String,
    pub user_id: Uuid,
}

pub struct ListOrganizationAttributesInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
}

pub struct UpsertOrganizationAttributeInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
    pub key: String,
    pub value: String,
}

pub struct DeleteOrganizationAttributeInput {
    pub realm_name: String,
    pub organization_id: OrganizationId,
    pub key: String,
}

fn validate_required_name(name: String) -> Result<String, OrganizationValidationError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(OrganizationValidationError::EmptyName);
    }
    if trimmed.chars().count() > 255 {
        return Err(OrganizationValidationError::NameTooLong);
    }

    Ok(trimmed.to_string())
}

fn validate_alias(alias: String) -> Result<String, OrganizationValidationError> {
    let trimmed = alias.trim();
    if trimmed.is_empty() {
        return Err(OrganizationValidationError::EmptyAlias);
    }
    if trimmed.chars().count() > 255 {
        return Err(OrganizationValidationError::AliasTooLong);
    }
    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(OrganizationValidationError::InvalidAlias);
    }

    Ok(trimmed.to_string())
}

fn validate_optional_domain(
    domain: Option<String>,
) -> Result<Option<String>, OrganizationValidationError> {
    let Some(domain) = domain else {
        return Ok(None);
    };

    let trimmed = domain.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > 255 {
        return Err(OrganizationValidationError::DomainTooLong);
    }

    Ok(Some(trimmed.to_string()))
}

fn validate_optional_redirect_url(
    redirect_url: Option<String>,
) -> Result<Option<String>, OrganizationValidationError> {
    let Some(redirect_url) = redirect_url else {
        return Ok(None);
    };

    let trimmed = redirect_url.trim();
    if trimmed.is_empty() {
        return Err(OrganizationValidationError::EmptyRedirectUrl);
    }

    Ok(Some(trimmed.to_string()))
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn validate_attribute_key(key: String) -> Result<String, OrganizationValidationError> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return Err(OrganizationValidationError::EmptyAttributeKey);
    }
    if trimmed.chars().count() > 255 {
        return Err(OrganizationValidationError::AttributeKeyTooLong);
    }

    Ok(trimmed.to_string())
}

fn validate_attribute_value(value: String) -> Result<String, OrganizationValidationError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(OrganizationValidationError::EmptyAttributeValue);
    }

    Ok(trimmed.to_string())
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
            alias: format!("{name}-alias"),
            domain: None,
            redirect_url: None,
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
            name: "Acme Corp".to_string(),
            alias: "acme".to_string(),
            domain: Some("acme.com".to_string()),
            redirect_url: Some("https://app.acme.com/callback".to_string()),
            description: Some("A test org".to_string()),
            enabled: true,
        };

        let org = Organization::new(config).unwrap();

        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.alias, "acme");
        assert_eq!(org.domain.as_deref(), Some("acme.com"));
        assert_eq!(
            org.redirect_url.as_deref(),
            Some("https://app.acme.com/callback")
        );
        assert_eq!(org.description.as_deref(), Some("A test org"));
        assert!(org.enabled);
        assert_eq!(org.realm_id, realm);
    }

    #[test]
    fn organization_new_generates_unique_ids() {
        let org1 = Organization::new(org_config("org1")).unwrap();
        let org2 = Organization::new(org_config("org2")).unwrap();

        assert_ne!(org1.id, org2.id);
    }

    #[test]
    fn organization_new_created_at_equals_updated_at() {
        let org = Organization::new(org_config("org")).unwrap();

        assert_eq!(org.created_at, org.updated_at);
    }

    #[test]
    fn organization_new_without_optional_fields() {
        let org = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "minimal".to_string(),
            alias: "minimal".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: false,
        })
        .unwrap();

        assert!(org.domain.is_none());
        assert!(org.redirect_url.is_none());
        assert!(org.description.is_none());
        assert!(!org.enabled);
    }

    #[test]
    fn organization_new_trims_optional_text_fields() {
        let org = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "  Acme Corp  ".to_string(),
            alias: "acme".to_string(),
            domain: Some("  acme.com  ".to_string()),
            redirect_url: Some("  https://app.acme.com  ".to_string()),
            description: Some("  Hello world  ".to_string()),
            enabled: true,
        })
        .unwrap();

        assert_eq!(org.name, "Acme Corp");
        assert_eq!(org.domain.as_deref(), Some("acme.com"));
        assert_eq!(org.redirect_url.as_deref(), Some("https://app.acme.com"));
        assert_eq!(org.description.as_deref(), Some("Hello world"));
    }

    #[test]
    fn organization_new_rejects_empty_name() {
        let result = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "   ".to_string(),
            alias: "acme".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: true,
        });

        assert_eq!(result, Err(OrganizationValidationError::EmptyName));
    }

    #[test]
    fn organization_new_rejects_invalid_alias() {
        let result = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "Acme".to_string(),
            alias: "Acme Corp".to_string(),
            domain: None,
            redirect_url: None,
            description: None,
            enabled: true,
        });

        assert_eq!(result, Err(OrganizationValidationError::InvalidAlias));
    }

    #[test]
    fn organization_new_rejects_blank_redirect_url() {
        let result = Organization::new(OrganizationConfig {
            realm_id: realm_id(),
            name: "Acme".to_string(),
            alias: "acme".to_string(),
            domain: None,
            redirect_url: Some("   ".to_string()),
            description: None,
            enabled: true,
        });

        assert_eq!(result, Err(OrganizationValidationError::EmptyRedirectUrl));
    }

    // --- OrganizationAttribute::new ---

    #[test]
    fn organization_attribute_new_sets_fields() {
        let organization_id = OrganizationId::new(Uuid::new_v4());

        let attribute = OrganizationAttribute::new(
            organization_id,
            "department".to_string(),
            "engineering".to_string(),
        )
        .unwrap();

        assert_eq!(attribute.organization_id, organization_id);
        assert_eq!(attribute.key, "department");
        assert_eq!(attribute.value, "engineering");
    }

    #[test]
    fn organization_attribute_new_rejects_empty_key() {
        let result = OrganizationAttribute::new(
            OrganizationId::new(Uuid::new_v4()),
            "  ".to_string(),
            "engineering".to_string(),
        );

        assert_eq!(result, Err(OrganizationValidationError::EmptyAttributeKey));
    }

    #[test]
    fn organization_attribute_new_rejects_empty_value() {
        let result = OrganizationAttribute::new(
            OrganizationId::new(Uuid::new_v4()),
            "department".to_string(),
            "  ".to_string(),
        );

        assert_eq!(
            result,
            Err(OrganizationValidationError::EmptyAttributeValue)
        );
    }

    // --- OrganizationMember::new ---

    #[test]
    fn organization_member_new_sets_fields() {
        let org_id = OrganizationId::new(Uuid::new_v4());
        let user_id = Uuid::new_v4();

        let member = OrganizationMember::new(org_id, user_id);

        assert_eq!(member.organization_id, org_id);
        assert_eq!(member.user_id, user_id);
        assert_ne!(member.id, Uuid::nil());
    }

    #[test]
    fn organization_member_new_generates_recent_created_at() {
        let before = chrono::Utc::now();
        let member = OrganizationMember::new(OrganizationId::new(Uuid::new_v4()), Uuid::new_v4());
        let after = chrono::Utc::now();

        assert!(member.created_at >= before);
        assert!(member.created_at <= after);
    }

    #[test]
    fn validate_membership_realms_accepts_same_realm() {
        let realm = realm_id();

        assert_eq!(validate_membership_realms(realm, realm), Ok(()));
    }

    #[test]
    fn validate_membership_realms_rejects_cross_realm_membership() {
        let result = validate_membership_realms(realm_id(), realm_id());

        assert_eq!(
            result,
            Err(OrganizationValidationError::CrossRealmMembership)
        );
    }
}
