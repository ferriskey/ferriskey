use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects the list of organizations the user belongs to as a JSON array of objects.
/// Mapper type: `oidc-organization-membership-mapper`
///
/// Each element in the array contains the stable identifiers of the organization.
/// Attributes are deliberately excluded from this mapper; use
/// `oidc-organization-detail-mapper` when richer per-org data is needed.
///
/// ## Config
///
/// ```json
/// {
///   "claim.name": "organizations",
///   "access.token.claim": "true",
///   "id.token.claim": "true"
/// }
/// ```
///
/// ## Output shape
///
/// ```json
/// {
///   "organizations": [
///     { "id": "<uuid>", "name": "Acme Corp", "alias": "acme" }
///   ]
/// }
/// ```
///
/// When the user belongs to no organizations the claim is set to an empty array.
#[derive(Debug)]
pub struct OrgMembershipMapper;

impl OrgMembershipMapper {
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
            .unwrap_or("organizations");

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let entries: Vec<Value> = context
            .organizations
            .iter()
            .map(|org| {
                serde_json::json!({
                    "id": org.id.as_uuid().to_string(),
                    "name": org.name,
                    "alias": org.alias,
                })
            })
            .collect();

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, Value::Array(entries));
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use ferriskey_organization::OrganizationId;
    use serde_json::json;
    use uuid::Uuid;

    use crate::domain::{
        authentication::mapper_engine::ContextOrganization, realm::entities::RealmId,
    };

    fn org(name: &str, alias: &str) -> ContextOrganization {
        ContextOrganization {
            id: OrganizationId::new(Uuid::new_v4()),
            name: name.to_string(),
            alias: alias.to_string(),
            domain: None,
            attributes: HashMap::new(),
        }
    }

    fn base_context(organizations: Vec<ContextOrganization>) -> MapperContext {
        MapperContext {
            user_id: Uuid::new_v4(),
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
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
            organizations,
        }
    }

    #[test]
    fn emits_empty_array_when_user_has_no_orgs() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let result = mapper
            .execute(&config, &base_context(vec![]), TokenType::AccessToken)
            .unwrap();

        assert_eq!(result.claims.get("organizations"), Some(&json!([])));
    }

    #[test]
    fn emits_org_entries_with_id_name_alias() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![org("Acme Corp", "acme"), org("Beta Ltd", "beta")];
        let context = base_context(orgs.clone());

        let result = mapper
            .execute(&config, &context, TokenType::AccessToken)
            .unwrap();

        let list = result
            .claims
            .get("organizations")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(list.len(), 2);

        assert_eq!(list[0]["name"], json!("Acme Corp"));
        assert_eq!(list[0]["alias"], json!("acme"));
        assert!(list[0]["id"].is_string());

        assert_eq!(list[1]["name"], json!("Beta Ltd"));
        assert_eq!(list[1]["alias"], json!("beta"));
    }

    #[test]
    fn uses_custom_claim_name() {
        let mapper = OrgMembershipMapper;
        let config = json!({
            "claim.name": "user.orgs",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![org("X", "x")]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert!(result.claims.contains_key("user"));
        assert_eq!(result.claims["user"]["orgs"][0]["alias"], json!("x"));
    }

    #[test]
    fn skips_when_access_token_disabled() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "false" });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![org("Acme", "acme")]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn applies_to_id_token_when_enabled() {
        let mapper = OrgMembershipMapper;
        let config = json!({
            "id.token.claim": "true",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![org("Acme", "acme")]),
                TokenType::IdToken,
            )
            .unwrap();

        assert!(result.claims.contains_key("organizations"));
    }

    #[test]
    fn id_is_stable_uuid_string_for_given_org() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let fixed_id =
            OrganizationId::new(Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap());
        let org = ContextOrganization {
            id: fixed_id,
            name: "Stable".to_string(),
            alias: "stable".to_string(),
            domain: None,
            attributes: HashMap::new(),
        };

        let result = mapper
            .execute(&config, &base_context(vec![org]), TokenType::AccessToken)
            .unwrap();

        let list = result.claims["organizations"].as_array().unwrap();
        assert_eq!(list[0]["id"], json!("11111111-1111-1111-1111-111111111111"));
    }
}
