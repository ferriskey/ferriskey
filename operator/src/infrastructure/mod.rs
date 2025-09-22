use kube::Client;

use crate::{
    domain::error::OperatorError,
    infrastructure::cluster::repositories::{ClusterRepository, k8s::K8sClusterRepository},
};

pub mod cluster;
pub mod ferriskey_client;

pub async fn build_repos_from_conf() -> Result<ClusterRepository, OperatorError> {
    let client = Client::try_default()
        .await
        .map_err(|e| OperatorError::InternalServerError {
            message: e.to_string(),
        })?;

    let cluster_repo = ClusterRepository::K8s(K8sClusterRepository::new(client.clone()));

    Ok(cluster_repo)
}
