use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{MapperContext, MapperOutput, TokenType, should_apply_to_token};

/// Adds entries to the audience (`aud`) claim.
/// Similar to Keycloak's `oidc-audience-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "included.client.audience": "another-client",
///   "access.token.claim": "true",
///   "id.token.claim": "false"
/// }
/// ```
#[derive(Debug)]
pub struct AudienceMapper;

impl AudienceMapper {
    pub fn execute(
        &self,
        config: &Value,
        _context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        let audience = config
            .get("included.client.audience")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        if audience.is_empty() {
            return Ok(MapperOutput::default());
        }

        Ok(MapperOutput {
            claims: Default::default(),
            additional_audiences: vec![audience.to_string()],
        })
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
    fn test_adds_audience() {
        let mapper = AudienceMapper;
        let config = json!({
            "included.client.audience": "backend-service",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(result.additional_audiences, vec!["backend-service"]);
        assert!(result.claims.is_empty());
    }

    #[test]
    fn test_empty_audience_skipped() {
        let mapper = AudienceMapper;
        let config = json!({
            "included.client.audience": "",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.additional_audiences.is_empty());
    }

    #[test]
    fn test_skips_when_token_type_disabled() {
        let mapper = AudienceMapper;
        let config = json!({
            "included.client.audience": "backend-service",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(&config, &test_context(), TokenType::AccessToken)
            .unwrap();
        assert!(result.additional_audiences.is_empty());
    }
}
