use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::sync::Mutex;

use crate::domain::{
    cluster::{ClusterPort, ClusterSpec, ClusterStatus},
    error::OperatorError,
};

#[derive(Debug, Clone)]
pub struct MockClusterConfig {
    pub apply_success: bool,
    pub delete_success: bool,
    pub operation_delay: Duration,
    pub error_message: String,
    pub eventual_ready: bool,
    pub ready_after: Duration,
}

impl Default for MockClusterConfig {
    fn default() -> Self {
        Self {
            apply_success: true,
            delete_success: true,
            operation_delay: Duration::from_millis(1),
            error_message: "Mock operation failed".to_string(),
            eventual_ready: true,
            ready_after: Duration::from_millis(10),
        }
    }
}

/// Represents the state of a mock cluster
#[derive(Debug, Clone)]
pub struct MockClusterState {
    pub spec: ClusterSpec,
    pub namespace: String,
    pub created_at: Instant,
    pub status: ClusterStatus,
}

impl MockClusterState {
    /// Get the age of this cluster state
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Check if this cluster is in the specified namespace
    pub fn is_in_namespace(&self, namespace: &str) -> bool {
        self.namespace == namespace
    }

    /// Get the cluster name
    pub fn cluster_name(&self) -> &str {
        &self.spec.name
    }

    /// Get a mutable reference to the status (for testing state changes)
    pub fn status_mut(&mut self) -> &mut ClusterStatus {
        &mut self.status
    }
}

#[derive(Clone)]
pub struct MockClusterRepository {
    config: MockClusterConfig,
    clusters: Arc<Mutex<HashMap<(String, String), MockClusterState>>>,
    operation_log: Arc<Mutex<Vec<MockOperation>>>,
}

#[derive(Debug, Clone)]
pub struct MockOperation {
    pub operation_type: String,
    pub cluster_name: String,
    pub namespace: String,
    pub timestamp: Instant,
    pub success: bool,
}

impl MockClusterRepository {
    /// Create a new mock repository with default configuration
    pub fn new() -> Self {
        Self::with_config(MockClusterConfig::default())
    }

    /// Create a new mock repository with custom configuration
    pub fn with_config(config: MockClusterConfig) -> Self {
        Self {
            config,
            clusters: Arc::new(Mutex::new(HashMap::new())),
            operation_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a mock repository that always fails
    pub fn failing() -> Self {
        Self::with_config(MockClusterConfig {
            apply_success: false,
            delete_success: false,
            error_message: "Mock repository configured to fail".to_string(),
            ..MockClusterConfig::default()
        })
    }

    /// Get the current cluster state (for testing)
    pub async fn get_cluster_state(
        &self,
        namespace: &str,
        cluster_name: &str,
    ) -> Option<MockClusterState> {
        let clusters = self.clusters.lock().await;
        clusters
            .get(&(namespace.to_string(), cluster_name.to_string()))
            .cloned()
    }

    /// Check if a cluster exists
    pub async fn cluster_exists(&self, namespace: &str, cluster_name: &str) -> bool {
        let clusters = self.clusters.lock().await;
        clusters.contains_key(&(namespace.to_string(), cluster_name.to_string()))
    }

    /// Get the number of managed clusters
    pub async fn cluster_count(&self) -> usize {
        let clusters = self.clusters.lock().await;
        clusters.len()
    }

    /// Get operation log (for testing)
    pub async fn get_operation_log(&self) -> Vec<MockOperation> {
        let log = self.operation_log.lock().await;
        log.clone()
    }

    /// Clear operation log (for testing)
    pub async fn clear_operation_log(&self) {
        let mut log = self.operation_log.lock().await;
        log.clear();
    }

    /// Get all clusters in a specific namespace
    pub async fn get_clusters_in_namespace(&self, namespace: &str) -> Vec<MockClusterState> {
        let clusters = self.clusters.lock().await;
        clusters
            .values()
            .filter(|state| state.is_in_namespace(namespace))
            .cloned()
            .collect()
    }

    /// Update cluster status (for testing state transitions)
    pub async fn update_cluster_status(
        &self,
        namespace: &str,
        cluster_name: &str,
        status: ClusterStatus,
    ) -> bool {
        let mut clusters = self.clusters.lock().await;
        if let Some(state) = clusters.get_mut(&(namespace.to_string(), cluster_name.to_string())) {
            state.status = status;
            true
        } else {
            false
        }
    }

    /// Get clusters older than the specified duration
    pub async fn get_clusters_older_than(&self, age: Duration) -> Vec<MockClusterState> {
        let clusters = self.clusters.lock().await;
        clusters
            .values()
            .filter(|state| state.age() > age)
            .cloned()
            .collect()
    }

    /// Log an operation for testing purposes
    async fn log_operation(
        &self,
        operation_type: &str,
        cluster_name: &str,
        namespace: &str,
        success: bool,
    ) {
        let mut log = self.operation_log.lock().await;
        log.push(MockOperation {
            operation_type: operation_type.to_string(),
            cluster_name: cluster_name.to_string(),
            namespace: namespace.to_string(),
            timestamp: Instant::now(),
            success,
        });
    }
}

impl Default for MockClusterRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterPort for MockClusterRepository {
    async fn apply(
        &self,
        spec: &ClusterSpec,
        namespace: &str,
    ) -> Result<ClusterStatus, OperatorError> {
        // Simulate operation delay
        if !self.config.operation_delay.is_zero() {
            tokio::time::sleep(self.config.operation_delay).await;
        }

        let success = self.config.apply_success;
        self.log_operation("apply", &spec.name, namespace, success)
            .await;

        if !success {
            return Err(OperatorError::ApplyApiError {
                message: self.config.error_message.clone(),
            });
        }

        // Validate spec (basic validation for testing)
        if spec.name.is_empty() {
            return Err(OperatorError::InvalidSpec {
                message: "Cluster name cannot be empty".to_string(),
            });
        }

        if spec.replicas == 0 {
            return Err(OperatorError::InvalidSpec {
                message: "Replica count must be greater than 0".to_string(),
            });
        }

        // Create or update cluster state
        let cluster_state = MockClusterState {
            spec: spec.clone(),
            namespace: namespace.to_string(),
            created_at: Instant::now(),
            status: ClusterStatus {
                ready: false,
                message: Some("Cluster creation initiated".to_string()),
                phase: Some("Progressing".to_string()),
            },
        };

        let key = (namespace.to_string(), spec.name.clone());
        {
            let mut clusters = self.clusters.lock().await;
            clusters.insert(key, cluster_state.clone());
        }

        // Return success status
        let status = ClusterStatus {
            ready: false,
            message: Some("Cluster applied successfully".to_string()),
            phase: Some("Progressing".to_string()),
        };

        tracing::info!(
            "Mock cluster '{}' applied in namespace '{}' - Ready: {}",
            spec.name,
            namespace,
            status.ready
        );

        Ok(status)
    }

    async fn delete(&self, spec: &ClusterSpec, namespace: &str) -> Result<(), OperatorError> {
        // Simulate operation delay
        if !self.config.operation_delay.is_zero() {
            tokio::time::sleep(self.config.operation_delay).await;
        }

        let success = self.config.delete_success;
        self.log_operation("delete", &spec.name, namespace, success)
            .await;

        if !success {
            return Err(OperatorError::DeleteApiError {
                message: self.config.error_message.clone(),
            });
        }

        // Remove cluster from state
        let key = (namespace.to_string(), spec.name.clone());
        {
            let mut clusters = self.clusters.lock().await;
            clusters.remove(&key);
        }

        tracing::info!(
            "Mock cluster '{}' deleted from namespace '{}'",
            spec.name,
            namespace
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::cluster::{ApiSpec, DatabaseConfig, SecretReference};

    use super::*;

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

    mod basic_operations {
        use crate::{
            domain::cluster::ClusterPort,
            infrastructure::cluster::repositories::mock::{
                MockClusterRepository,
                tests::{create_custom_spec, create_test_spec},
            },
        };

        #[tokio::test]
        async fn test_successful_apply() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            let result = repo.apply(&spec, "default").await;
            assert!(result.is_ok());

            let status = result.unwrap();
            assert!(!status.ready); // Initially not ready
            assert_eq!(status.phase, Some("Progressing".to_string()));
            assert!(status.message.is_some());

            // Verify cluster exists in repository
            assert!(repo.cluster_exists("default", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 1);
        }

        #[tokio::test]
        async fn test_successful_delete() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // First apply the cluster
            repo.apply(&spec, "default").await.unwrap();
            assert!(repo.cluster_exists("default", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 1);

            // Then delete it
            let result = repo.delete(&spec, "default").await;
            assert!(result.is_ok());

            // Verify cluster is removed
            assert!(!repo.cluster_exists("default", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 0);
        }

        #[tokio::test]
        async fn test_delete_nonexistent_cluster() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Delete without creating first - should succeed (idempotent)
            let result = repo.delete(&spec, "default").await;
            assert!(result.is_ok());
            assert_eq!(repo.cluster_count().await, 0);
        }

        #[tokio::test]
        async fn test_multiple_clusters_different_namespaces() {
            let repo = MockClusterRepository::new();
            let spec1 = create_custom_spec("cluster-1", 1);
            let spec2 = create_custom_spec("cluster-2", 2);

            // Apply clusters in different namespaces
            repo.apply(&spec1, "namespace-1").await.unwrap();
            repo.apply(&spec2, "namespace-2").await.unwrap();

            assert_eq!(repo.cluster_count().await, 2);
            assert!(repo.cluster_exists("namespace-1", "cluster-1").await);
            assert!(repo.cluster_exists("namespace-2", "cluster-2").await);
            assert!(!repo.cluster_exists("namespace-1", "cluster-2").await);
            assert!(!repo.cluster_exists("namespace-2", "cluster-1").await);
        }

        #[tokio::test]
        async fn test_same_cluster_name_different_namespaces() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Apply same cluster spec in different namespaces
            repo.apply(&spec, "ns1").await.unwrap();
            repo.apply(&spec, "ns2").await.unwrap();

            assert_eq!(repo.cluster_count().await, 2);
            assert!(repo.cluster_exists("ns1", "test-cluster").await);
            assert!(repo.cluster_exists("ns2", "test-cluster").await);

            // Delete from one namespace shouldn't affect the other
            repo.delete(&spec, "ns1").await.unwrap();
            assert!(!repo.cluster_exists("ns1", "test-cluster").await);
            assert!(repo.cluster_exists("ns2", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 1);
        }
    }

    mod failure_scenarios {
        use crate::{
            domain::{cluster::ClusterPort, error::OperatorError},
            infrastructure::cluster::repositories::mock::{
                MockClusterConfig, MockClusterRepository, tests::create_test_spec,
            },
        };

        #[tokio::test]
        async fn test_failed_apply() {
            let repo = MockClusterRepository::failing();
            let spec = create_test_spec();

            let result = repo.apply(&spec, "default").await;
            assert!(result.is_err());

            match result {
                Err(OperatorError::ApplyApiError { message }) => {
                    assert_eq!(message, "Mock repository configured to fail");
                }
                _ => panic!("Expected ApplyApiError"),
            }

            // Cluster should not exist after failed apply
            assert!(!repo.cluster_exists("default", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 0);
        }

        #[tokio::test]
        async fn test_failed_delete() {
            let config = MockClusterConfig {
                apply_success: true,
                delete_success: false,
                error_message: "Delete operation failed".to_string(),
                ..MockClusterConfig::default()
            };
            let repo = MockClusterRepository::with_config(config);
            let spec = create_test_spec();

            // Apply should succeed
            repo.apply(&spec, "default").await.unwrap();
            assert!(repo.cluster_exists("default", "test-cluster").await);

            // Delete should fail
            let result = repo.delete(&spec, "default").await;
            assert!(result.is_err());

            match result {
                Err(OperatorError::DeleteApiError { message }) => {
                    assert_eq!(message, "Delete operation failed");
                }
                _ => panic!("Expected DeleteApiError"),
            }

            // Cluster should still exist after failed delete
            assert!(repo.cluster_exists("default", "test-cluster").await);
            assert_eq!(repo.cluster_count().await, 1);
        }

        #[tokio::test]
        async fn test_invalid_spec_empty_name() {
            let repo = MockClusterRepository::new();
            let mut spec = create_test_spec();
            spec.name = "".to_string(); // Invalid empty name

            let result = repo.apply(&spec, "default").await;
            assert!(result.is_err());

            match result {
                Err(OperatorError::InvalidSpec { message }) => {
                    assert!(message.contains("Cluster name cannot be empty"));
                }
                _ => panic!("Expected InvalidSpec error"),
            }
        }

        #[tokio::test]
        async fn test_invalid_spec_zero_replicas() {
            let repo = MockClusterRepository::new();
            let mut spec = create_test_spec();
            spec.replicas = 0; // Invalid zero replicas

            let result = repo.apply(&spec, "default").await;
            assert!(result.is_err());

            match result {
                Err(OperatorError::InvalidSpec { message }) => {
                    assert!(message.contains("Replica count must be greater than 0"));
                }
                _ => panic!("Expected InvalidSpec error"),
            }
        }
    }

    mod state_management {
        use std::time::Duration;

        use crate::{
            domain::cluster::{ClusterPort, ClusterStatus},
            infrastructure::cluster::repositories::mock::{
                MockClusterRepository,
                tests::{create_custom_spec, create_test_spec},
            },
        };

        #[tokio::test]
        async fn test_cluster_state_inspection() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Apply cluster
            repo.apply(&spec, "test-ns").await.unwrap();

            // Get and inspect cluster state
            let state = repo.get_cluster_state("test-ns", "test-cluster").await;
            assert!(state.is_some());

            let state = state.unwrap();
            assert_eq!(state.spec.name, "test-cluster");
            assert_eq!(state.namespace, "test-ns");
            assert_eq!(state.spec.replicas, 2);
            assert!(!state.status.ready);
            assert_eq!(state.status.phase, Some("Progressing".to_string()));

            // Test MockClusterState methods
            assert!(state.age() < Duration::from_secs(1)); // Should be very recent
            assert!(state.is_in_namespace("test-ns"));
            assert!(!state.is_in_namespace("other-ns"));
            assert_eq!(state.cluster_name(), "test-cluster");
        }

        #[tokio::test]
        async fn test_namespace_filtering() {
            let repo = MockClusterRepository::new();
            let spec1 = create_custom_spec("cluster-1", 1);
            let spec2 = create_custom_spec("cluster-2", 2);
            let spec3 = create_custom_spec("cluster-3", 3);

            // Apply clusters in different namespaces
            repo.apply(&spec1, "ns-a").await.unwrap();
            repo.apply(&spec2, "ns-a").await.unwrap();
            repo.apply(&spec3, "ns-b").await.unwrap();

            // Test namespace filtering
            let clusters_ns_a = repo.get_clusters_in_namespace("ns-a").await;
            let clusters_ns_b = repo.get_clusters_in_namespace("ns-b").await;
            let clusters_ns_c = repo.get_clusters_in_namespace("ns-c").await;

            assert_eq!(clusters_ns_a.len(), 2);
            assert_eq!(clusters_ns_b.len(), 1);
            assert_eq!(clusters_ns_c.len(), 0);

            // Verify cluster names in ns-a
            let names_ns_a: Vec<&str> = clusters_ns_a.iter().map(|c| c.cluster_name()).collect();
            assert!(names_ns_a.contains(&"cluster-1"));
            assert!(names_ns_a.contains(&"cluster-2"));

            // Verify cluster name in ns-b
            assert_eq!(clusters_ns_b[0].cluster_name(), "cluster-3");
        }

        #[tokio::test]
        async fn test_cluster_status_updates() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Apply cluster
            repo.apply(&spec, "test-ns").await.unwrap();

            // Verify initial status
            let state = repo
                .get_cluster_state("test-ns", "test-cluster")
                .await
                .unwrap();
            assert!(!state.status.ready);

            // Update status to ready
            let new_status = ClusterStatus {
                ready: true,
                message: Some("Cluster is now ready".to_string()),
                phase: Some("Ready".to_string()),
            };

            let updated = repo
                .update_cluster_status("test-ns", "test-cluster", new_status.clone())
                .await;
            assert!(updated);

            // Verify status was updated
            let state = repo
                .get_cluster_state("test-ns", "test-cluster")
                .await
                .unwrap();
            assert!(state.status.ready);
            assert_eq!(state.status.phase, Some("Ready".to_string()));
            assert_eq!(
                state.status.message,
                Some("Cluster is now ready".to_string())
            );

            // Test updating non-existent cluster
            let not_updated = repo
                .update_cluster_status("test-ns", "non-existent", new_status)
                .await;
            assert!(!not_updated);
        }
    }

    mod operation_logging {
        use crate::{
            domain::cluster::ClusterPort,
            infrastructure::cluster::repositories::mock::{
                MockClusterRepository, tests::create_test_spec,
            },
        };

        #[tokio::test]
        async fn test_operation_logging() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Perform operations
            repo.apply(&spec, "default").await.unwrap();
            repo.delete(&spec, "default").await.unwrap();

            // Check operation log
            let log = repo.get_operation_log().await;
            assert_eq!(log.len(), 2);

            // Verify apply operation
            assert_eq!(log[0].operation_type, "apply");
            assert_eq!(log[0].cluster_name, "test-cluster");
            assert_eq!(log[0].namespace, "default");
            assert!(log[0].success);

            // Verify delete operation
            assert_eq!(log[1].operation_type, "delete");
            assert_eq!(log[1].cluster_name, "test-cluster");
            assert_eq!(log[1].namespace, "default");
            assert!(log[1].success);
        }

        #[tokio::test]
        async fn test_failed_operation_logging() {
            let repo = MockClusterRepository::failing();
            let spec = create_test_spec();

            // Perform failed operations
            let _ = repo.apply(&spec, "default").await;
            let _ = repo.delete(&spec, "default").await;

            // Check operation log
            let log = repo.get_operation_log().await;
            assert_eq!(log.len(), 2);

            // Both operations should be logged as failures
            assert!(!log[0].success);
            assert!(!log[1].success);
        }

        #[tokio::test]
        async fn test_clear_operation_log() {
            let repo = MockClusterRepository::new();
            let spec = create_test_spec();

            // Perform some operations
            repo.apply(&spec, "default").await.unwrap();
            repo.delete(&spec, "default").await.unwrap();

            // Verify operations are logged
            let log = repo.get_operation_log().await;
            assert_eq!(log.len(), 2);

            // Clear log
            repo.clear_operation_log().await;

            // Verify log is cleared
            let log = repo.get_operation_log().await;
            assert_eq!(log.len(), 0);
        }
    }

    mod concurrency {
        use std::sync::Arc;

        use crate::{
            domain::cluster::ClusterPort,
            infrastructure::cluster::repositories::mock::MockClusterRepository,
        };

        use super::*;

        #[tokio::test]
        async fn test_concurrent_apply_operations() {
            let repo = Arc::new(MockClusterRepository::new());

            // Create multiple clusters concurrently
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let repo = repo.clone();
                    let spec = create_custom_spec(&format!("cluster-{}", i), 1);
                    tokio::spawn(async move { repo.apply(&spec, "default").await })
                })
                .collect();

            // Wait for all operations to complete
            let mut success_count = 0;
            for handle in handles {
                if handle.await.unwrap().is_ok() {
                    success_count += 1;
                }
            }

            // All operations should succeed
            assert_eq!(success_count, 10);
            assert_eq!(repo.cluster_count().await, 10);

            // Verify all clusters exist
            for i in 0..10 {
                assert!(
                    repo.cluster_exists("default", &format!("cluster-{}", i))
                        .await
                );
            }
        }

        #[tokio::test]
        async fn test_concurrent_apply_and_delete() {
            let repo = Arc::new(MockClusterRepository::new());
            let spec = create_test_spec();

            // Apply cluster first
            repo.apply(&spec, "default").await.unwrap();

            // Perform concurrent operations on the same cluster
            let repo1 = repo.clone();
            let repo2 = repo.clone();
            let spec1 = spec.clone();
            let spec2 = spec.clone();

            let apply_handle = tokio::spawn(async move { repo1.apply(&spec1, "default").await });

            let delete_handle = tokio::spawn(async move { repo2.delete(&spec2, "default").await });

            // Wait for both operations
            let apply_result = apply_handle.await.unwrap();
            let delete_result = delete_handle.await.unwrap();

            // Both operations should succeed (they're independent)
            assert!(apply_result.is_ok());
            assert!(delete_result.is_ok());
        }

        #[tokio::test]
        async fn test_concurrent_state_inspection() {
            let repo = Arc::new(MockClusterRepository::new());

            // Apply some clusters
            for i in 0..5 {
                let spec = create_custom_spec(&format!("cluster-{}", i), 1);
                repo.apply(&spec, "default").await.unwrap();
            }

            // Perform concurrent state inspections
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let repo = repo.clone();
                    tokio::spawn(async move {
                        let count = repo.cluster_count().await;
                        let exists = repo.cluster_exists("default", "cluster-0").await;
                        let clusters = repo.get_clusters_in_namespace("default").await;
                        (count, exists, clusters.len())
                    })
                })
                .collect();

            // Wait for all inspections to complete
            for handle in handles {
                let (count, exists, namespace_count) = handle.await.unwrap();
                assert_eq!(count, 5);
                assert!(exists);
                assert_eq!(namespace_count, 5);
            }
        }
    }

    mod configuration {
        use std::time::Duration;

        use crate::{
            domain::{cluster::ClusterPort, error::OperatorError},
            infrastructure::cluster::repositories::mock::{
                MockClusterConfig, MockClusterRepository,
            },
        };

        use super::*;

        #[tokio::test]
        async fn test_custom_delay_configuration() {
            let config = MockClusterConfig {
                operation_delay: Duration::from_millis(100),
                ..MockClusterConfig::default()
            };
            let repo = MockClusterRepository::with_config(config);
            let spec = create_test_spec();

            let start = std::time::Instant::now();
            repo.apply(&spec, "default").await.unwrap();
            let duration = start.elapsed();

            // Operation should take at least the configured delay
            assert!(duration >= Duration::from_millis(90)); // Allow some tolerance
        }

        #[tokio::test]
        async fn test_selective_failure_configuration() {
            let config = MockClusterConfig {
                apply_success: false,
                delete_success: true,
                error_message: "Apply failed".to_string(),
                ..MockClusterConfig::default()
            };
            let repo = MockClusterRepository::with_config(config);
            let spec = create_test_spec();

            // Apply should fail
            let apply_result = repo.apply(&spec, "default").await;
            assert!(apply_result.is_err());

            // Delete should succeed (even though cluster doesn't exist)
            let delete_result = repo.delete(&spec, "default").await;
            assert!(delete_result.is_ok());
        }

        #[tokio::test]
        async fn test_custom_error_messages() {
            let config = MockClusterConfig {
                apply_success: false,
                error_message: "Custom error message for testing".to_string(),
                ..MockClusterConfig::default()
            };
            let repo = MockClusterRepository::with_config(config);
            let spec = create_test_spec();

            let result = repo.apply(&spec, "default").await;
            assert!(result.is_err());

            match result {
                Err(OperatorError::ApplyApiError { message }) => {
                    assert_eq!(message, "Custom error message for testing");
                }
                _ => panic!("Expected ApplyApiError with custom message"),
            }
        }
    }

    mod integration {
        use std::time::Duration;

        use super::*;
        use crate::{
            domain::cluster::ClusterPort,
            infrastructure::cluster::repositories::{ClusterRepository, mock::MockClusterConfig},
        };

        #[tokio::test]
        async fn test_repository_enum_integration() {
            let repo = ClusterRepository::mock();
            let spec = create_test_spec();

            // Test through the enum interface
            let result = repo.apply(&spec, "default").await;
            assert!(result.is_ok());

            let delete_result = repo.delete(&spec, "default").await;
            assert!(delete_result.is_ok());
        }

        #[tokio::test]
        async fn test_repository_enum_failing() {
            let repo = ClusterRepository::mock_failing();
            let spec = create_test_spec();

            // Apply should fail
            let result = repo.apply(&spec, "default").await;
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn test_repository_enum_with_config() {
            let config = MockClusterConfig {
                operation_delay: Duration::from_millis(50),
                ..MockClusterConfig::default()
            };
            let repo = ClusterRepository::mock_with_config(config);
            let spec = create_test_spec();

            let start = std::time::Instant::now();
            let result = repo.apply(&spec, "default").await;
            let duration = start.elapsed();

            assert!(result.is_ok());
            assert!(duration >= Duration::from_millis(40)); // Allow some tolerance
        }
    }
}
