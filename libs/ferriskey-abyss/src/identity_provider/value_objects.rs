use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;

/// Request DTO for creating a new identity provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdentityProviderRequest {
    pub realm_id: RealmId,
    pub alias: String,
    pub provider_id: String,
    pub enabled: bool,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: bool,
    pub add_read_token_role_on_create: bool,
    pub trust_email: bool,
    pub link_only: bool,
    pub config: JsonValue,
}

/// Request DTO for updating an existing identity provider
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateIdentityProviderRequest {
    pub enabled: Option<bool>,
    pub display_name: Option<String>,
    pub first_broker_login_flow_alias: Option<String>,
    pub post_broker_login_flow_alias: Option<String>,
    pub store_token: Option<bool>,
    pub add_read_token_role_on_create: Option<bool>,
    pub trust_email: Option<bool>,
    pub link_only: Option<bool>,
    pub config: Option<JsonValue>,
    pub use_pkce: Option<bool>,
}

impl UpdateIdentityProviderRequest {
    pub fn resolve_config(&self, stored: &JsonValue) -> Result<Option<JsonValue>, CoreError> {
        let Some(use_pkce) = self.use_pkce else {
            return Ok(self.config.clone());
        };

        let mut merged = match self.config.as_ref().unwrap_or(stored) {
            JsonValue::Object(map) => map.clone(),
            JsonValue::Null => serde_json::Map::new(),
            _ => {
                return Err(CoreError::InvalidProviderConfiguration(
                    "Provider configuration must be a JSON object".to_string(),
                ));
            }
        };

        merged.insert("use_pkce".to_string(), JsonValue::Bool(use_pkce));

        Ok(Some(JsonValue::Object(merged)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity_provider::broker::OAuthProviderConfig;
    use serde_json::json;

    fn stored_config() -> JsonValue {
        json!({
            "client_id": "ferriskey",
            "client_secret": "s3cr3t",
            "authorization_url": "https://idp.example/auth",
            "token_url": "https://idp.example/token",
            "scopes": "openid email profile"
        })
    }

    #[test]
    fn enabling_pkce_keeps_every_other_config_key() {
        let request = UpdateIdentityProviderRequest {
            use_pkce: Some(true),
            ..Default::default()
        };

        let resolved = request
            .resolve_config(&stored_config())
            .expect("resolving a stored object config must succeed")
            .expect("use_pkce alone must still write the config column");

        let mut expected = stored_config();
        expected["use_pkce"] = json!(true);
        assert_eq!(resolved, expected);
    }

    #[test]
    fn disabling_pkce_overwrites_the_stored_flag() {
        let mut stored = stored_config();
        stored["use_pkce"] = json!(true);
        let request = UpdateIdentityProviderRequest {
            use_pkce: Some(false),
            ..Default::default()
        };

        let resolved = request
            .resolve_config(&stored)
            .expect("resolving a stored object config must succeed")
            .expect("use_pkce alone must still write the config column");

        assert_eq!(resolved["use_pkce"], json!(false));
        assert_eq!(resolved["client_secret"], json!("s3cr3t"));
    }

    #[test]
    fn an_update_touching_neither_field_leaves_the_config_column_alone() {
        let request = UpdateIdentityProviderRequest {
            enabled: Some(false),
            ..Default::default()
        };

        assert_eq!(
            request
                .resolve_config(&stored_config())
                .expect("resolving a stored object config must succeed"),
            None
        );
    }

    #[test]
    fn an_explicit_config_is_the_base_the_flag_is_merged_into() {
        let request = UpdateIdentityProviderRequest {
            config: Some(json!({ "client_id": "replaced" })),
            use_pkce: Some(true),
            ..Default::default()
        };

        let resolved = request
            .resolve_config(&stored_config())
            .expect("resolving a stored object config must succeed")
            .expect("an explicit config must be written");

        assert_eq!(
            resolved,
            json!({ "client_id": "replaced", "use_pkce": true })
        );
    }

    #[test]
    fn a_missing_config_becomes_an_object_holding_only_the_flag() {
        let request = UpdateIdentityProviderRequest {
            use_pkce: Some(true),
            ..Default::default()
        };

        let resolved = request
            .resolve_config(&JsonValue::Null)
            .expect("a null stored config has nothing to lose")
            .expect("use_pkce alone must still write the config column");

        assert_eq!(resolved, json!({ "use_pkce": true }));
    }

    #[test]
    fn the_merged_flag_is_the_one_the_broker_reads() {
        let request = UpdateIdentityProviderRequest {
            use_pkce: Some(true),
            ..Default::default()
        };

        let merged = request
            .resolve_config(&stored_config())
            .expect("resolving a stored object config must succeed")
            .expect("use_pkce alone must still write the config column");

        let config = OAuthProviderConfig::try_from(merged)
            .expect("the merged config must still parse as an OAuth provider config");

        assert_eq!(config.use_pkce, Some(true));
    }

    #[test]
    fn a_non_object_config_is_refused_rather_than_flattened() {
        let request = UpdateIdentityProviderRequest {
            use_pkce: Some(true),
            ..Default::default()
        };

        let error = request
            .resolve_config(&json!(["not", "an", "object"]))
            .expect_err("a non-object config must not be silently replaced");

        assert!(matches!(error, CoreError::InvalidProviderConfiguration(_)));
    }
}
