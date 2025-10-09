use crate::domain::{
    cluster::{
        entities::{ClusterSpec, ClusterStatus},
        ports::{ClusterRepository, ClusterService},
    },
    common::services::Service,
    error::OperatorError,
};

impl<C> ClusterService for Service<C>
where
    C: ClusterRepository,
{
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

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::domain::{
        cluster::{
            ports::ClusterService,
            test_helpers::{
                create_cluster_spec_with_name, create_cluster_spec_with_replicas,
                create_default_cluster_spec, create_default_cluster_status,
                create_service_with_custom_behavior, scenarios,
            },
        },
        common::testing::TestServiceBuilder,
        error::OperatorError,
    };

    #[tokio::test]
    async fn test_reconcile_cluster_success() {
        let service = scenarios::always_succeeds();
        let spec = create_default_cluster_spec();

        let result = service.reconcile_cluster(&spec, "default").await;

        assert!(result.is_ok());
        let status = result.unwrap();
        assert!(status.ready);
        assert!(status.message.is_some());
    }

    #[tokio::test]
    async fn test_reconcile_cluster_empty_name_fails() {
        // Arrange
        let service = scenarios::always_succeeds();
        let mut spec = create_default_cluster_spec();
        spec.name = "".to_string();

        // Act
        let result = service.reconcile_cluster(&spec, "default").await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            OperatorError::InvalidSpec { message } => {
                assert_eq!(message, "Cluster name cannot be empty");
            }
            _ => panic!("Expected InvalidSpec error"),
        }
    }

    #[tokio::test]
    async fn test_reconcile_cluster_repository_error() {
        // Arrange
        let service = scenarios::always_fails();
        let spec = create_default_cluster_spec();

        // Act
        let result = service.reconcile_cluster(&spec, "default").await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            OperatorError::ApplyApiError { message } => {
                assert_eq!(message, "Simulated failure");
            }
            _ => panic!("Expected ApplyApiError"),
        }
    }

    #[tokio::test]
    async fn test_cleanup_cluster_success() {
        // Arrange
        let service = scenarios::always_succeeds();
        let spec = create_default_cluster_spec();

        // Act
        let result = service.cleanup_cluster(&spec, "default").await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_cluster_repository_error() {
        // Arrange
        let service = scenarios::always_fails();
        let spec = create_default_cluster_spec();

        // Act
        let result = service.cleanup_cluster(&spec, "default").await;

        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            OperatorError::DeleteApiError { message } => {
                assert_eq!(message, "Simulated failure");
            }
            _ => panic!("Expected DeleteApiError"),
        }
    }

    #[tokio::test]
    async fn test_multiple_operations_with_different_namespaces() {
        // Arrange
        let spec = create_default_cluster_spec();
        let service = create_service_with_custom_behavior(|mock| {
            mock.expect_apply()
                .with(eq(spec.clone()), eq("production"))
                .times(1)
                .returning(|_, _| Box::pin(async move { Ok(create_default_cluster_status()) }));
            mock.expect_apply()
                .with(eq(spec.clone()), eq("staging"))
                .times(1)
                .returning(|_, _| Box::pin(async move { Ok(create_default_cluster_status()) }));
        });

        // Act
        let result1 = service.reconcile_cluster(&spec, "production").await;
        let result2 = service.reconcile_cluster(&spec, "staging").await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_different_cluster_configurations() {
        // Arrange
        let service = scenarios::always_succeeds();

        let test_cases = vec![
            create_cluster_spec_with_name("small-cluster"),
            create_cluster_spec_with_replicas(1),
            create_cluster_spec_with_replicas(10),
        ];

        // Act & Assert
        for spec in test_cases {
            let result = service.reconcile_cluster(&spec, "default").await;
            assert!(result.is_ok(), "Failed for cluster: {}", spec.name);
        }
    }

    #[tokio::test]
    async fn test_mixed_success_failure_scenario() {
        // Arrange
        let service = scenarios::fails_on_apply_succeeds_on_delete();
        let spec = create_default_cluster_spec();

        // Act
        let reconcile_result = service.reconcile_cluster(&spec, "default").await;
        let cleanup_result = service.cleanup_cluster(&spec, "default").await;

        // Assert
        assert!(reconcile_result.is_err());
        assert!(cleanup_result.is_ok());
    }

    #[tokio::test]
    async fn test_specific_expectations() {
        // Arrange
        let spec = create_cluster_spec_with_name("specific-cluster");
        let service = scenarios::with_specific_expectations(spec.clone(), "production");

        // Act
        let reconcile_result = service.reconcile_cluster(&spec, "production").await;
        let cleanup_result = service.cleanup_cluster(&spec, "production").await;

        // Assert
        assert!(reconcile_result.is_ok());
        assert!(cleanup_result.is_ok());
    }

    #[tokio::test]
    async fn test_custom_behavior_with_builder() {
        // Arrange
        let spec = create_default_cluster_spec();
        let service = TestServiceBuilder::new()
            .customize_cluster_repository(|mock| {
                mock.expect_apply()
                    .with(eq(spec.clone()), eq("default"))
                    .times(1)
                    .returning(|_, _| Box::pin(async move { Ok(create_default_cluster_status()) }));

                mock.expect_apply()
                    .with(eq(spec.clone()), eq("default"))
                    .times(1)
                    .returning(|_, _| {
                        Box::pin(async move {
                            Err(OperatorError::ApplyApiError {
                                message: "Resource already exists".to_string(),
                            })
                        })
                    });
            })
            .build();

        // Act
        let result1 = service.reconcile_cluster(&spec, "default").await;
        let result2 = service.reconcile_cluster(&spec, "default").await;

        // Assert
        assert!(result1.is_ok());
        assert!(result2.is_err());
    }
}
