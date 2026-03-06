use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Maps user realm roles to token claims.
/// Similar to Keycloak's `oidc-usermodel-realm-role-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "claim.name": "realm_access.roles",
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
#[derive(Debug)]
pub struct UserRealmRoleMapper;

impl UserRealmRoleMapper {
    pub fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        let claim_name = config
            .get("claim.name")
            .and_then(|v| v.as_str())
            .unwrap_or("realm_access.roles");

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let mut output = MapperOutput::default();

        // Convert realm roles to JSON array
        let roles_value = Value::Array(
            context
                .realm_roles
                .iter()
                .map(|role| Value::String(role.clone()))
                .collect(),
        );

        set_claim_at_path(&mut output.claims, claim_name, roles_value);

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
            realm_roles: vec!["admin".to_string(), "user".to_string()],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_maps_realm_roles() {
        let mapper = UserRealmRoleMapper;
        let config = json!({
            "claim.name": "realm_access.roles",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("realm_access"),
            Some(&json!({"roles": ["admin", "user"]}))
        );
    }

    #[test]
    fn test_maps_realm_roles_to_custom_path() {
        let mapper = UserRealmRoleMapper;
        let config = json!({
            "claim.name": "custom.roles",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("custom"),
            Some(&json!({"roles": ["admin", "user"]}))
        );
    }

    #[test]
    fn test_empty_roles() {
        let mut context = test_context();
        context.realm_roles = vec![];

        let mapper = UserRealmRoleMapper;
        let config = json!({
            "claim.name": "realm_access.roles",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &context, TokenType::AccessToken)
            .unwrap();
        assert_eq!(
            result.claims.get("realm_access"),
            Some(&json!({"roles": []}))
        );
    }

    #[test]
    fn test_skips_when_token_type_disabled() {
        let mapper = UserRealmRoleMapper;
        let config = json!({
            "claim.name": "realm_access.roles",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.claims.is_empty());
    }
}
