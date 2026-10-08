use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Maps the user's first and last name, joined by a space, to the `name` claim.
/// Similar to Keycloak's `oidc-full-name-mapper`.
///
/// Expected config:
/// ```json
/// {
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
#[derive(Debug)]
pub struct FullNameMapper;

impl FullNameMapper {
    pub fn execute(
        &self,
        config: &Value,
        context: &MapperContext,
        token_type: TokenType,
    ) -> Result<MapperOutput, CoreError> {
        if !should_apply_to_token(config, token_type) {
            return Ok(MapperOutput::default());
        }

        let full_name = [context.firstname.trim(), context.lastname.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let mut output = MapperOutput::default();
        if !full_name.is_empty() {
            set_claim_at_path(&mut output.claims, "name", Value::String(full_name));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::domain::realm::entities::RealmId;

    fn context(firstname: &str, lastname: &str) -> MapperContext {
        MapperContext {
            user_id: Uuid::new_v4(),
            username: "test".to_string(),
            email: "test@test.com".to_string(),
            email_verified: true,
            firstname: firstname.to_string(),
            lastname: lastname.to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "my-client".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "test-realm".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![],
            groups: vec![],
        }
    }

    fn run(firstname: &str, lastname: &str, config: Value) -> MapperOutput {
        FullNameMapper
            .execute(&config, &context(firstname, lastname), TokenType::IdToken)
            .unwrap()
    }

    #[test]
    fn test_joins_first_and_last_name() {
        let output = run(" Test ", "User", json!({}));
        assert_eq!(output.claims.get("name"), Some(&json!("Test User")));
    }

    #[test]
    fn test_uses_single_part_when_other_is_empty() {
        assert_eq!(
            run("Test", "", json!({})).claims.get("name"),
            Some(&json!("Test"))
        );
        assert_eq!(
            run("", "User", json!({})).claims.get("name"),
            Some(&json!("User"))
        );
    }

    #[test]
    fn test_no_claim_when_both_names_are_empty() {
        assert!(run("", " ", json!({})).claims.is_empty());
    }

    #[test]
    fn test_respects_token_inclusion() {
        let output = run("Test", "User", json!({ "id.token.claim": "false" }));
        assert!(output.claims.is_empty());
    }
}
