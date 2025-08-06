use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum HealthCheckError {
    #[error("Database connection error: {0}")]
    DatabaseConnectionError(String),

    #[error("Cache connection error: {0}")]
    CacheConnectionError(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Unknown health check error")]
    Unknown,
}

#[derive(Clone, Serialize, PartialEq, Debug)]
pub struct DatabaseHealthStatus {
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ReadinessResponse {
    pub status: String,
    pub database: DatabaseHealthStatus,
    pub timestamp: String,
}