use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects detailed information about a single organization into the token.
/// Mapper type: `oidc-organization-detail-mapper`
///
/// When `organization.alias` is set the mapper only emits a claim if the user
/// belongs to that specific organization.  When omitted, the first organization
/// in the user's membership list is used.  If the user belongs to no matching
/// organization the mapper emits nothing.
///
/// ## Config
///
/// ```json
/// {
///   "claim.name":          "organization",
///   "organization.alias":  "acme",
///   "include.attributes":  "false",
///   "access.token.claim":  "true",
///   "id.token.claim":      "true"
/// }
/// ```
///
/// ## Output shape
///
/// ```json
/// {
///   "organization": {
///     "id":         "<uuid>",
///     "name":       "Acme Corp",
///     "alias":      "acme",
///     "domain":     "acme.com"
///   }
/// }
/// ```
///
/// When `include.attributes` is `"true"` an extra `"attributes"` key is added:
///
/// ```json
/// {
///   "organization": {
///     "id":    "<uuid>",
///     "name":  "Acme Corp",
///     "alias": "acme",
///     "domain": null,
///     "attributes": { "department": "engineering" }
///   }
/// }
/// ```
#[derive(Debug)]
pub struct OrgDetailMapper;

impl OrgDetailMapper {
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
            .unwrap_or("organization");

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let alias_filter = config
            .get("organization.alias")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let include_attributes = config
            .get("include.attributes")
            .and_then(|v| match v {
                Value::Bool(b) => Some(*b),
                Value::String(s) => Some(s == "true"),
                _ => None,
            })
            .unwrap_or(false);

        let org = match alias_filter {
            Some(alias) => context.organizations.iter().find(|o| o.alias == alias),
            None => context.organizations.first(),
        };

        let Some(org) = org else {
            return Ok(MapperOutput::default());
        };

        let mut obj = serde_json::Map::new();
        obj.insert(
            "id".to_string(),
            Value::String(org.id.as_uuid().to_string()),
        );
        obj.insert("name".to_string(), Value::String(org.name.clone()));
        obj.insert("alias".to_string(), Value::String(org.alias.clone()));
        obj.insert(
            "domain".to_string(),
            org.domain
                .as_deref()
                .map(|d| Value::String(d.to_string()))
                .unwrap_or(Value::Null),
        );

        if include_attributes {
            let attrs: serde_json::Map<String, Value> = org
                .attributes
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect();
            obj.insert("attributes".to_string(), Value::Object(attrs));
        }

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, Value::Object(obj));
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

    fn make_org(alias: &str, domain: Option<&str>, attrs: &[(&str, &str)]) -> ContextOrganization {
        ContextOrganization {
            id: OrganizationId::new(Uuid::new_v4()),
            name: format!("{alias}-name"),
            alias: alias.to_string(),
            domain: domain.map(|s| s.to_string()),
            attributes: attrs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
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
    fn emits_nothing_when_user_has_no_orgs() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "true" });

        let result = mapper
            .execute(&config, &base_context(vec![]), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn uses_first_org_when_no_alias_filter() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![make_org("alpha", None, &[]), make_org("beta", None, &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(result.claims["organization"]["alias"], json!("alpha"));
    }

    #[test]
    fn selects_org_by_alias_filter() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "organization.alias": "beta",
            "access.token.claim": "true",
        });

        let orgs = vec![make_org("alpha", None, &[]), make_org("beta", None, &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(result.claims["organization"]["alias"], json!("beta"));
    }

    #[test]
    fn emits_nothing_when_alias_filter_matches_no_org() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "organization.alias": "unknown",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![make_org("alpha", None, &[])]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn includes_id_name_alias_domain_fields() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![make_org("acme", Some("acme.com"), &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let org = &result.claims["organization"];
        assert!(org["id"].is_string());
        assert_eq!(org["name"], json!("acme-name"));
        assert_eq!(org["alias"], json!("acme"));
        assert_eq!(org["domain"], json!("acme.com"));
    }

    #[test]
    fn domain_is_null_when_absent() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "true" });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![make_org("nodomain", None, &[])]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert_eq!(result.claims["organization"]["domain"], json!(null));
    }

    #[test]
    fn attributes_absent_by_default() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![make_org("acme", None, &[("dept", "eng")])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims["organization"].get("attributes").is_none());
    }

    #[test]
    fn attributes_included_when_flag_is_true() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "include.attributes": "true",
            "access.token.claim": "true",
        });

        let orgs = vec![make_org(
            "acme",
            None,
            &[("dept", "engineering"), ("tier", "enterprise")],
        )];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let attrs = &result.claims["organization"]["attributes"];
        assert_eq!(attrs["dept"], json!("engineering"));
        assert_eq!(attrs["tier"], json!("enterprise"));
    }

    #[test]
    fn attributes_included_when_flag_is_bool_true() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "include.attributes": true,
            "access.token.claim": "true",
        });

        let orgs = vec![make_org("acme", None, &[("k", "v")])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims["organization"]["attributes"].is_object());
    }

    #[test]
    fn custom_claim_name_produces_nested_path() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "claim.name": "context.org",
            "access.token.claim": "true",
        });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![make_org("acme", None, &[])]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert!(result.claims.contains_key("context"));
        assert_eq!(result.claims["context"]["org"]["alias"], json!("acme"));
    }

    #[test]
    fn skips_when_access_token_disabled() {
        let mapper = OrgDetailMapper;
        let config = json!({ "access.token.claim": "false" });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![make_org("acme", None, &[])]),
                TokenType::AccessToken,
            )
            .unwrap();

        assert!(result.claims.is_empty());
    }

    #[test]
    fn applies_to_id_token_when_enabled() {
        let mapper = OrgDetailMapper;
        let config = json!({
            "id.token.claim": "true",
            "access.token.claim": "false",
        });

        let result = mapper
            .execute(
                &config,
                &base_context(vec![make_org("acme", None, &[])]),
                TokenType::IdToken,
            )
            .unwrap();

        assert!(result.claims.contains_key("organization"));
    }
}
