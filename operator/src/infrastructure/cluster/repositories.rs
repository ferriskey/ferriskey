use crate::{
    domain::{
        cluster::{ClusterPort, ClusterSpec, ClusterStatus},
        error::OperatorError,
    },
    infrastructure::cluster::repositories::k8s::K8sClusterRepository,
};

pub mod k8s;

#[derive(Clone)]
pub enum ClusterRepository {
    K8s(K8sClusterRepository),
}

impl ClusterPort for ClusterRepository {
    async fn apply(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<ClusterStatus, OperatorError> {
        match self {
            ClusterRepository::K8s(a) => a.apply(spec, namespace).await,
        }
    }

    async fn delete(&self, spec: &ClusterSpec, namespace: &str) -> Result<(), OperatorError> {
        match self {
            ClusterRepository::K8s(a) => a.delete(spec, namespace).await,
        }
    }
}
