use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum FederationType {
    Ldap,
    Kerberos,
    ActiveDirectory,
    Custom(String),
}

impl fmt::Display for FederationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FederationType::Ldap => write!(f, "Ldap"),
            FederationType::Kerberos => write!(f, "Kerberos"),
            FederationType::ActiveDirectory => write!(f, "ActiveDirectory"),
            FederationType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    Import,
    Force,
    LinkOnly,
}

impl fmt::Display for SyncMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncMode::Import => write!(f, "Import"),
            SyncMode::Force => write!(f, "Force"),
            SyncMode::LinkOnly => write!(f, "LinkOnly"),
        }
    }
}

impl FromStr for SyncMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Import" => Ok(SyncMode::Import),
            "Force" => Ok(SyncMode::Force),
            "LinkOnly" => Ok(SyncMode::LinkOnly),
            _ => Err(format!("Invalid SyncMode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Success,
    Failure,
    PartialSuccess,
    InProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FederationProvider {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub provider_type: FederationType,
    pub enabled: bool,
    pub priority: i32,
    pub config: serde_json::Value,
    pub sync_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMapping {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub user_id: Uuid,
    pub external_id: String,
    pub external_username: String,
    pub mapping_metadata: serde_json::Value,
    pub last_synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederatedUser {
    pub external_id: String,
    pub username: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<HashMap<String, Vec<String>>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_mode_display_and_from_str() {
        let mode = SyncMode::Import;
        assert_eq!(mode.to_string(), "Import");
        assert_eq!(SyncMode::from_str("Import").unwrap(), SyncMode::Import);

        let mode = SyncMode::Force;
        assert_eq!(mode.to_string(), "Force");
        assert_eq!(SyncMode::from_str("Force").unwrap(), SyncMode::Force);

        let mode = SyncMode::LinkOnly;
        assert_eq!(mode.to_string(), "LinkOnly");
        assert_eq!(SyncMode::from_str("LinkOnly").unwrap(), SyncMode::LinkOnly);

        assert!(SyncMode::from_str("Invalid").is_err());
    }
}
