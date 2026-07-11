use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects the groups the user belongs to under `claim.name` (default `groups`).
/// Mapper type: `oidc-group-membership-mapper`
///
/// ## Membership mode (`membership`)
///
/// - `"effective"` (default): every group the user belongs to *plus all ancestors* — a member of
///   `/novotel/EN/leaf` gets all three paths. Reflects the hierarchy and matches how roles are
///   inherited in FerrisKey.
/// - `"direct"`: only groups the user is a *direct* member of (`["/novotel/EN/leaf"]`). Matches
///   Keycloak's group-membership mapper; relying parties reconstruct the hierarchy via path
///   prefixes if they need it. Keeps tokens smaller and discloses less org structure.
///
/// Each entry is the group's full path (`/parent/child`); set `full.path` to `false` to emit bare
/// group names instead.
///
/// ## Org prefix (`prefix.org`)
///
/// A FerrisKey token can carry groups from several organizations at once, and paths are relative
/// to their org — so `/finance` from two orgs collides. Set `prefix.org` to `true` to prepend the
/// group's org alias (`/acme/finance`), disambiguating multi-org tokens. Only applies to full
/// paths. Defaults to `false` (Keycloak-compatible).
///
/// ## Config
///
/// ```json
/// {
///   "claim.name":         "groups",
///   "membership":         "effective",
///   "full.path":          "true",
///   "prefix.org":         "false",
///   "access.token.claim": "true",
///   "id.token.claim":     "true"
/// }
/// ```
#[derive(Debug)]
pub struct GroupMembershipMapper;

fn bool_config(config: &Value, key: &str, default: bool) -> bool {
    config
        .get(key)
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::String(s) => Some(s == "true"),
            _ => None,
        })
        .unwrap_or(default)
}

impl GroupMembershipMapper {
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
            .unwrap_or("groups");

        if claim_name.is_empty() {
            return Ok(MapperOutput::default());
        }

        let full_path = bool_config(config, "full.path", true);

        // Default to effective (direct + ancestors); "direct" emits only direct memberships.
        let direct_only = config
            .get("membership")
            .and_then(|v| v.as_str())
            .map(|s| s.eq_ignore_ascii_case("direct"))
            .unwrap_or(false);

        // When enabled, prefix each full path with the group's org alias (`/acme/parent/child`)
        // to disambiguate groups from different organizations in a multi-org token. Only applies
        // to full paths — bare names (`full.path=false`) carry no path to prefix.
        let prefix_org = bool_config(config, "prefix.org", false) && full_path;

        let values: Vec<Value> = context
            .groups
            .iter()
            .filter(|group| !direct_only || group.direct)
            .map(|group| {
                if !full_path {
                    return Value::String(group.name.clone());
                }
                if prefix_org
                    && let Some(alias) = context
                        .organizations
                        .iter()
                        .find(|org| org.id.as_uuid() == group.organization_id)
                        .map(|org| org.alias.as_str())
                {
                    return Value::String(format!("/{}{}", alias, group.path));
                }
                Value::String(group.path.clone())
            })
            .collect();

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, Value::Array(values));
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

    use crate::domain::authentication::mapper_engine::{ContextGroup, ContextOrganization};
    use crate::domain::realm::entities::RealmId;

    fn org_id() -> Uuid {
        Uuid::from_u128(0xace)
    }

    fn group(path: &str, direct: bool) -> ContextGroup {
        let name = path.rsplit('/').next().unwrap_or(path).to_string();
        ContextGroup {
            id: Uuid::new_v4(),
            organization_id: org_id(),
            name,
            path: path.to_string(),
            direct,
        }
    }

    fn acme() -> ContextOrganization {
        ContextOrganization {
            id: OrganizationId::new(org_id()),
            name: "acme".to_string(),
            alias: "acme".to_string(),
            domain: None,
            attributes: HashMap::new(),
        }
    }

    /// User is a direct member of the leaf; the two ancestors come from the effective set.
    fn context() -> MapperContext {
        MapperContext {
            user_id: Uuid::new_v4(),
            username: "nathaelb".to_string(),
            email: "nathael@ferriskey.rs".to_string(),
            email_verified: true,
            firstname: "Nathael".to_string(),
            lastname: "Bonnal".to_string(),
            realm_roles: vec![],
            client_roles: HashMap::new(),
            client_id: "test".to_string(),
            client_uuid: Uuid::new_v4(),
            realm_name: "master".to_string(),
            realm_id: RealmId::new(Uuid::new_v4()),
            user_attributes: HashMap::new(),
            organizations: vec![acme()],
            groups: vec![
                group("/novotel", false),
                group("/novotel/EN", false),
                group("/novotel/EN/novotel-ld-01", true),
            ],
        }
    }

    fn groups_claim(output: &MapperOutput) -> Vec<String> {
        match output.claims.get("groups") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect(),
            _ => vec![],
        }
    }

    #[test]
    fn effective_mode_emits_direct_plus_ancestors() {
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "effective" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(
            groups_claim(&out),
            vec!["/novotel", "/novotel/EN", "/novotel/EN/novotel-ld-01"]
        );
    }

    #[test]
    fn defaults_to_effective_when_unset() {
        let out = GroupMembershipMapper
            .execute(&json!({}), &context(), TokenType::AccessToken)
            .unwrap();
        assert_eq!(groups_claim(&out).len(), 3);
    }

    #[test]
    fn direct_mode_emits_only_direct_membership() {
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["/novotel/EN/novotel-ld-01"]);
    }

    #[test]
    fn direct_mode_respects_full_path_false() {
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct", "full.path": "false" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["novotel-ld-01"]);
    }

    #[test]
    fn prefix_org_prepends_the_org_alias() {
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct", "prefix.org": "true" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["/acme/novotel/EN/novotel-ld-01"]);
    }

    #[test]
    fn prefix_org_is_off_by_default() {
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["/novotel/EN/novotel-ld-01"]);
    }

    #[test]
    fn prefix_org_ignored_without_full_path() {
        // No path to prefix when emitting bare names — prefix.org is a no-op.
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct", "prefix.org": "true", "full.path": "false" }),
                &context(),
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["novotel-ld-01"]);
    }

    #[test]
    fn prefix_org_falls_back_when_org_missing() {
        // Org not present in the context (edge case) → emit the unprefixed path rather than drop.
        let mut ctx = context();
        ctx.organizations.clear();
        let out = GroupMembershipMapper
            .execute(
                &json!({ "membership": "direct", "prefix.org": "true" }),
                &ctx,
                TokenType::AccessToken,
            )
            .unwrap();
        assert_eq!(groups_claim(&out), vec!["/novotel/EN/novotel-ld-01"]);
    }
}
