use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::generate_uuid_v7;
use crate::domain::realm::entities::RealmId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FlowId(pub Uuid);

impl FlowId {
    pub fn new() -> Self {
        FlowId(generate_uuid_v7())
    }
}

impl Default for FlowId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for FlowId {
    fn from(value: Uuid) -> Self {
        FlowId(value)
    }
}

impl From<FlowId> for Uuid {
    fn from(value: FlowId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct FlowStepId(pub Uuid);

impl FlowStepId {
    pub fn new() -> Self {
        FlowStepId(generate_uuid_v7())
    }
}

impl Default for FlowStepId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for FlowStepId {
    fn from(value: Uuid) -> Self {
        FlowStepId(value)
    }
}

impl From<FlowStepId> for Uuid {
    fn from(value: FlowStepId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum FlowStatus {
    #[serde(rename = "pending")]
    Pending,

    #[serde(rename = "success")]
    Success,

    #[serde(rename = "failure")]
    Failure,

    #[serde(rename = "expired")]
    Expired,
}

impl Display for FlowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowStatus::Pending => write!(f, "pending"),
            FlowStatus::Success => write!(f, "success"),
            FlowStatus::Failure => write!(f, "failure"),
            FlowStatus::Expired => write!(f, "expired"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum StepStatus {
    #[serde(rename = "success")]
    Success,

    #[serde(rename = "failure")]
    Failure,

    #[serde(rename = "skipped")]
    Skipped,
}

impl Display for StepStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StepStatus::Success => write!(f, "success"),
            StepStatus::Failure => write!(f, "failure"),
            StepStatus::Skipped => write!(f, "skipped"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub enum FlowStepName {
    #[serde(rename = "authorize")]
    Authorize,

    #[serde(rename = "credential_validation")]
    CredentialValidation,

    #[serde(rename = "mfa_challenge")]
    MfaChallenge,

    #[serde(rename = "token_exchange")]
    TokenExchange,

    #[serde(rename = "token_refresh")]
    TokenRefresh,

    #[serde(rename = "finalize")]
    Finalize,
}

impl Display for FlowStepName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlowStepName::Authorize => write!(f, "authorize"),
            FlowStepName::CredentialValidation => write!(f, "credential_validation"),
            FlowStepName::MfaChallenge => write!(f, "mfa_challenge"),
            FlowStepName::TokenExchange => write!(f, "token_exchange"),
            FlowStepName::TokenRefresh => write!(f, "token_refresh"),
            FlowStepName::Finalize => write!(f, "finalize"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CompassFlow {
    pub id: FlowId,
    pub realm_id: RealmId,
    pub client_id: Option<String>,
    pub user_id: Option<Uuid>,
    pub grant_type: String,
    pub status: FlowStatus,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub steps: Vec<CompassFlowStep>,
}

impl CompassFlow {
    pub fn new(
        realm_id: RealmId,
        client_id: Option<String>,
        grant_type: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            id: FlowId::new(),
            realm_id,
            client_id,
            user_id: None,
            grant_type,
            status: FlowStatus::Pending,
            ip_address,
            user_agent,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
            steps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CompassFlowStep {
    pub id: FlowStepId,
    pub flow_id: FlowId,
    pub step_name: String,
    pub status: StepStatus,
    pub duration_ms: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
}

impl CompassFlowStep {
    pub fn new(
        flow_id: FlowId,
        step_name: FlowStepName,
        status: StepStatus,
        duration_ms: Option<i64>,
        error_code: Option<String>,
        error_message: Option<String>,
        started_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: FlowStepId::new(),
            flow_id,
            step_name: step_name.to_string(),
            status,
            duration_ms,
            error_code,
            error_message,
            started_at,
        }
    }
}
