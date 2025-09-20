use crate::{
    domain::error::OperatorError,
    infrastructure::{build_repos_from_conf, cluster::repositories::ClusterRepository},
};

pub mod cluster;

#[derive(Clone)]
pub struct OperatorService {
    cluster_repository: ClusterRepository,
}

impl OperatorService {
    pub async fn new() -> Result<Self, OperatorError> {
        let cluster_repository = build_repos_from_conf().await?;
        Ok(Self { cluster_repository })
    }

    pub async fn run() -> Result<(), OperatorError> {
        // Placeholder for future implementation
        //
        Ok(())
    }
}
