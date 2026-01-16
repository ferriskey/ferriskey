use chrono::{DateTime, Utc};
use ferriskey_core::domain::abyss::federation::{
    entities::FederationProvider,
    value_objects::{SyncResult, TestConnectionResult},
};
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

#[derive(Debug, Serialize, ToSchema)]
pub struct TestConnectionResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl From<TestConnectionResult> for TestConnectionResponse {
    fn from(result: TestConnectionResult) -> Self {
        Self {
            success: result.success,
            message: result.message,
            details: result.details,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SyncUsersResponse {
    pub total_processed: u32,
    pub created: u32,
    pub updated: u32,
    pub disabled: u32,
    pub failed: u32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<SyncResult> for SyncUsersResponse {
    fn from(result: SyncResult) -> Self {
        let started_at: Option<DateTime<Utc>> = result
            .started_at
            .and_then(|dt| DateTime::parse_from_rfc3339(&dt).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let completed_at: Option<DateTime<Utc>> = result
            .completed_at
            .and_then(|dt| DateTime::parse_from_rfc3339(&dt).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Self {
            total_processed: result.total_processed,
            created: result.created,
            updated: result.updated,
            disabled: result.disabled,
            failed: result.failed,
            started_at,
            completed_at,
        }
    }
}
