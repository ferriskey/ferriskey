use ferriskey_core::domain::abyss::federation::entities::FederationProvider;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProviderRequest {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub priority: i32,
    pub config: serde_json::Value,
    pub sync_enabled: bool,
    pub sync_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_interval_minutes: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProviderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_interval_minutes: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProviderResponse {
    pub id: String,
    pub realm_id: String,
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub priority: i32,
    pub config: serde_json::Value, // SANITIZED - no passwords
    pub sync_enabled: bool,
    pub sync_mode: String,
    pub sync_interval_minutes: Option<i32>,
    pub last_sync_at: Option<String>,
    pub last_sync_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<FederationProvider> for ProviderResponse {
    fn from(provider: FederationProvider) -> Self {
        // Sanitize config
        let mut config = provider.config.clone();
        if let Some(obj) = config.as_object_mut() {
            let sensitive_keys = [
                "password",
                "secret",
                "client_secret",
                "bind_password",
                "bind_password_encrypted",
            ];
            for key in sensitive_keys {
                if obj.contains_key(key) {
                    obj.insert(key.to_string(), serde_json::json!("********"));
                }
            }
            // Also check nested bind object if it exists (common in LDAP)
            if let Some(bind) = obj.get_mut("bind").and_then(|b| b.as_object_mut()) {
                for key in sensitive_keys {
                    if bind.contains_key(key) {
                        bind.insert(key.to_string(), serde_json::json!("********"));
                    }
                }
            }
        }

        // Extract sync settings
        let sync = &provider.sync_settings;
        let sync_enabled = sync
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let sync_mode = sync
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("LinkOnly")
            .to_string();
        let sync_interval_minutes = sync
            .get("interval_minutes")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32);
        let last_sync_at = sync
            .get("last_sync_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let last_sync_status = sync
            .get("last_sync_status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Self {
            id: provider.id.to_string(),
            realm_id: provider.realm_id.to_string(),
            name: provider.name,
            provider_type: provider.provider_type.to_string(),
            enabled: provider.enabled,
            priority: provider.priority,
            config,
            sync_enabled,
            sync_mode,
            sync_interval_minutes,
            last_sync_at,
            last_sync_status,
            created_at: provider.created_at.to_rfc3339(),
            updated_at: provider.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListProvidersResponse {
    pub data: Vec<ProviderResponse>,
}
