use kube_derive::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[kube(
    group = "ferriskey.io",
    version = "v1",
    kind = "FerriskeyCluster",
    plural = "ferriskeyclusters",
    namespaced
)]
#[kube(shortname = "fkc", status = "FerrisKeyClusterStatus")]
pub struct FerriskeyClusterSpec {
    pub version: Option<String>,
    pub frontend: FrontendSpec,
    pub backend: BackendSpec,
    pub postgres: PostgresSpec,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct FrontendSpec {
    pub enabled: bool,
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct BackendSpec {
    pub replicas: u8,
    pub image: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub allowed_origins: Option<Vec<String>>,
    pub portal_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct PostgresSpec {
    pub image: Option<String>,
    pub storage: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
pub struct FerrisKeyClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
}
