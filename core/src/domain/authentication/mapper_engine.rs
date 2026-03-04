use std::collections::HashMap;

use ferriskey_aegis::entities::ProtocolMapper;
use serde_json::Value;
use uuid::Uuid;

use crate::domain::{common::entities::app_errors::CoreError, realm::entities::RealmId};

use super::mappers::{
    audience_mapper::AudienceMapper, hardcoded_claim_mapper::HardcodedClaimMapper,
    user_attribute_mapper::UserAttributeMapper, user_property_mapper::UserPropertyMapper,
};

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
    pub client_id: String,
    pub client_uuid: Uuid,
    pub realm_name: String,
    pub realm_id: RealmId,
    pub user_attributes: HashMap<String, Value>,
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

/// The core abstraction: takes a mapper config + context, returns claims to merge.
pub trait ProtocolMapperExecutor: Send + Sync {
    fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError>;
}

/// Registry-based engine that dispatches protocol mappers to their executor.
pub struct MapperEngine {
    executors: HashMap<String, Box<dyn ProtocolMapperExecutor>>,
}

impl std::fmt::Debug for MapperEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapperEngine")
            .field("executors", &self.executors.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl MapperEngine {
    pub fn new() -> Self {
        let mut executors: HashMap<String, Box<dyn ProtocolMapperExecutor>> = HashMap::new();
        executors.insert(
            "oidc-usermodel-property-mapper".to_string(),
            Box::new(UserPropertyMapper),
        );
        executors.insert(
            "oidc-hardcoded-claim-mapper".to_string(),
            Box::new(HardcodedClaimMapper),
        );
        executors.insert("oidc-audience-mapper".to_string(), Box::new(AudienceMapper));
        executors.insert(
            "oidc-usermodel-attribute-mapper".to_string(),
            Box::new(UserAttributeMapper),
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
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
        };

        // Empty mappers should produce empty output
        let result = engine
            .apply_mappers(&[], &context, TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
        assert!(result.additional_audiences.is_empty());
    }
}
