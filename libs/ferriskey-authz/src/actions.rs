//! The action catalogue: one entry per authorization-guarded operation.
//!
//! Names are `resource_type:verb`. They are the stable vocabulary the
//! AuthZen facade accepts and any engine resolves against, so they must
//! not be renamed casually — an action name is a public contract.
//!
//! Generated from the `ensure_policy` call sites; kept in sync by
//! `matrix_covers_every_call_site`.

use crate::entities::ActionId;

pub const CLIENT_CREATE: ActionId = ActionId::new("client:create");
pub const CLIENT_DELETE: ActionId = ActionId::new("client:delete");
pub const CLIENT_READ: ActionId = ActionId::new("client:read");
pub const CLIENT_UPDATE: ActionId = ActionId::new("client:update");
pub const CLIENT_SCOPE_CREATE: ActionId = ActionId::new("client_scope:create");
pub const CLIENT_SCOPE_DELETE: ActionId = ActionId::new("client_scope:delete");
pub const CLIENT_SCOPE_READ: ActionId = ActionId::new("client_scope:read");
pub const CLIENT_SCOPE_UPDATE: ActionId = ActionId::new("client_scope:update");
pub const CREDENTIAL_DELETE: ActionId = ActionId::new("credential:delete");
pub const CREDENTIAL_READ: ActionId = ActionId::new("credential:read");
pub const EMAIL_TEMPLATE_CREATE: ActionId = ActionId::new("email_template:create");
pub const EMAIL_TEMPLATE_DELETE: ActionId = ActionId::new("email_template:delete");
pub const EMAIL_TEMPLATE_READ: ActionId = ActionId::new("email_template:read");
pub const EMAIL_TEMPLATE_UPDATE: ActionId = ActionId::new("email_template:update");
pub const FEDERATION_PROVIDER_CREATE: ActionId = ActionId::new("federation_provider:create");
pub const FEDERATION_PROVIDER_DELETE: ActionId = ActionId::new("federation_provider:delete");
pub const FEDERATION_PROVIDER_READ: ActionId = ActionId::new("federation_provider:read");
pub const FEDERATION_PROVIDER_UPDATE: ActionId = ActionId::new("federation_provider:update");
pub const FLOW_READ: ActionId = ActionId::new("flow:read");
pub const GROUP_CREATE: ActionId = ActionId::new("group:create");
pub const GROUP_DELETE: ActionId = ActionId::new("group:delete");
pub const GROUP_LIST: ActionId = ActionId::new("group:list");
pub const GROUP_READ: ActionId = ActionId::new("group:read");
pub const GROUP_UPDATE: ActionId = ActionId::new("group:update");
pub const GROUP_ATTRIBUTE_DELETE: ActionId = ActionId::new("group_attribute:delete");
pub const GROUP_ATTRIBUTE_UPDATE: ActionId = ActionId::new("group_attribute:update");
pub const GROUP_MEMBER_CREATE: ActionId = ActionId::new("group_member:create");
pub const GROUP_MEMBER_DELETE: ActionId = ActionId::new("group_member:delete");
pub const GROUP_MEMBER_LIST: ActionId = ActionId::new("group_member:list");
pub const GROUP_ROLE_ASSIGN: ActionId = ActionId::new("group_role:assign");
pub const GROUP_ROLE_REVOKE: ActionId = ActionId::new("group_role:revoke");
pub const IDENTITY_PROVIDER_CREATE: ActionId = ActionId::new("identity_provider:create");
pub const IDENTITY_PROVIDER_DELETE: ActionId = ActionId::new("identity_provider:delete");
pub const IDENTITY_PROVIDER_READ: ActionId = ActionId::new("identity_provider:read");
pub const IDENTITY_PROVIDER_UPDATE: ActionId = ActionId::new("identity_provider:update");
pub const MAINTENANCE_CREATE: ActionId = ActionId::new("maintenance:create");
pub const MAINTENANCE_DELETE: ActionId = ActionId::new("maintenance:delete");
pub const MAINTENANCE_READ: ActionId = ActionId::new("maintenance:read");
pub const MAINTENANCE_TOGGLE: ActionId = ActionId::new("maintenance:toggle");
pub const ORGANIZATION_CREATE: ActionId = ActionId::new("organization:create");
pub const ORGANIZATION_DELETE: ActionId = ActionId::new("organization:delete");
pub const ORGANIZATION_LIST: ActionId = ActionId::new("organization:list");
pub const ORGANIZATION_READ: ActionId = ActionId::new("organization:read");
pub const ORGANIZATION_UPDATE: ActionId = ActionId::new("organization:update");
pub const ORGANIZATION_ATTRIBUTE_DELETE: ActionId = ActionId::new("organization_attribute:delete");
pub const ORGANIZATION_ATTRIBUTE_UPDATE: ActionId = ActionId::new("organization_attribute:update");
pub const ORGANIZATION_MEMBER_CREATE: ActionId = ActionId::new("organization_member:create");
pub const ORGANIZATION_MEMBER_DELETE: ActionId = ActionId::new("organization_member:delete");
pub const ORGANIZATION_MEMBER_LIST: ActionId = ActionId::new("organization_member:list");
pub const ORGANIZATION_MEMBER_ROLE_ASSIGN: ActionId =
    ActionId::new("organization_member_role:assign");
pub const ORGANIZATION_MEMBER_ROLE_LIST: ActionId = ActionId::new("organization_member_role:list");
pub const ORGANIZATION_MEMBER_ROLE_REVOKE: ActionId =
    ActionId::new("organization_member_role:revoke");
pub const PASSWORD_POLICY_READ: ActionId = ActionId::new("password_policy:read");
pub const PASSWORD_POLICY_UPDATE: ActionId = ActionId::new("password_policy:update");
pub const PORTAL_LAYOUT_CREATE: ActionId = ActionId::new("portal_layout:create");
pub const PORTAL_LAYOUT_DELETE: ActionId = ActionId::new("portal_layout:delete");
pub const PORTAL_LAYOUT_LIST: ActionId = ActionId::new("portal_layout:list");
pub const PORTAL_LAYOUT_READ: ActionId = ActionId::new("portal_layout:read");
pub const PORTAL_LAYOUT_UPDATE: ActionId = ActionId::new("portal_layout:update");
pub const PORTAL_THEME_ACTIVATE: ActionId = ActionId::new("portal_theme:activate");
pub const PORTAL_THEME_CREATE: ActionId = ActionId::new("portal_theme:create");
pub const PORTAL_THEME_DELETE: ActionId = ActionId::new("portal_theme:delete");
pub const PORTAL_THEME_LIST: ActionId = ActionId::new("portal_theme:list");
pub const PORTAL_THEME_READ: ActionId = ActionId::new("portal_theme:read");
pub const PORTAL_THEME_UPDATE: ActionId = ActionId::new("portal_theme:update");
pub const PROTOCOL_MAPPER_CREATE: ActionId = ActionId::new("protocol_mapper:create");
pub const PROTOCOL_MAPPER_DELETE: ActionId = ActionId::new("protocol_mapper:delete");
pub const PROTOCOL_MAPPER_UPDATE: ActionId = ActionId::new("protocol_mapper:update");
pub const PROVIDER_CREATE: ActionId = ActionId::new("provider:create");
pub const PROVIDER_DELETE: ActionId = ActionId::new("provider:delete");
pub const PROVIDER_READ: ActionId = ActionId::new("provider:read");
pub const PROVIDER_TOGGLE: ActionId = ActionId::new("provider:toggle");
pub const PROVIDER_UPDATE: ActionId = ActionId::new("provider:update");
pub const PROVIDER_MAPPING_CREATE: ActionId = ActionId::new("provider_mapping:create");
pub const PROVIDER_MAPPING_DELETE: ActionId = ActionId::new("provider_mapping:delete");
pub const PROVIDER_MAPPING_LIST: ActionId = ActionId::new("provider_mapping:list");
pub const REALM_CREATE: ActionId = ActionId::new("realm:create");
pub const REALM_DELETE: ActionId = ActionId::new("realm:delete");
pub const REALM_READ: ActionId = ActionId::new("realm:read");
pub const REALM_UPDATE: ActionId = ActionId::new("realm:update");
pub const ROLE_CREATE: ActionId = ActionId::new("role:create");
pub const ROLE_DELETE: ActionId = ActionId::new("role:delete");
pub const ROLE_READ: ActionId = ActionId::new("role:read");
pub const ROLE_UPDATE: ActionId = ActionId::new("role:update");
pub const SCOPE_MAPPING_ASSIGN: ActionId = ActionId::new("scope_mapping:assign");
pub const SCOPE_MAPPING_READ: ActionId = ActionId::new("scope_mapping:read");
pub const SCOPE_MAPPING_UNASSIGN: ActionId = ActionId::new("scope_mapping:unassign");
pub const SECURITY_EVENT_READ: ActionId = ActionId::new("security_event:read");
pub const SECURITY_EVENT_VERIFY_CHAIN: ActionId = ActionId::new("security_event:verify_chain");
pub const USER_BULK_DELETE: ActionId = ActionId::new("user:bulk_delete");
pub const USER_CREATE: ActionId = ActionId::new("user:create");
pub const USER_DELETE: ActionId = ActionId::new("user:delete");
pub const USER_READ: ActionId = ActionId::new("user:read");
pub const USER_UNLOCK: ActionId = ActionId::new("user:unlock");
pub const USER_UPDATE: ActionId = ActionId::new("user:update");
pub const USER_ATTRIBUTE_DELETE: ActionId = ActionId::new("user_attribute:delete");
pub const USER_ATTRIBUTE_READ: ActionId = ActionId::new("user_attribute:read");
pub const USER_ROLE_ASSIGN: ActionId = ActionId::new("user_role:assign");
pub const USER_ROLE_UNASSIGN: ActionId = ActionId::new("user_role:unassign");
pub const WEBHOOK_CREATE: ActionId = ActionId::new("webhook:create");
pub const WEBHOOK_DELETE: ActionId = ActionId::new("webhook:delete");
pub const WEBHOOK_READ: ActionId = ActionId::new("webhook:read");
pub const WEBHOOK_UPDATE: ActionId = ActionId::new("webhook:update");

/// Every action the matrix references, for catalogue lookups by name.
pub const ALL: &[ActionId] = &[
    CLIENT_CREATE,
    CLIENT_DELETE,
    CLIENT_READ,
    CLIENT_UPDATE,
    CLIENT_SCOPE_CREATE,
    CLIENT_SCOPE_DELETE,
    CLIENT_SCOPE_READ,
    CLIENT_SCOPE_UPDATE,
    CREDENTIAL_DELETE,
    CREDENTIAL_READ,
    EMAIL_TEMPLATE_CREATE,
    EMAIL_TEMPLATE_DELETE,
    EMAIL_TEMPLATE_READ,
    EMAIL_TEMPLATE_UPDATE,
    FEDERATION_PROVIDER_CREATE,
    FEDERATION_PROVIDER_DELETE,
    FEDERATION_PROVIDER_READ,
    FEDERATION_PROVIDER_UPDATE,
    FLOW_READ,
    GROUP_CREATE,
    GROUP_DELETE,
    GROUP_LIST,
    GROUP_READ,
    GROUP_UPDATE,
    GROUP_ATTRIBUTE_DELETE,
    GROUP_ATTRIBUTE_UPDATE,
    GROUP_MEMBER_CREATE,
    GROUP_MEMBER_DELETE,
    GROUP_MEMBER_LIST,
    GROUP_ROLE_ASSIGN,
    GROUP_ROLE_REVOKE,
    IDENTITY_PROVIDER_CREATE,
    IDENTITY_PROVIDER_DELETE,
    IDENTITY_PROVIDER_READ,
    IDENTITY_PROVIDER_UPDATE,
    MAINTENANCE_CREATE,
    MAINTENANCE_DELETE,
    MAINTENANCE_READ,
    MAINTENANCE_TOGGLE,
    ORGANIZATION_CREATE,
    ORGANIZATION_DELETE,
    ORGANIZATION_LIST,
    ORGANIZATION_READ,
    ORGANIZATION_UPDATE,
    ORGANIZATION_ATTRIBUTE_DELETE,
    ORGANIZATION_ATTRIBUTE_UPDATE,
    ORGANIZATION_MEMBER_CREATE,
    ORGANIZATION_MEMBER_DELETE,
    ORGANIZATION_MEMBER_LIST,
    ORGANIZATION_MEMBER_ROLE_ASSIGN,
    ORGANIZATION_MEMBER_ROLE_LIST,
    ORGANIZATION_MEMBER_ROLE_REVOKE,
    PASSWORD_POLICY_READ,
    PASSWORD_POLICY_UPDATE,
    PORTAL_LAYOUT_CREATE,
    PORTAL_LAYOUT_DELETE,
    PORTAL_LAYOUT_LIST,
    PORTAL_LAYOUT_READ,
    PORTAL_LAYOUT_UPDATE,
    PORTAL_THEME_ACTIVATE,
    PORTAL_THEME_CREATE,
    PORTAL_THEME_DELETE,
    PORTAL_THEME_LIST,
    PORTAL_THEME_READ,
    PORTAL_THEME_UPDATE,
    PROTOCOL_MAPPER_CREATE,
    PROTOCOL_MAPPER_DELETE,
    PROTOCOL_MAPPER_UPDATE,
    PROVIDER_CREATE,
    PROVIDER_DELETE,
    PROVIDER_READ,
    PROVIDER_TOGGLE,
    PROVIDER_UPDATE,
    PROVIDER_MAPPING_CREATE,
    PROVIDER_MAPPING_DELETE,
    PROVIDER_MAPPING_LIST,
    REALM_CREATE,
    REALM_DELETE,
    REALM_READ,
    REALM_UPDATE,
    ROLE_CREATE,
    ROLE_DELETE,
    ROLE_READ,
    ROLE_UPDATE,
    SCOPE_MAPPING_ASSIGN,
    SCOPE_MAPPING_READ,
    SCOPE_MAPPING_UNASSIGN,
    SECURITY_EVENT_READ,
    SECURITY_EVENT_VERIFY_CHAIN,
    USER_BULK_DELETE,
    USER_CREATE,
    USER_DELETE,
    USER_READ,
    USER_UNLOCK,
    USER_UPDATE,
    USER_ATTRIBUTE_DELETE,
    USER_ATTRIBUTE_READ,
    USER_ROLE_ASSIGN,
    USER_ROLE_UNASSIGN,
    WEBHOOK_CREATE,
    WEBHOOK_DELETE,
    WEBHOOK_READ,
    WEBHOOK_UPDATE,
];

/// Resolve an action name coming from outside (an AuthZen request).
pub fn by_name(name: &str) -> Option<ActionId> {
    ALL.iter().find(|a| a.as_str() == name).cloned()
}
