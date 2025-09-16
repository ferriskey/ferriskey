use crate::domain::error::OperatorError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FerrisKeyCluster {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub database_url: String,
}

#[derive(Debug, Clone)]
pub struct ClusterSpec {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub database_url: String,
}

#[derive(Debug, Clone)]
pub struct ClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
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
