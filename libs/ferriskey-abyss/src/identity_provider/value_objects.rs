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
    pub patch: IdentityProviderConfigPatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdentityProviderConfigPatch {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub authorization_url: Option<String>,
    pub token_url: Option<String>,
    pub userinfo_url: Option<String>,
    pub scopes: Option<String>,
    pub use_pkce: Option<bool>,
}

impl IdentityProviderConfigPatch {
    pub fn touches_config(&self) -> bool {
        self.client_id.is_some()
            || self.client_secret.is_some()
            || self.authorization_url.is_some()
            || self.token_url.is_some()
            || self.userinfo_url.is_some()
            || self.scopes.is_some()
            || self.use_pkce.is_some()
    }

    pub fn apply(&self, stored: &JsonValue) -> Result<Option<JsonValue>, CoreError> {
        if !self.touches_config() {
            return Ok(None);
        }

        let mut merged = match stored {
            JsonValue::Object(map) => map.clone(),
            JsonValue::Null => serde_json::Map::new(),
            _ => {
                return Err(CoreError::InvalidProviderConfiguration(
                    "Provider configuration must be a JSON object".to_string(),
                ));
            }
        };

        set_required(&mut merged, "client_id", self.client_id.as_deref())?;
        set_required(&mut merged, "client_secret", self.client_secret.as_deref())?;
        set_required(
            &mut merged,
            "authorization_url",
            self.authorization_url.as_deref(),
        )?;
        set_required(&mut merged, "token_url", self.token_url.as_deref())?;

        if let Some(scopes) = self.scopes.as_deref() {
            merged.insert("scopes".to_string(), JsonValue::String(scopes.to_string()));
        }

        if let Some(userinfo_url) = self.userinfo_url.as_deref() {
            if userinfo_url.is_empty() {
                merged.remove("userinfo_url");
            } else {
                merged.insert(
                    "userinfo_url".to_string(),
                    JsonValue::String(userinfo_url.to_string()),
                );
            }
        }

        if let Some(use_pkce) = self.use_pkce {
            merged.insert("use_pkce".to_string(), JsonValue::Bool(use_pkce));
        }

        Ok(Some(JsonValue::Object(merged)))
    }
}

fn set_required(
    config: &mut serde_json::Map<String, JsonValue>,
    key: &str,
    value: Option<&str>,
) -> Result<(), CoreError> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.is_empty() {
        return Err(CoreError::InvalidProviderConfiguration(format!(
            "{key} cannot be emptied on an existing provider"
        )));
    }

    config.insert(key.to_string(), JsonValue::String(value.to_string()));

    Ok(())
}

impl UpdateIdentityProviderRequest {
    pub fn resolve_config(&self, stored: &JsonValue) -> Result<Option<JsonValue>, CoreError> {
        match self.patch.apply(self.config.as_ref().unwrap_or(stored))? {
            Some(merged) => Ok(Some(merged)),
            None => Ok(self.config.clone()),
        }
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
            "userinfo_url": "https://idp.example/userinfo",
            "scopes": "openid email profile"
        })
    }

    fn applied(patch: IdentityProviderConfigPatch) -> JsonValue {
        patch
            .apply(&stored_config())
            .expect("applying a patch to a stored object config must succeed")
            .expect("a non-empty patch must write the config column")
    }

    #[test]
    fn a_new_secret_replaces_only_the_secret() {
        let merged = applied(IdentityProviderConfigPatch {
            client_secret: Some("rotated".to_string()),
            ..Default::default()
        });

        let mut expected = stored_config();
        expected["client_secret"] = json!("rotated");
        assert_eq!(merged, expected);
    }

    #[test]
    fn enabling_pkce_keeps_every_other_config_key() {
        let merged = applied(IdentityProviderConfigPatch {
            use_pkce: Some(true),
            ..Default::default()
        });

        let mut expected = stored_config();
        expected["use_pkce"] = json!(true);
        assert_eq!(merged, expected);
    }

    #[test]
    fn disabling_pkce_overwrites_the_stored_flag() {
        let mut stored = stored_config();
        stored["use_pkce"] = json!(true);

        let merged = IdentityProviderConfigPatch {
            use_pkce: Some(false),
            ..Default::default()
        }
        .apply(&stored)
        .expect("applying a patch to a stored object config must succeed")
        .expect("a non-empty patch must write the config column");

        assert_eq!(merged["use_pkce"], json!(false));
        assert_eq!(merged["client_secret"], json!("s3cr3t"));
    }

    #[test]
    fn an_emptied_userinfo_url_drops_the_key() {
        let merged = applied(IdentityProviderConfigPatch {
            userinfo_url: Some(String::new()),
            ..Default::default()
        });

        assert_eq!(merged.get("userinfo_url"), None);
        assert_eq!(merged["token_url"], json!("https://idp.example/token"));
    }

    #[test]
    fn a_new_userinfo_url_is_stored() {
        let merged = applied(IdentityProviderConfigPatch {
            userinfo_url: Some("https://idp.example/me".to_string()),
            ..Default::default()
        });

        assert_eq!(merged["userinfo_url"], json!("https://idp.example/me"));
    }

    #[test]
    fn scopes_may_be_emptied_because_the_create_wizard_allows_it() {
        let merged = applied(IdentityProviderConfigPatch {
            scopes: Some(String::new()),
            ..Default::default()
        });

        assert_eq!(merged["scopes"], json!(""));
    }

    #[test]
    fn emptying_a_required_key_is_refused() {
        let patches = [
            IdentityProviderConfigPatch {
                client_id: Some(String::new()),
                ..Default::default()
            },
            IdentityProviderConfigPatch {
                client_secret: Some(String::new()),
                ..Default::default()
            },
            IdentityProviderConfigPatch {
                authorization_url: Some(String::new()),
                ..Default::default()
            },
            IdentityProviderConfigPatch {
                token_url: Some(String::new()),
                ..Default::default()
            },
        ];

        for patch in patches {
            let error = patch
                .apply(&stored_config())
                .expect_err(&format!("{patch:?} must be refused"));

            assert!(
                matches!(error, CoreError::InvalidProviderConfiguration(_)),
                "{patch:?} produced {error:?}"
            );
        }
    }

    #[test]
    fn an_empty_patch_leaves_the_config_column_alone() {
        assert_eq!(
            IdentityProviderConfigPatch::default()
                .apply(&stored_config())
                .expect("an empty patch cannot fail"),
            None
        );
    }

    #[test]
    fn a_missing_config_becomes_an_object_holding_only_the_patch() {
        let merged = IdentityProviderConfigPatch {
            use_pkce: Some(true),
            ..Default::default()
        }
        .apply(&JsonValue::Null)
        .expect("a null stored config has nothing to lose")
        .expect("a non-empty patch must write the config column");

        assert_eq!(merged, json!({ "use_pkce": true }));
    }

    #[test]
    fn a_non_object_config_is_refused_rather_than_flattened() {
        let error = IdentityProviderConfigPatch {
            use_pkce: Some(true),
            ..Default::default()
        }
        .apply(&json!(["not", "an", "object"]))
        .expect_err("a non-object config must not be silently replaced");

        assert!(matches!(error, CoreError::InvalidProviderConfiguration(_)));
    }

    #[test]
    fn the_merged_config_still_parses_for_the_broker() {
        let merged = applied(IdentityProviderConfigPatch {
            client_secret: Some("rotated".to_string()),
            use_pkce: Some(true),
            ..Default::default()
        });

        let config = OAuthProviderConfig::try_from(merged)
            .expect("the merged config must still parse as an OAuth provider config");

        assert_eq!(config.client_secret, "rotated");
        assert_eq!(config.use_pkce, Some(true));
    }

    #[test]
    fn an_explicit_config_is_the_base_the_patch_is_merged_into() {
        let request = UpdateIdentityProviderRequest {
            config: Some(json!({ "client_id": "replaced" })),
            patch: IdentityProviderConfigPatch {
                use_pkce: Some(true),
                ..Default::default()
            },
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
}
