use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects the organizations the user belongs to as a JSON object keyed by alias.
/// Mapper type: `oidc-organization-membership-mapper`
///
/// Each key is the organization alias; each value is an object containing the
/// stable fields for that organization.  The alias-keyed structure makes it
/// trivial for API consumers to look up a specific organization without scanning
/// an array.
///
/// The base fields (`id`, `name`, `alias`) are always present.
/// `domain` and `attributes` are opt-in via config so that tokens stay compact
/// by default.
///
/// ## Config
///
/// ```json
/// {
///   "claim.name":         "organizations",
///   "include.domain":     "false",
///   "include.attributes": "false",
///   "access.token.claim": "true",
///   "id.token.claim":     "true"
/// }
/// ```
///
/// ## Output shape (default)
///
/// ```json
/// {
///   "organizations": {
///     "acme": { "id": "<uuid>", "name": "Acme Corp", "alias": "acme" },
///     "beta": { "id": "<uuid>", "name": "Beta Ltd",  "alias": "beta" }
///   }
/// }
/// ```
///
/// ## Output shape (`include.domain` + `include.attributes` enabled)
///
/// ```json
/// {
///   "organizations": {
///     "acme": {
///       "id":         "<uuid>",
///       "name":       "Acme Corp",
///       "alias":      "acme",
///       "domain":     "acme.com",
///       "attributes": { "tier": "enterprise" }
///     }
///   }
/// }
/// ```
///
/// When the user belongs to no organizations the claim is set to an empty object `{}`.
#[derive(Debug)]
pub struct OrgMembershipMapper;

fn bool_config(config: &Value, key: &str) -> bool {
    config
        .get(key)
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::String(s) => Some(s == "true"),
            _ => None,
        })
        .unwrap_or(false)
}

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

        let include_domain = bool_config(config, "include.domain");
        let include_attributes = bool_config(config, "include.attributes");

        let map: serde_json::Map<String, Value> = context
            .organizations
            .iter()
            .map(|org| {
                let mut obj = serde_json::Map::new();
                obj.insert(
                    "id".to_string(),
                    Value::String(org.id.as_uuid().to_string()),
                );
                obj.insert("name".to_string(), Value::String(org.name.clone()));
                obj.insert("alias".to_string(), Value::String(org.alias.clone()));

                if include_domain {
                    obj.insert(
                        "domain".to_string(),
                        org.domain
                            .as_deref()
                            .map(|d| Value::String(d.to_string()))
                            .unwrap_or(Value::Null),
                    );
                }

                if include_attributes {
                    let attrs: serde_json::Map<String, Value> = org
                        .attributes
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                        .collect();
                    obj.insert("attributes".to_string(), Value::Object(attrs));
                }

                (org.alias.clone(), Value::Object(obj))
            })
            .collect();

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, Value::Object(map));
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

    fn org_full(
        name: &str,
        alias: &str,
        domain: Option<&str>,
        attrs: &[(&str, &str)],
    ) -> ContextOrganization {
        ContextOrganization {
            id: OrganizationId::new(Uuid::new_v4()),
            name: name.to_string(),
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
    fn emits_empty_object_when_user_has_no_orgs() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let result = mapper
            .execute(&config, &base_context(vec![]), TokenType::AccessToken)
            .unwrap();

        assert_eq!(result.claims.get("organizations"), Some(&json!({})));
    }

    #[test]
    fn emits_orgs_keyed_by_alias() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![org("Acme Corp", "acme"), org("Beta Ltd", "beta")];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let map = result.claims["organizations"].as_object().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(map["acme"]["name"], json!("Acme Corp"));
        assert_eq!(map["acme"]["alias"], json!("acme"));
        assert!(map["acme"]["id"].is_string());
        assert_eq!(map["beta"]["name"], json!("Beta Ltd"));
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
        assert_eq!(result.claims["user"]["orgs"]["x"]["alias"], json!("x"));
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
    fn domain_absent_by_default() {
        let mapper = OrgMembershipMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![org_full("Acme", "acme", Some("acme.com"), &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let entry = &result.claims["organizations"]["acme"];
        assert!(entry.get("domain").is_none());
        assert!(entry.get("attributes").is_none());
    }

    #[test]
    fn includes_domain_when_flag_enabled() {
        let mapper = OrgMembershipMapper;
        let config = json!({
            "include.domain": "true",
            "access.token.claim": "true",
        });

        let orgs = vec![
            org_full("Acme", "acme", Some("acme.com"), &[]),
            org_full("Beta", "beta", None, &[]),
        ];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(
            result.claims["organizations"]["acme"]["domain"],
            json!("acme.com")
        );
        assert_eq!(
            result.claims["organizations"]["beta"]["domain"],
            json!(null)
        );
    }

    #[test]
    fn includes_attributes_when_flag_enabled() {
        let mapper = OrgMembershipMapper;
        let config = json!({
            "include.attributes": "true",
            "access.token.claim": "true",
        });

        let orgs = vec![
            org_full(
                "Acme",
                "acme",
                None,
                &[("tier", "enterprise"), ("region", "eu")],
            ),
            org_full("Beta", "beta", None, &[]),
        ];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(
            result.claims["organizations"]["acme"]["attributes"]["tier"],
            json!("enterprise")
        );
        assert_eq!(
            result.claims["organizations"]["acme"]["attributes"]["region"],
            json!("eu")
        );
        assert!(
            result.claims["organizations"]["beta"]["attributes"]
                .as_object()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn all_orgs_emitted_for_multi_org_user() {
        let mapper = OrgMembershipMapper;
        let config = json!({
            "include.domain": "true",
            "include.attributes": "true",
            "access.token.claim": "true",
        });

        let orgs = vec![
            org_full("Acme", "acme", Some("acme.com"), &[("tier", "enterprise")]),
            org_full("Beta", "beta", Some("beta.io"), &[("tier", "starter")]),
            org_full("Gamma", "gamma", None, &[]),
        ];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let map = result.claims["organizations"].as_object().unwrap();
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("acme"));
        assert!(map.contains_key("beta"));
        assert!(map.contains_key("gamma"));
        assert_eq!(map["acme"]["attributes"]["tier"], json!("enterprise"));
        assert_eq!(map["beta"]["domain"], json!("beta.io"));
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

        assert_eq!(
            result.claims["organizations"]["stable"]["id"],
            json!("11111111-1111-1111-1111-111111111111")
        );
    }
}
