use serde_json::Value;

use crate::domain::common::entities::app_errors::CoreError;

use super::super::mapper_engine::{
    MapperContext, MapperOutput, TokenType, set_claim_at_path, should_apply_to_token,
};

/// Injects the groups the user effectively belongs to (recursively, including ancestors).
/// Mapper type: `oidc-group-membership-mapper`
///
/// Emits an array under `claim.name` (default `groups`). By default each entry is the group's
/// full path (`/parent/child`); set `full.path` to `false` to emit bare group names instead.
///
/// ## Config
///
/// ```json
/// {
///   "claim.name":         "groups",
///   "full.path":          "true",
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

        let values: Vec<Value> = context
            .groups
            .iter()
            .map(|group| {
                if full_path {
                    Value::String(group.path.clone())
                } else {
                    Value::String(group.name.clone())
                }
            })
            .collect();

        let mut output = MapperOutput::default();
        set_claim_at_path(&mut output.claims, claim_name, Value::Array(values));
        Ok(output)
    }
}
