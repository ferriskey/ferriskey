use tracing::info;

use crate::{
    application::services::OperatorService,
    domain::{
        cluster::{ClusterPort, ClusterService, ClusterSpec, ClusterStatus},
        error::OperatorError,
    },
};

impl ClusterService for OperatorService {
    async fn reconcile_cluster(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<ClusterStatus, OperatorError> {
        if spec.name.is_empty() {
            return Err(OperatorError::InvalidSpec {
                message: "Cluster name cannot be empty".into(),
            });
        }

        info!("Je suis dans le cluster service reconcile cluster");

        self.cluster_repository.apply(spec, namespace).await
    }

    async fn cleanup_cluster(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<(), OperatorError> {
        self.cluster_repository.delete(spec, namespace).await
    }
}
