use crate::{
    domain::{
        cluster::{ClusterPort, ClusterSpec, ClusterStatus},
        error::OperatorError,
    },
    infrastructure::cluster::repositories::{
        k8s::K8sClusterRepository,
        mock::{MockClusterConfig, MockClusterRepository},
    },
};

pub mod k8s;
pub mod mock;

#[derive(Clone)]
pub enum ClusterRepository {
    K8s(K8sClusterRepository),
    Mock(MockClusterRepository),
}

impl ClusterRepository {
    pub fn k8s(client: kube::Client) -> Self {
        Self::K8s(K8sClusterRepository::new(client))
    }

    /// Create a new mock repository for testing
    pub fn mock() -> Self {
        Self::Mock(MockClusterRepository::new())
    }

    /// Create a mock repository with custom configuration
    pub fn mock_with_config(config: MockClusterConfig) -> Self {
        Self::Mock(MockClusterRepository::with_config(config))
    }

    /// Create a failing mock repository for testing error scenarios
    pub fn mock_failing() -> Self {
        Self::Mock(MockClusterRepository::failing())
    }
}

impl ClusterPort for ClusterRepository {
    async fn apply(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<ClusterStatus, OperatorError> {
        match self {
            ClusterRepository::K8s(a) => a.apply(spec, namespace).await,
            ClusterRepository::Mock(a) => a.apply(spec, namespace).await,
        }
    }

    async fn delete(&self, spec: &ClusterSpec, namespace: &str) -> Result<(), OperatorError> {
        match self {
            ClusterRepository::K8s(a) => a.delete(spec, namespace).await,
            ClusterRepository::Mock(a) => a.delete(spec, namespace).await,
        }
    }
}
