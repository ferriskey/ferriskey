use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Adds a hardcoded/static value as a claim.
/// Similar to Keycloak's `oidc-hardcoded-claim-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "claim.name": "my_custom_claim",
///   "claim.value": "static_value",
///   "jsonType.label": "String",
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
#[derive(Debug)]
pub struct HardcodedClaimMapper;

impl HardcodedClaimMapper {
    pub fn execute(
        &self,
        config: &Value,
        _context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        let claim_name = config
            .get("claim.name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let claim_value = config
            .get("claim.value")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let json_type = config
            .get("jsonType.label")
            .and_then(|v| v.as_str())
            .unwrap_or("String");

        let value = match json_type {
            "boolean" | "Boolean" => Value::Bool(claim_value == "true"),
            "int" | "Integer" => claim_value
                .parse::<i64>()
                .map(|n| Value::Number(n.into()))
                .unwrap_or_else(|_| Value::String(claim_value.to_string())),
            "long" | "Long" => claim_value
                .parse::<i64>()
                .map(|n| Value::Number(n.into()))
                .unwrap_or_else(|_| Value::String(claim_value.to_string())),
            "JSON" => serde_json::from_str(claim_value)
                .unwrap_or_else(|_| Value::String(claim_value.to_string())),
            _ => Value::String(claim_value.to_string()),
        };

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, value);

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use serde_json::json;
    use uuid::Uuid;

    use crate::domain::realm::entities::RealmId;

    fn test_context() -> MapperContext {
        MapperContext {
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
        }
    }

    #[test]
    fn test_string_claim() {
        let mapper = HardcodedClaimMapper;
        let config = json!({
            "claim.name": "custom_claim",
            "claim.value": "hello",
            "jsonType.label": "String",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("custom_claim"), Some(&json!("hello")));
    }

    #[test]
    fn test_boolean_claim() {
        let mapper = HardcodedClaimMapper;
        let config = json!({
            "claim.name": "is_admin",
            "claim.value": "true",
            "jsonType.label": "boolean",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("is_admin"), Some(&json!(true)));
    }

    #[test]
    fn test_integer_claim() {
        let mapper = HardcodedClaimMapper;
        let config = json!({
            "claim.name": "level",
            "claim.value": "42",
            "jsonType.label": "int",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("level"), Some(&json!(42)));
    }

    #[test]
    fn test_json_claim() {
        let mapper = HardcodedClaimMapper;
        let config = json!({
            "claim.name": "metadata",
            "claim.value": r#"{"key": "value"}"#,
            "jsonType.label": "JSON",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("metadata"),
            Some(&json!({"key": "value"}))
        );
    }

    #[test]
    fn test_empty_claim_name_skipped() {
        let mapper = HardcodedClaimMapper;
        let config = json!({
            "claim.name": "",
            "claim.value": "hello",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
    }
}
