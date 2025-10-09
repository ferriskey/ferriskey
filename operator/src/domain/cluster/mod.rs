pub mod entities;
pub mod ports;
pub mod services;

#[cfg(test)]
pub mod test_helpers {
    use crate::domain::{
        cluster::{
            entities::{ApiSpec, ClusterSpec, ClusterStatus, DatabaseConfig, SecretReference},
            ports::MockClusterRepository,
        },
        common::{services::Service, testing::TestServiceBuilder},
        error::OperatorError,
    };

    pub fn create_default_cluster_spec() -> ClusterSpec {
        ClusterSpec {
            name: "test-cluster".to_string(),
            version: "1.0.0".to_string(),
            replicas: 3,
            database: DatabaseConfig {
                secret_ref: SecretReference {
                    name: "postgres-secret".to_string(),
                    namespace: Some("default".to_string()),
                },
                database_name: Some("ferriskey".to_string()),
                ssl_mode: Some("require".to_string()),
            },
            api: ApiSpec {
                webapp_url: "https://app.ferriskey.io".to_string(),
                api_url: "https://api.ferriskey.io".to_string(),
                allowed_origins: vec!["https://app.ferriskey.io".to_string()],
            },
        }
    }

    /// Creates a standard test ClusterStatus
    pub fn create_default_cluster_status() -> ClusterStatus {
        ClusterStatus {
            ready: true,
            message: Some("Cluster applied successfully".to_string()),
            phase: Some("Running".to_string()),
        }
    }

    /// Create a custom ClusterSpec
    pub fn create_cluster_spec_with_name(name: &str) -> ClusterSpec {
        let mut spec = create_default_cluster_spec();
        spec.name = name.to_string();
        spec
    }

    /// Creates a ClusterSpec for testing with specific configurations.
    pub fn create_cluster_spec_with_replicas(replicas: u32) -> ClusterSpec {
        let mut spec = create_default_cluster_spec();
        spec.replicas = replicas;
        spec
    }

    /// Helper for creating a test service with a repository that always succeeds
    pub fn create_service_with_successful_cluster_ops() -> Service<MockClusterRepository> {
        TestServiceBuilder::new()
            .customize_cluster_repository(|mock| {
                mock.expect_apply().returning(move |_, _| {
                    Box::pin(async move { Ok(create_default_cluster_status()) })
                });
                mock.expect_delete()
                    .returning(move |_, _| Box::pin(async move { Ok(()) }));
            })
            .build()
    }

    ///  Helper for creating a test service with a repository that always fails
    pub fn create_service_with_failing_cluster_ops() -> Service<MockClusterRepository> {
        TestServiceBuilder::new()
            .customize_cluster_repository(|mock| {
                mock.expect_apply().returning(|_, _| {
                    Box::pin(async move {
                        Err(OperatorError::ApplyApiError {
                            message: "Simulated failure".to_string(),
                        })
                    })
                });
                mock.expect_delete().returning(|_, _| {
                    Box::pin(async move {
                        Err(OperatorError::DeleteApiError {
                            message: "Simulated failure".to_string(),
                        })
                    })
                });
            })
            .build()
    }

    pub fn create_service_with_custom_behavior<F>(configurator: F) -> Service<MockClusterRepository>
    where
        F: FnOnce(&mut MockClusterRepository),
    {
        TestServiceBuilder::new()
            .customize_cluster_repository(configurator)
            .build()
    }

    pub mod scenarios {

        use mockall::predicate::eq;

        use crate::domain::{
            cluster::{
                entities::ClusterSpec,
                ports::MockClusterRepository,
                test_helpers::{
                    create_default_cluster_status, create_service_with_custom_behavior,
                    create_service_with_failing_cluster_ops,
                    create_service_with_successful_cluster_ops,
                },
            },
            common::services::Service,
            error::OperatorError,
        };

        pub fn always_succeeds() -> Service<MockClusterRepository> {
            create_service_with_successful_cluster_ops()
        }

        /// Scenario: Repository always fails
        pub fn always_fails() -> Service<MockClusterRepository> {
            create_service_with_failing_cluster_ops()
        }

        /// Scenario: Repository fails to apply but succeeds in deleting
        pub fn fails_on_apply_succeeds_on_delete() -> Service<MockClusterRepository> {
            create_service_with_custom_behavior(|mock| {
                mock.expect_apply().returning(|_, _| {
                    Box::pin(async move {
                        Err(OperatorError::ApplyApiError {
                            message: "Apply failed".to_string(),
                        })
                    })
                });
                mock.expect_delete()
                    .returning(|_, _| Box::pin(async move { Ok(()) }));
            })
        }

        /// Scenario: Repository succeeds in applying but fails in deleting
        pub fn succeeds_on_apply_fails_on_delete() -> Service<MockClusterRepository> {
            create_service_with_custom_behavior(|mock| {
                mock.expect_apply()
                    .returning(|_, _| Box::pin(async move { Ok(create_default_cluster_status()) }));
                mock.expect_delete().returning(|_, _| {
                    Box::pin(async move {
                        Err(OperatorError::DeleteApiError {
                            message: "Delete failed".to_string(),
                        })
                    })
                });
            })
        }

        /// Scenario: Repository has specific expectations
        pub fn with_specific_expectations(
            spec: ClusterSpec,
            namespace: &str,
        ) -> Service<MockClusterRepository> {
            let ns = namespace.to_string();
            create_service_with_custom_behavior(move |mock| {
                mock.expect_apply()
                    .with(eq(spec.clone()), eq(ns.clone()))
                    .times(1)
                    .returning(|_, _| Box::pin(async move { Ok(create_default_cluster_status()) }));
                mock.expect_delete()
                    .with(eq(spec), eq(ns))
                    .times(1)
                    .returning(|_, _| Box::pin(async move { Ok(()) }));
            })
        }
    }
}
