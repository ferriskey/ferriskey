#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterSpec {
    pub name: String,
    pub version: String,
    pub replicas: u32,
    pub database: DatabaseConfig,

    pub api: ApiSpec,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiSpec {
    pub webapp_url: String,
    pub api_url: String,
    pub allowed_origins: Vec<String>,
}

impl Default for ApiSpec {
    fn default() -> Self {
        ApiSpec {
            webapp_url: "http://localhost:3000".to_string(),
            api_url: "http://localhost:8080".to_string(),
            allowed_origins: vec!["http://localhost:3000".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseConfig {
    pub secret_ref: SecretReference,
    /// Optional: Database name override (if not specified in secret)
    pub database_name: Option<String>,
    /// Optional: SSL mode for database connection
    pub ssl_mode: Option<String>, // e.g., "require", "disable", "prefer"}
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            secret_ref: SecretReference {
                name: "db-credentials".to_string(),
                namespace: None,
            },
            database_name: Some("ferriskey".to_string()),
            ssl_mode: Some("require".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretReference {
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterStatus {
    pub ready: bool,
    pub message: Option<String>,
    pub phase: Option<String>,
}

impl Default for ClusterStatus {
    fn default() -> Self {
        ClusterStatus {
            ready: true,
            message: Some("Cluster applied successfully".to_string()),
            phase: Some("Running".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ClusterAction {
    Create,
    Update,
    NoOp,
}
