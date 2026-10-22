use cedar_policy::SchemaFragment;
use ferriskey_domain::common::app_errors::PolicyIssue;

use crate::issues::issues_from;

pub const BUILTIN_SCHEMA: &str = r#"namespace FerrisKey {
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
    roles?: Set<String>,
    organisation?: String,
  } tags String;

  entity Client in [Role] = {
    client_id: String,
  };

  entity Agent in [Role] = {
    client_id: String,
  };

  entity McpServer = {
    name: String,
  };

  entity Tool in [McpServer] = {
    server: String,
    tool: String,
    annotations?: {
      readOnlyHint?: Bool,
      destructiveHint?: Bool,
      idempotentHint?: Bool,
      openWorldHint?: Bool,
    },
  };

  entity Resource in [McpServer] = {
    server: String,
    uri: String,
  };

  entity Prompt in [McpServer] = {
    server: String,
    name: String,
  };

  type AgentContext = {
    agent: String,
    delegated: Bool,
    act?: {
      sub?: String,
      client_id?: String,
      act?: {
        sub?: String,
        client_id?: String,
        act?: {
          sub?: String,
          client_id?: String,
        },
      },
    },
    time: {
      weekday: Long,
      hour: Long,
      iso: String,
    },
  };

  action call_tool appliesTo {
    principal: [User, Agent, Client],
    resource: [Tool],
    context: AgentContext,
  };

  action read_resource appliesTo {
    principal: [User, Agent, Client],
    resource: [Resource],
    context: AgentContext,
  };

  action get_prompt appliesTo {
    principal: [User, Agent, Client],
    resource: [Prompt],
    context: AgentContext,
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
    fn the_builtin_schema_declares_the_agent_types_and_actions() {
        let schema =
            cedar_policy::Schema::from_schema_fragments([builtin_fragment().expect("fragment")])
                .expect("schema");
        let types: Vec<String> = schema.entity_types().map(|t| t.to_string()).collect();
        for expected in [
            "FerrisKey::Agent",
            "FerrisKey::Tool",
            "FerrisKey::McpServer",
            "FerrisKey::Resource",
            "FerrisKey::Prompt",
        ] {
            assert!(types.iter().any(|t| t == expected), "{expected}");
        }
        let actions: Vec<String> = schema.actions().map(|a| a.to_string()).collect();
        for expected in ["call_tool", "read_resource", "get_prompt"] {
            assert!(
                actions
                    .iter()
                    .any(|a| a == &format!("FerrisKey::Action::\"{expected}\"")),
                "{expected}: {actions:?}"
            );
        }
    }

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
