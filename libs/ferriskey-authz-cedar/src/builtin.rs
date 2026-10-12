use cedar_policy::SchemaFragment;
use ferriskey_domain::common::app_errors::PolicyIssue;

use crate::issues::issues_from;

const BUILTIN_SCHEMA: &str = r#"namespace FerrisKey {
  entity Role = {
    name: String,
  };

  entity Organization = {
    alias: String,
    name: String,
  };

  entity Group in [Group, Organization] = {
    name: String,
  };

  entity User in [Role, Group, Organization] = {
    username: String,
    email?: String,
    email_verified: Bool,
    enabled: Bool,
  } tags String;

  entity Client in [Role] = {
    client_id: String,
  };
}
"#;

pub(crate) fn builtin_fragment() -> Result<SchemaFragment, Vec<PolicyIssue>> {
    SchemaFragment::from_cedarschema_str(BUILTIN_SCHEMA)
        .map(|(fragment, _warnings)| fragment)
        .map_err(|e| issues_from(None, &e, BUILTIN_SCHEMA))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_schema_parses_on_its_own() {
        let fragment = builtin_fragment().expect("builtin schema");
        let namespaces: Vec<String> = fragment
            .namespaces()
            .map(|ns| ns.map(|n| n.to_string()).unwrap_or_default())
            .collect();
        assert_eq!(namespaces, ["FerrisKey"]);
    }
}
