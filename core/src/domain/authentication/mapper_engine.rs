use std::collections::HashMap;

use ferriskey_aegis::entities::ProtocolMapper;
use ferriskey_organization::OrganizationId;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{common::entities::app_errors::CoreError, realm::entities::RealmId};

use super::mappers::{
    audience_mapper::AudienceMapper, hardcoded_claim_mapper::HardcodedClaimMapper,
    org_detail_mapper::OrgDetailMapper, org_membership_mapper::OrgMembershipMapper,
    user_attribute_mapper::UserAttributeMapper, user_client_role_mapper::UserClientRoleMapper,
    user_property_mapper::UserPropertyMapper, user_realm_role_mapper::UserRealmRoleMapper,
};

/// Organization membership data available to protocol mappers.
/// Attributes are pre-loaded so mappers don't need repository access.
#[derive(Debug, Clone)]
pub struct ContextOrganization {
    pub id: OrganizationId,
    pub name: String,
    pub alias: String,
    pub domain: Option<String>,
    /// Flat key-value attributes defined on the organization.
    pub attributes: HashMap<String, String>,
}

/// All user/client/realm data that protocol mappers may need.
/// Assembled once before mapper execution and passed by reference.
#[derive(Debug, Clone)]
pub struct MapperContext {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub email_verified: bool,
    pub firstname: String,
    pub lastname: String,
    pub realm_roles: Vec<String>,
    /// Client roles keyed by the client's string `client_id` (e.g. `"backend"`).
    /// Populated from user role assignments that are scoped to a specific client.
    pub client_roles: HashMap<String, Vec<String>>,
    pub client_id: String,
    pub client_uuid: Uuid,
    pub realm_name: String,
    pub realm_id: RealmId,
    pub user_attributes: HashMap<String, Value>,
    /// Organizations the user belongs to, with their attributes pre-loaded.
    pub organizations: Vec<ContextOrganization>,
}

/// Which token the mapper should apply to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    AccessToken,
    IdToken,
}

/// The result of mapper execution. Separates flat claims from audience additions
/// since `aud` is a typed Vec<String> on JwtClaim and needs special handling.
#[derive(Debug, Clone, Default)]
pub struct MapperOutput {
    pub claims: HashMap<String, Value>,
    pub additional_audiences: Vec<String>,
}

/// Static dispatch enum for protocol mapper executors.
/// All mapper types are known at compile time, so no dynamic dispatch is needed.
#[derive(Debug)]
enum MapperExecutor {
    UserProperty(UserPropertyMapper),
    HardcodedClaim(HardcodedClaimMapper),
    Audience(AudienceMapper),
    UserAttribute(UserAttributeMapper),
    UserClientRole(UserClientRoleMapper),
    UserRealmRole(UserRealmRoleMapper),
    OrgMembership(OrgMembershipMapper),
    OrgDetail(OrgDetailMapper),
}

impl MapperExecutor {
    fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        match self {
            Self::UserProperty(m) => m.execute(config, context, token_type),
            Self::HardcodedClaim(m) => m.execute(config, context, token_type),
            Self::Audience(m) => m.execute(config, context, token_type),
            Self::UserAttribute(m) => m.execute(config, context, token_type),
            Self::UserClientRole(m) => m.execute(config, context, token_type),
            Self::UserRealmRole(m) => m.execute(config, context, token_type),
            Self::OrgMembership(m) => m.execute(config, context, token_type),
            Self::OrgDetail(m) => m.execute(config, context, token_type),
        }
    }
}

/// Registry-based engine that dispatches protocol mappers to their executor.
#[derive(Debug)]
pub struct MapperEngine {
    executors: HashMap<String, MapperExecutor>,
}

impl MapperEngine {
    pub fn new() -> Self {
        let mut executors = HashMap::new();
        executors.insert(
            "oidc-usermodel-property-mapper".to_string(),
            MapperExecutor::UserProperty(UserPropertyMapper),
        );
        executors.insert(
            "oidc-hardcoded-claim-mapper".to_string(),
            MapperExecutor::HardcodedClaim(HardcodedClaimMapper),
        );
        executors.insert(
            "oidc-audience-mapper".to_string(),
            MapperExecutor::Audience(AudienceMapper),
        );
        executors.insert(
            "oidc-usermodel-attribute-mapper".to_string(),
            MapperExecutor::UserAttribute(UserAttributeMapper),
        );
        executors.insert(
            "oidc-usermodel-realm-role-mapper".to_string(),
            MapperExecutor::UserRealmRole(UserRealmRoleMapper),
        );
        executors.insert(
            "oidc-usermodel-client-role-mapper".to_string(),
            MapperExecutor::UserClientRole(UserClientRoleMapper),
        );
        executors.insert(
            "oidc-organization-membership-mapper".to_string(),
            MapperExecutor::OrgMembership(OrgMembershipMapper),
        );
        executors.insert(
            "oidc-organization-detail-mapper".to_string(),
            MapperExecutor::OrgDetail(OrgDetailMapper),
        );
        Self { executors }
    }

    /// Apply all mappers and produce merged claims + audience additions.
    pub fn apply_mappers(
        &self,
        protocol_mappers: &[ProtocolMapper],
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        let mut result = MapperOutput::default();

        for mapper in protocol_mappers {
            if let Some(executor) = self.executors.get(&mapper.mapper_type) {
                let output = executor.execute(&mapper.config, context, token_type)?;
                deep_merge(&mut result.claims, output.claims);
                result
                    .additional_audiences
                    .extend(output.additional_audiences);
            } else {
                tracing::warn!(
                    mapper_type = %mapper.mapper_type,
                    mapper_name = %mapper.name,
                    "Unknown mapper type, skipping"
                );
            }
        }

        // Filter out reserved claim names to prevent serde flatten conflicts
        for reserved in RESERVED_CLAIMS {
            result.claims.remove(*reserved);
        }

        Ok(result)
    }
}

impl Default for MapperEngine {
    fn default() -> Self {
        Self::new()
    }
}

const RESERVED_CLAIMS: &[&str] = &[
    "sub",
    "iat",
    "jti",
    "iss",
    "typ",
    "azp",
    "aud",
    "scope",
    "exp",
    "client_id",
];

/// Resolve the claim name from config, checking `claim.name` first then `token.claim.name`
/// as a legacy fallback (some frontends stored mappers under `token.claim.name`).
pub fn resolve_claim_name<'a>(config: &'a Value, fallback: &'a str) -> &'a str {
    config
        .get("claim.name")
        .or_else(|| config.get("token.claim.name"))
        .and_then(|v| v.as_str())
        .unwrap_or(fallback)
}

/// Check if a mapper should apply to the given token type based on its config.
pub fn should_apply_to_token(config: &Value, token_type: TokenType) -> bool {
    let key = match token_type {
        TokenType::AccessToken => "access.token.claim",
        TokenType::IdToken => "id.token.claim",
    };

    config
        .get(key)
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::String(s) => Some(s == "true"),
            _ => None,
        })
        .unwrap_or(true)
}

/// Set a claim value at a dot-notation path, creating intermediate objects as needed.
/// e.g., "realm_access.roles" with value ["admin"] produces {"realm_access": {"roles": ["admin"]}}
pub fn set_claim_at_path(claims: &mut HashMap<String, Value>, path: &str, value: Value) {
    let parts: Vec<&str> = path.split('.').collect();

    if parts.len() == 1 {
        claims.insert(path.to_string(), value);
        return;
    }

    // Build nested JSON object from the inside out
    let nested = build_nested_value(&parts[1..], value);

    let root = parts[0].to_string();
    match claims.get_mut(&root) {
        Some(Value::Object(existing_obj)) => {
            if let Value::Object(new_obj) = &nested {
                merge_json_objects(existing_obj, new_obj);
            }
        }
        _ => {
            claims.insert(root, nested);
        }
    }
}

/// Recursively build a nested JSON object from path parts.
fn build_nested_value(parts: &[&str], value: Value) -> Value {
    if parts.len() == 1 {
        let mut map = serde_json::Map::new();
        map.insert(parts[0].to_string(), value);
        Value::Object(map)
    } else {
        let mut map = serde_json::Map::new();
        map.insert(parts[0].to_string(), build_nested_value(&parts[1..], value));
        Value::Object(map)
    }
}

/// Recursively merge source JSON object into target.
fn merge_json_objects(
    target: &mut serde_json::Map<String, Value>,
    source: &serde_json::Map<String, Value>,
) {
    for (key, source_val) in source {
        match target.get_mut(key) {
            Some(Value::Object(target_inner)) => {
                if let Value::Object(source_inner) = source_val {
                    merge_json_objects(target_inner, source_inner);
                } else {
                    target.insert(key.clone(), source_val.clone());
                }
            }
            _ => {
                target.insert(key.clone(), source_val.clone());
            }
        }
    }
}

/// Deep merge source into target. For conflicting object keys, merge recursively.
/// For conflicting non-object keys, source overwrites target.
pub fn deep_merge(target: &mut HashMap<String, Value>, source: HashMap<String, Value>) {
    for (key, source_val) in source {
        match target.get_mut(&key) {
            Some(Value::Object(target_obj)) => {
                if let Value::Object(source_obj) = source_val {
                    for (k, v) in source_obj {
                        match target_obj.get_mut(&k) {
                            Some(Value::Object(inner_target)) => {
                                if let Value::Object(inner_source) = v {
                                    for (ik, iv) in inner_source {
                                        inner_target.insert(ik, iv);
                                    }
                                } else {
                                    target_obj.insert(k, v);
                                }
                            }
                            _ => {
                                target_obj.insert(k, v);
                            }
                        }
                    }
                } else {
                    target.insert(key, source_val);
                }
            }
            _ => {
                target.insert(key, source_val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_should_apply_to_token_with_bool() {
        let config = json!({
            "access.token.claim": true,
            "id.token.claim": false,
        });

        assert!(should_apply_to_token(&config, TokenType::AccessToken));
        assert!(!should_apply_to_token(&config, TokenType::IdToken));
    }

    #[test]
    fn test_should_apply_to_token_with_string() {
        let config = json!({
            "access.token.claim": "true",
            "id.token.claim": "false",
        });

        assert!(should_apply_to_token(&config, TokenType::AccessToken));
        assert!(!should_apply_to_token(&config, TokenType::IdToken));
    }

    #[test]
    fn test_should_apply_defaults_to_true() {
        let config = json!({});
        assert!(should_apply_to_token(&config, TokenType::AccessToken));
        assert!(should_apply_to_token(&config, TokenType::IdToken));
    }

    #[test]
    fn test_set_claim_simple_path() {
        let mut claims = HashMap::new();
        set_claim_at_path(&mut claims, "preferred_username", json!("john"));
        assert_eq!(claims.get("preferred_username"), Some(&json!("john")));
    }

    #[test]
    fn test_set_claim_nested_path() {
        let mut claims = HashMap::new();
        set_claim_at_path(&mut claims, "realm_access.roles", json!(["admin", "user"]));
        assert_eq!(
            claims.get("realm_access"),
            Some(&json!({"roles": ["admin", "user"]}))
        );
    }

    #[test]
    fn test_deep_merge_objects() {
        let mut target = HashMap::new();
        target.insert("realm_access".to_string(), json!({"roles": ["admin"]}));

        let mut source = HashMap::new();
        source.insert("new_claim".to_string(), json!("value"));

        deep_merge(&mut target, source);

        assert_eq!(
            target.get("realm_access"),
            Some(&json!({"roles": ["admin"]}))
        );
        assert_eq!(target.get("new_claim"), Some(&json!("value")));
    }

    #[test]
    fn test_reserved_claims_are_filtered() {
        let engine = MapperEngine::new();
        let context = MapperContext {
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![],
        };

        // Empty mappers should produce empty output
        let result = engine
            .apply_mappers(&[], &context, TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
        assert!(result.additional_audiences.is_empty());
    }

    #[test]
    fn test_user_realm_role_mapper_integration() {
        use ferriskey_aegis::entities::ProtocolMapper;

        let engine = MapperEngine::new();
        let context = MapperContext {
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            realm_roles: vec!["admin".to_string(), "user".to_string()],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![],
        };

        let mapper = ProtocolMapper {
            id: Uuid::new_v4(),
            client_scope_id: Uuid::new_v4(),
            name: "realm-roles-mapper".to_string(),
            mapper_type: "oidc-usermodel-realm-role-mapper".to_string(),
            config: json!({
                "claim.name": "realm_access.roles",
                "access.token.claim": "true",
            }),
            created_at: chrono::Utc::now(),
        };

        let result = engine
            .apply_mappers(&[mapper], &context, TokenType::AccessToken)
            .unwrap();

        assert_eq!(
            result.claims.get("realm_access"),
            Some(&json!({"roles": ["admin", "user"]}))
        );
        assert!(result.additional_audiences.is_empty());
    }

    #[test]
    fn test_context_organization_with_no_orgs() {
        let context = MapperContext {
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![],
        };

        assert!(context.organizations.is_empty());
    }

    #[test]
    fn test_context_organization_fields_accessible() {
        use ferriskey_organization::OrganizationId;

        let org_id = OrganizationId::new(Uuid::new_v4());
        let mut attributes = HashMap::new();
        attributes.insert("department".to_string(), "engineering".to_string());
        attributes.insert("tier".to_string(), "enterprise".to_string());

        let org = ContextOrganization {
            id: org_id,
            name: "Acme Corp".to_string(),
            alias: "acme".to_string(),
            domain: Some("acme.com".to_string()),
            attributes,
        };

        let context = MapperContext {
            user_id: Uuid::new_v4(),
            username: "alice".to_string(),
            email: "alice@acme.com".to_string(),
            email_verified: true,
            firstname: "Alice".to_string(),
            lastname: "Smith".to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![org],
        };

        assert_eq!(context.organizations.len(), 1);
        let o = &context.organizations[0];
        assert_eq!(o.id, org_id);
        assert_eq!(o.name, "Acme Corp");
        assert_eq!(o.alias, "acme");
        assert_eq!(o.domain.as_deref(), Some("acme.com"));
        assert_eq!(
            o.attributes.get("department").map(String::as_str),
            Some("engineering")
        );
        assert_eq!(
            o.attributes.get("tier").map(String::as_str),
            Some("enterprise")
        );
    }

    #[test]
    fn test_context_supports_multiple_organizations() {
        use ferriskey_organization::OrganizationId;

        let orgs = vec![
            ContextOrganization {
                id: OrganizationId::new(Uuid::new_v4()),
                name: "Org A".to_string(),
                alias: "org-a".to_string(),
                domain: None,
                attributes: HashMap::new(),
            },
            ContextOrganization {
                id: OrganizationId::new(Uuid::new_v4()),
                name: "Org B".to_string(),
                alias: "org-b".to_string(),
                domain: Some("orgb.io".to_string()),
                attributes: HashMap::new(),
            },
        ];

        let context = MapperContext {
            user_id: Uuid::new_v4(),
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
            email_verified: false,
            firstname: "Bob".to_string(),
            lastname: "Jones".to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: orgs,
        };

        assert_eq!(context.organizations.len(), 2);
        assert_eq!(context.organizations[0].alias, "org-a");
        assert_eq!(context.organizations[1].alias, "org-b");
        assert!(context.organizations[1].domain.is_some());
    }
}
