use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "ferriskey.rs",
    version = "v1alpha1",
    kind = "FerrisKeyCluster",
    plural = "ferriskeyclusters",
    namespaced
)]
#[kube(status = "FerrisKeyClusterStatus")]
pub struct FerrisKeyClusterSpec {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub database: DatabaseSpec,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DatabaseSpec {
    /// Reference to a secret containing database credentials
    pub secret_ref: SecretReference,
    /// Optional: Database name override (if not specified in secret)
    pub database_name: Option<String>,
    /// Optional: SSL mode for database connection
    pub ssl_mode: Option<String>, // e.g., "require", "disable", "prefer"
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct SecretReference {
    /// Name of the secret containing database credentials
    pub name: String,
    /// Optional: Namespace of the secret (defaults to same namespace as cluster)
    pub namespace: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct FerrisKeyClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
    pub phase: Option<String>, // e.g., "Pending", "Running", "Failed", "Terminating"
    pub conditions: Option<Vec<ClusterCondition>>,
    pub database_status: Option<DatabaseStatus>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct ClusterCondition {
    pub condition_type: String, // e.g., "Ready", "Progressing", "Degraded",
    pub status: String,         // "True", "False", "Unknown"
    pub last_transition_time: String, // ISO 8601 timestamp
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema)]
pub struct DatabaseStatus {
    pub connected: bool,
    pub host: Option<String>,
    pub database: Option<String>,
    pub last_check: Option<String>,
}
