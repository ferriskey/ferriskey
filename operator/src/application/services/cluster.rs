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
    use crate::domain::cluster::{ApiSpec, ClusterSpec, DatabaseConfig, SecretReference};

    fn create_test_spec() -> ClusterSpec {
        ClusterSpec {
            name: "test-cluster".to_string(),
            version: "1.0.0".to_string(),
            replicas: 2,
            database: DatabaseConfig {
                secret_ref: SecretReference {
                    name: "db-secret".to_string(),
                    namespace: Some("default".to_string()),
                },
                database_name: Some("ferriskey".to_string()),
                ssl_mode: Some("require".to_string()),
            },
            api: ApiSpec {
                webapp_url: "https://webapp.example.com".to_string(),
                api_url: "https://api.example.com".to_string(),
                allowed_origins: vec!["https://frontend.example.com".to_string()],
            },
        }
    }

    fn create_custom_spec(name: &str, replicas: u32) -> ClusterSpec {
        ClusterSpec {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            replicas,
            database: DatabaseConfig {
                secret_ref: SecretReference {
                    name: format!("{}-db-secret", name),
                    namespace: Some("default".to_string()),
                },
                database_name: Some(format!("{}_db", name)),
                ssl_mode: Some("require".to_string()),
            },
            api: ApiSpec {
                webapp_url: format!("https://{}.example.com", name),
                api_url: format!("https://api-{}.example.com", name),
                allowed_origins: vec![format!("https://{}.example.com", name)],
            },
        }
    }

    mod reconcile_cluster_tests {
        use crate::{
            application::services::{
                OperatorService,
                cluster::tests::{create_custom_spec, create_test_spec},
            },
            domain::{cluster::ClusterService, error::OperatorError},
        };

        #[tokio::test]
        async fn test_successful_reconcile() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            let result = service.reconcile_cluster(&spec, "production").await;

            assert!(result.is_ok());
            let status = result.unwrap();
            assert!(!status.ready);
            assert_eq!(status.phase, Some("Progressing".to_string()));
            assert!(status.message.is_some());
        }

        #[tokio::test]
        async fn test_reconcile_empty_cluster_name() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let mut spec = create_test_spec();
            spec.name = "".to_string();

            let result = service.reconcile_cluster(&spec, "production").await;

            assert!(result.is_err());
            match result {
                Err(OperatorError::InvalidSpec { message }) => {
                    assert_eq!(message, "Cluster name cannot be empty");
                }
                _ => panic!("Expected InvalidSpec error"),
            }
        }

        #[tokio::test]
        async fn test_reconcile_multiple_clusters_same_namespace() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec1 = create_custom_spec("cluster-1", 1);
            let spec2 = create_custom_spec("cluster-2", 2);

            let result1 = service.reconcile_cluster(&spec1, "production").await;
            let result2 = service.reconcile_cluster(&spec2, "production").await;

            assert!(result1.is_ok());
            assert!(result2.is_ok());
        }

        #[tokio::test]
        async fn test_reconcile_same_cluster_different_namespaces() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            let result1 = service.reconcile_cluster(&spec, "production").await;
            let result2 = service.reconcile_cluster(&spec, "staging").await;

            assert!(result1.is_ok());
            assert!(result2.is_ok());
        }

        #[tokio::test]
        async fn test_reconcile_idempotency() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            let result1 = service.reconcile_cluster(&spec, "production").await;
            assert!(result1.is_ok());

            let result2 = service.reconcile_cluster(&spec, "production").await;
            assert!(result2.is_ok());
        }

        #[tokio::test]
        async fn test_reconcile_with_various_replica_counts() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");

            for replicas in [1, 3, 5, 10] {
                let spec = create_custom_spec(&format!("cluster-{}", replicas), replicas);
                let result = service.reconcile_cluster(&spec, "test").await;
                assert!(result.is_ok(), "Failed for {} replicas", replicas);
            }
        }
    }

    mod cleanup_cluster_tests {
        use crate::{application::services::OperatorService, domain::cluster::ClusterService};

        use super::*;

        #[tokio::test]
        async fn test_successful_cleanup() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            service
                .reconcile_cluster(&spec, "production")
                .await
                .unwrap();
            let result = service.cleanup_cluster(&spec, "production").await;

            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_cleanup_nonexistent_cluster() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            let result = service.cleanup_cluster(&spec, "production").await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_cleanup_multiple_clusters() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec1 = create_custom_spec("cluster-1", 1);
            let spec2 = create_custom_spec("cluster-2", 2);

            service
                .reconcile_cluster(&spec1, "production")
                .await
                .unwrap();
            service
                .reconcile_cluster(&spec2, "production")
                .await
                .unwrap();

            let result1 = service.cleanup_cluster(&spec1, "production").await;
            let result2 = service.cleanup_cluster(&spec2, "production").await;

            assert!(result1.is_ok());
            assert!(result2.is_ok());
        }
    }

    mod integration_tests {
        use crate::{application::services::OperatorService, domain::cluster::ClusterService};

        use super::*;

        #[tokio::test]
        async fn test_full_cluster_lifecycle() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();

            // 1. Reconcile cluster
            let reconcile_result = service.reconcile_cluster(&spec, "production").await;
            assert!(reconcile_result.is_ok());

            // 2. Reconcile again (idempotent)
            let reconcile_result2 = service.reconcile_cluster(&spec, "production").await;
            assert!(reconcile_result2.is_ok());

            // 3. Cleanup cluster
            let cleanup_result = service.cleanup_cluster(&spec, "production").await;
            assert!(cleanup_result.is_ok());

            // 4. Cleanup again (idempotent)
            let cleanup_result2 = service.cleanup_cluster(&spec, "production").await;
            assert!(cleanup_result2.is_ok());
        }

        #[tokio::test]
        async fn test_multiple_namespaces_lifecycle() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");
            let spec = create_test_spec();
            let namespaces = ["production", "staging", "development"];

            for namespace in &namespaces {
                let result = service.reconcile_cluster(&spec, namespace).await;
                assert!(result.is_ok(), "Failed to deploy to {}", namespace);
            }

            for namespace in &namespaces {
                let result = service.cleanup_cluster(&spec, namespace).await;
                assert!(result.is_ok(), "Failed to cleanup from {}", namespace);
            }
        }

        #[tokio::test]
        async fn test_complex_deployment_scenario() {
            let service = OperatorService::mock()
                .await
                .expect("operator service error");

            let clusters = vec![
                ("web-frontend", 3),
                ("api-backend", 2),
                ("database-cluster", 1),
                ("cache-cluster", 2),
            ];

            // Deploy all clusters
            for (name, replicas) in &clusters {
                let spec = create_custom_spec(name, *replicas);
                let result = service.reconcile_cluster(&spec, "production").await;
                assert!(result.is_ok(), "Failed to deploy {}", name);
            }

            // Cleanup all clusters
            for (name, replicas) in &clusters {
                let spec = create_custom_spec(name, *replicas);
                let result = service.cleanup_cluster(&spec, "production").await;
                assert!(result.is_ok(), "Failed to cleanup {}", name);
            }
        }
    }
}
