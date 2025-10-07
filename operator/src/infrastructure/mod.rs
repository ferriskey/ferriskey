use kube::Client;

use crate::{
    domain::{
        common::{Environment, OperatorConfig},
        error::OperatorError,
    },
    infrastructure::cluster::repositories::{
        ClusterRepository, k8s::K8sClusterRepository, mock::MockClusterRepository,
    },
};

pub mod cluster;
pub mod ferriskey_client;

pub async fn build_repos_from_conf(
    config: &OperatorConfig,
) -> Result<ClusterRepository, OperatorError> {
    let client: Option<Client> =
        match config.env {
            Environment::Test => None,
            _ => Some(Client::try_default().await.map_err(|e| {
                OperatorError::InternalServerError {
                    message: e.to_string(),
                }
            })?),
        };

    let cluster_repo = match config.env {
        Environment::Test => ClusterRepository::Mock(MockClusterRepository::new()),
        _ => {
            let client = client.ok_or(OperatorError::InternalServerError {
                message: "Kubernetes client is not initialized".to_string(),
            })?;
            ClusterRepository::K8s(K8sClusterRepository::new(client))
        }
    };

    Ok(cluster_repo)
}
