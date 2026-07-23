use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects the roles a user holds *within the scope of each organization* as a JSON object keyed
/// by organization alias. Mapper type: `oidc-organization-role-mapper`.
///
/// The org-scoped roles come from two sources, merged upstream in token assembly:
/// roles assigned directly to the membership, and roles inherited from the user's groups in
/// that organization. Realm roles land under `roles`; client roles under
/// `clients.<client_id>.roles`.
///
/// This mapper writes into the same `organizations` claim as the membership mapper, so when both
/// are enabled the objects deep-merge (membership contributes `id`/`name`/`alias`, this one
/// contributes `roles`/`clients`). The alias is the claim key; the membership mapper's `id` field
/// is the stable identifier consumers should match on if the alias may be renamed.
///
/// ## Config
///
/// ```json
/// {
///   "claim.name":         "organizations",
///   "access.token.claim": "true",
///   "id.token.claim":     "true"
/// }
/// ```
///
/// ## Output shape
///
/// ```json
/// {
///   "organizations": {
///     "acme": {
///       "roles": ["org-admin"],
///       "clients": { "billing-api": { "roles": ["viewer"] } }
///     }
///   }
/// }
/// ```
///
/// Organizations with no scoped roles are omitted. When the user has no org-scoped roles at all
/// the claim is set to an empty object `{}`.
#[derive(Debug)]
pub struct UserOrganizationRoleMapper;

impl UserOrganizationRoleMapper {
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

        let mut map = serde_json::Map::new();
        for org in &context.organizations {
            if org.roles.is_empty() && org.client_roles.is_empty() {
                continue;
            }

            let mut obj = serde_json::Map::new();

            if !org.roles.is_empty() {
                obj.insert(
                    "roles".to_string(),
                    Value::Array(org.roles.iter().map(|r| Value::String(r.clone())).collect()),
                );
            }

            if !org.client_roles.is_empty() {
                let mut clients = serde_json::Map::new();
                for (client_id, roles) in &org.client_roles {
                    let mut client_obj = serde_json::Map::new();
                    client_obj.insert(
                        "roles".to_string(),
                        Value::Array(roles.iter().map(|r| Value::String(r.clone())).collect()),
                    );
                    clients.insert(client_id.clone(), Value::Object(client_obj));
                }
                obj.insert("clients".to_string(), Value::Object(clients));
            }

            map.insert(org.alias.clone(), Value::Object(obj));
        }

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

    fn org_with_roles(
        alias: &str,
        roles: &[&str],
        client_roles: &[(&str, &[&str])],
    ) -> ContextOrganization {
        ContextOrganization {
            id: OrganizationId::new(Uuid::new_v4()),
            name: format!("{alias}-name"),
            alias: alias.to_string(),
            domain: None,
            attributes: HashMap::new(),
            roles: roles.iter().map(|r| r.to_string()).collect(),
            client_roles: client_roles
                .iter()
                .map(|(cid, rs)| (cid.to_string(), rs.iter().map(|r| r.to_string()).collect()))
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
            groups: vec![],
        }
    }

    #[test]
    fn emits_realm_roles_keyed_by_alias() {
        let mapper = UserOrganizationRoleMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![org_with_roles("acme", &["org-admin"], &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(
            result.claims["organizations"]["acme"]["roles"],
            json!(["org-admin"])
        );
    }

    #[test]
    fn emits_client_roles_nested_under_clients() {
        let mapper = UserOrganizationRoleMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![org_with_roles("acme", &[], &[("billing-api", &["viewer"])])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert_eq!(
            result.claims["organizations"]["acme"]["clients"]["billing-api"]["roles"],
            json!(["viewer"])
        );
    }

    #[test]
    fn omits_orgs_without_scoped_roles() {
        let mapper = UserOrganizationRoleMapper;
        let config = json!({ "access.token.claim": "true" });

        let orgs = vec![
            org_with_roles("acme", &["admin"], &[]),
            org_with_roles("beta", &[], &[]),
        ];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        let map = result.claims["organizations"].as_object().unwrap();
        assert!(map.contains_key("acme"));
        assert!(!map.contains_key("beta"));
    }

    #[test]
    fn skips_when_access_token_disabled() {
        let mapper = UserOrganizationRoleMapper;
        let config = json!({ "access.token.claim": "false" });

        let orgs = vec![org_with_roles("acme", &["admin"], &[])];
        let result = mapper
            .execute(&config, &base_context(orgs), TokenType::AccessToken)
            .unwrap();

        assert!(result.claims.is_empty());
    }
}
