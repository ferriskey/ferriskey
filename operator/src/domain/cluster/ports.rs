use crate::domain::{
    cluster::entities::{ClusterSpec, ClusterStatus},
    error::OperatorError,
};

#[cfg_attr(test, mockall::automock)]
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

#[cfg_attr(test, mockall::automock)]
pub trait ClusterRepository: Send + Sync {
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
