pub mod services;

pub struct OperatorConfig {
    pub env: Environment,
}

pub enum Environment {
    Test,
    Development,
    Production,
}

#[cfg(test)]
pub mod testing {
    use crate::domain::{
        cluster::ports::{ClusterRepository, MockClusterRepository},
        common::services::Service,
    };

    pub trait TestableService<C>
    where
        C: ClusterRepository,
    {
        fn with_mock_cluster_repository(cluster_repo: C) -> Service<C>;
    }

    impl<C> TestableService<C> for Service<C>
    where
        C: ClusterRepository,
    {
        fn with_mock_cluster_repository(cluster_repo: C) -> Service<C> {
            Service::new(cluster_repo)
        }
    }

    /// Builder to easily create test services
    pub struct TestServiceBuilder {
        cluster_repository: Option<MockClusterRepository>,
    }

    impl TestServiceBuilder {
        pub fn new() -> Self {
            Self {
                cluster_repository: None,
            }
        }

        /// Configure a custom mock cluster repository
        pub fn with_cluster_repository(mut self, repo: MockClusterRepository) -> Self {
            self.cluster_repository = Some(repo);
            self
        }

        /// Configures a mock cluster repository with a closure
        pub fn customize_cluster_repository<F>(mut self, configurator: F) -> Self
        where
            F: FnOnce(&mut MockClusterRepository),
        {
            let mut mock = MockClusterRepository::new();
            configurator(&mut mock);
            self.cluster_repository = Some(mock);
            self
        }

        /// Construit le service avec des mocks par défaut
        pub fn build(self) -> Service<MockClusterRepository> {
            let cluster_repo = self.cluster_repository.unwrap_or_default();

            Service::new(cluster_repo)
        }
    }

    impl Default for TestServiceBuilder {
        fn default() -> Self {
            Self::new()
        }
    }
}
