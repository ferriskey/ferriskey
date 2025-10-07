use crate::{
    domain::{
        common::{Environment, OperatorConfig},
        error::OperatorError,
    },
    infrastructure::{build_repos_from_conf, cluster::repositories::ClusterRepository},
};

pub mod cluster;

#[derive(Clone)]
pub struct OperatorService {
    cluster_repository: ClusterRepository,
}

impl OperatorService {
    pub async fn new(config: &OperatorConfig) -> Result<Self, OperatorError> {
        let cluster_repository = build_repos_from_conf(config).await?;
        Ok(Self { cluster_repository })
    }

    pub async fn mock() -> Result<Self, OperatorError> {
        let cluster_repository = build_repos_from_conf(&OperatorConfig {
            env: Environment::Test,
        })
        .await?;

        Ok(Self { cluster_repository })
    }

    pub async fn run() -> Result<(), OperatorError> {
        // Placeholder for future implementation
        //
        Ok(())
    }
}
