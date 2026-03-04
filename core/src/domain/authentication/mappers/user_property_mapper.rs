use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Maps built-in user properties (username, email, firstname, lastname, email_verified)
/// to token claims. Similar to Keycloak's `oidc-usermodel-property-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "user.attribute": "username",
///   "claim.name": "preferred_username",
///   "jsonType.label": "String",
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
#[derive(Debug)]
pub struct UserPropertyMapper;

impl UserPropertyMapper {
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

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let value = match user_attribute {
            "username" => Some(Value::String(context.username.clone())),
            "email" => Some(Value::String(context.email.clone())),
            "firstName" | "firstname" => Some(Value::String(context.firstname.clone())),
            "lastName" | "lastname" => Some(Value::String(context.lastname.clone())),
            "emailVerified" | "email_verified" => Some(Value::Bool(context.email_verified)),
            _ => {
                tracing::debug!(
                    attribute = user_attribute,
                    "Unknown user property, skipping"
                );
                None
            }
        };

        let mut output = MapperOutput::default();
        if let Some(val) = value {
            let typed_value = cast_value(val, config);
            set_claim_at_path(&mut output.claims, claim_name, typed_value);
        }

        Ok(output)
    }
}

/// Cast a value based on `jsonType.label` config.
fn cast_value(value: Value, config: &Value) -> Value {
    let json_type = config
        .get("jsonType.label")
        .and_then(|v| v.as_str())
        .unwrap_or("String");

    match json_type {
        "boolean" | "Boolean" => match &value {
            Value::Bool(_) => value,
            Value::String(s) => Value::Bool(s == "true"),
            _ => value,
        },
        "int" | "Integer" | "long" | "Long" => match &value {
            Value::Number(_) => value,
            Value::String(s) => s
                .parse::<i64>()
                .map(|n| Value::Number(n.into()))
                .unwrap_or(value),
            _ => value,
        },
        _ => value, // Default to String, keep as-is
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
            username: "john.doe".to_string(),
            email: "john@example.com".to_string(),
            email_verified: true,
            firstname: "John".to_string(),
            lastname: "Doe".to_string(),
            realm_roles: vec!["admin".to_string()],
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_maps_username() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "username",
            "claim.name": "preferred_username",
            "jsonType.label": "String",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("preferred_username"),
            Some(&json!("john.doe"))
        );
    }

    #[test]
    fn test_maps_email() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "email",
            "claim.name": "email",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("email"), Some(&json!("john@example.com")));
    }

    #[test]
    fn test_maps_email_verified_as_boolean() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "emailVerified",
            "claim.name": "email_verified",
            "jsonType.label": "boolean",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("email_verified"), Some(&json!(true)));
    }

    #[test]
    fn test_skips_when_token_type_disabled() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "username",
            "claim.name": "preferred_username",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_maps_firstname() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "firstName",
            "claim.name": "given_name",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("given_name"), Some(&json!("John")));
    }

    #[test]
    fn test_maps_lastname() {
        let mapper = UserPropertyMapper;
        let config = json!({
            "user.attribute": "lastName",
            "claim.name": "family_name",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.claims.get("family_name"), Some(&json!("Doe")));
    }
}
