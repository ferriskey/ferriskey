use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{MapperContext, MapperOutput, TokenType, should_apply_to_token};

/// Maps client-scoped roles assigned to a user into `resource_access.<client_id>.roles`.
/// Similar to Keycloak's `oidc-usermodel-client-role-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "client.id": "backend",
///   "access.token.claim": "true",
///   "id.token.claim": "false"
/// }
/// ```
///
/// When `client.id` is empty or absent, roles from **all** clients are included.
/// Output shape (always injected under the top-level `resource_access` key):
/// ```json
/// {
///   "resource_access": {
///     "backend": { "roles": ["role1", "role2"] }
///   }
/// }
/// ```
#[derive(Debug)]
pub struct UserClientRoleMapper;

impl UserClientRoleMapper {
    pub fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        // Optional filter: only include roles for this specific client.
        let client_id_filter = config
            .get("client.id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let mut resource_access = serde_json::Map::new();

        for (client_id, roles) in &context.client_roles {
            if let Some(filter) = client_id_filter
                && client_id != filter
            {
                continue;
            }

            let roles_array =
                Value::Array(roles.iter().map(|r| Value::String(r.clone())).collect());

            let mut client_obj = serde_json::Map::new();
            client_obj.insert("roles".to_string(), roles_array);
            resource_access.insert(client_id.clone(), Value::Object(client_obj));
        }

        if resource_access.is_empty() {
            return Ok(MapperOutput::default());
        }

        let mut output = MapperOutput::default();
        output.claims.insert(
            "resource_access".to_string(),
            Value::Object(resource_access),
        );

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
        let mut client_roles = HashMap::new();
        client_roles.insert(
            "backend".to_string(),
            vec!["role1".to_string(), "role2".to_string()],
        );
        client_roles.insert("frontend".to_string(), vec!["viewer".to_string()]);

        MapperContext {
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            realm_roles: vec![],
            client_roles,
            client_id: "backend".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_injects_all_client_roles_when_no_filter() {
        let mapper = UserClientRoleMapper;
        let config = json!({
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();

        let resource_access = result.claims.get("resource_access").unwrap();
        assert_eq!(
            resource_access["backend"]["roles"],
            json!(["role1", "role2"])
        );
        assert_eq!(resource_access["frontend"]["roles"], json!(["viewer"]));
    }

    #[test]
    fn test_filters_to_specific_client() {
        let mapper = UserClientRoleMapper;
        let config = json!({
            "client.id": "backend",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();

        let resource_access = result.claims.get("resource_access").unwrap();
        assert_eq!(
            resource_access["backend"]["roles"],
            json!(["role1", "role2"])
        );
        // frontend must not appear
        assert!(resource_access.get("frontend").is_none());
    }

    #[test]
    fn test_empty_when_no_client_roles() {
        let mapper = UserClientRoleMapper;
        let config = json!({ "access.token.claim": "true" });

        let mut context = test_context();
        context.client_roles = HashMap::new();

        let result = mapper
            .execute(&config, &context, TokenType::AccessToken)
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_empty_when_filter_matches_no_client() {
        let mapper = UserClientRoleMapper;
        let config = json!({
            "client.id": "unknown-client",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_skips_when_token_type_disabled() {
        let mapper = UserClientRoleMapper;
        let config = json!({
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_applies_to_id_token_when_enabled() {
        let mapper = UserClientRoleMapper;
        let config = json!({
            "id.token.claim": "true",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::IdToken)
            .unwrap();

        assert!(result.claims.contains_key("resource_access"));
    }
}
