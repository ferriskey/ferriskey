use crate::application::common::services::DefaultHealthCheckService;
use crate::domain::health::entities::{DatabaseHealthStatus, HealthCheckError, ReadinessResponse};
use crate::domain::health::ports::HealthCheckService;
use chrono::Utc;

#[derive(Clone)]
pub struct HealthCheckUseCase {
    health_check_service: DefaultHealthCheckService,
}

impl HealthCheckUseCase {
    pub fn new(health_check_service: DefaultHealthCheckService) -> Self {
        Self {
            health_check_service,
        }
    }

    pub async fn execute_readiness(&self) -> Result<ReadinessResponse, HealthCheckError> {
        let database_health = match self.health_check_service.check_database_status().await {
            Ok(health) => health,
            Err(e) => DatabaseHealthStatus {
                status: "unhealthy".to_string(),
                response_time_ms: None,
                error: Some(e.to_string()),
            },
        };

        let overall_status = match database_health.status.as_str() {
            "ok" | "healthy" => "ok".to_string(),
            _ => "unhealthy".to_string(),
        };

        let is_healthy = overall_status == "ok";

        Ok(ReadinessResponse {
            is_healthy,
            status: overall_status,
            timestamp: Utc::now().to_rfc3339(),
            database: database_health,
        })
    }

    pub async fn execute_liveness(&self) -> Result<(), HealthCheckError> {
        self.health_check_service.check_health().await.map(|_| ())
    }
}
