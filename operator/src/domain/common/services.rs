use crate::domain::cluster::ports::ClusterRepository;

#[derive(Clone)]
pub struct Service<C>
where
    C: ClusterRepository,
{
    pub(crate) cluster_repository: C,
}

impl<C> Service<C>
where
    C: ClusterRepository,
{
    pub fn new(cluster_repository: C) -> Self {
        Service { cluster_repository }
    }
}
