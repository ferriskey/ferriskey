use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Maps custom user attributes to token claims.
/// Similar to Keycloak's `oidc-usermodel-attribute-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "user.attribute": "department",
///   "claim.name": "department",
///   "jsonType.label": "String",
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
#[derive(Debug)]
pub struct UserAttributeMapper;

impl UserAttributeMapper {
    pub fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        let user_attribute = config
            .get("user.attribute")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let claim_name = config
            .get("claim.name")
            .and_then(|v| v.as_str())
            .unwrap_or(user_attribute);

        if claim_name.is_empty() || user_attribute.is_empty() {
            return Ok(MapperOutput::default());
        }

        let mut output = MapperOutput::default();

        if let Some(value) = context.user_attributes.get(user_attribute) {
            set_claim_at_path(&mut output.claims, claim_name, value.clone());
        }

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
        let mut user_attributes = HashMap::new();
        user_attributes.insert("department".to_string(), json!("engineering"));
        user_attributes.insert("level".to_string(), json!(5));

        MapperContext {
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
            user_attributes,
        }
    }

    #[test]
    fn test_maps_existing_attribute() {
        let mapper = UserAttributeMapper;
        let config = json!({
            "user.attribute": "department",
            "claim.name": "department",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("department"), Some(&json!("engineering")));
    }

    #[test]
    fn test_missing_attribute_produces_no_claim() {
        let mapper = UserAttributeMapper;
        let config = json!({
            "user.attribute": "nonexistent",
            "claim.name": "nonexistent",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_different_claim_name() {
        let mapper = UserAttributeMapper;
        let config = json!({
            "user.attribute": "department",
            "claim.name": "org.department",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("org"),
            Some(&json!({"department": "engineering"}))
        );
    }
}
