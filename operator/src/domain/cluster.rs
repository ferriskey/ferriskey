use crate::domain::error::OperatorError;

#[derive(Debug, Clone)]
pub struct ClusterSpec {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub database: DatabaseConfig,

    pub api: ApiSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSpec {
    pub webapp_url: String,
    pub api_url: String,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub secret_ref: SecretReference,
    /// Optional: Database name override (if not specified in secret)
    pub database_name: Option<String>,
    /// Optional: SSL mode for database connection
    pub ssl_mode: Option<String>, // e.g., "require", "disable", "prefer"}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretReference {
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
    pub phase: Option<String>,
}

#[derive(Debug)]
pub enum ClusterAction {
    Create,
    Update,
    NoOp,
}

pub trait ClusterService: Send + Sync {
    fn reconcile_cluster(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> impl Future<Output = Result<ClusterStatus, OperatorError>> + Send;
    fn cleanup_cluster(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> impl Future<Output = Result<(), OperatorError>> + Send;
}

pub trait ClusterPort: Send + Sync {
    fn apply(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> impl Future<Output = Result<ClusterStatus, OperatorError>> + Send;
    fn delete(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> impl Future<Output = Result<(), OperatorError>> + Send;
}
