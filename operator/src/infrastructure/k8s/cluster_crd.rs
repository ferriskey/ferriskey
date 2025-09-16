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
    pub database_url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default, JsonSchema)]
pub struct FerrisKeyClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
}
