use chrono::{TimeZone, Utc};
use sea_orm::ActiveValue::Set;

use ferriskey_compass::entities::{
    CompassFlow, CompassFlowStep, FlowId, FlowStatus, FlowStepId, FlowStepName, StepStatus,
};

use crate::entity::{compass_flow_steps, compass_flows};

impl From<compass_flows::Model> for CompassFlow {
    fn from(model: compass_flows::Model) -> Self {
        let status = match model.status.as_str() {
            "success" => FlowStatus::Success,
            "failure" => FlowStatus::Failure,
            "expired" => FlowStatus::Expired,
            _ => FlowStatus::Pending,
        };

        CompassFlow {
            id: FlowId::from(model.id),
            realm_id: model.realm_id.into(),
            client_id: model.client_id,
            user_id: model.user_id,
            grant_type: model.grant_type,
            status,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            started_at: Utc.from_utc_datetime(&model.started_at),
            completed_at: model.completed_at.map(|dt| Utc.from_utc_datetime(&dt)),
            duration_ms: model.duration_ms,
            steps: Vec::new(),
        }
    }
}

impl From<CompassFlow> for compass_flows::ActiveModel {
    fn from(flow: CompassFlow) -> Self {
        compass_flows::ActiveModel {
            id: Set(flow.id.into()),
            realm_id: Set(flow.realm_id.into()),
            client_id: Set(flow.client_id),
            user_id: Set(flow.user_id),
            grant_type: Set(flow.grant_type),
            status: Set(flow.status.to_string()),
            ip_address: Set(flow.ip_address),
            user_agent: Set(flow.user_agent),
            started_at: Set(flow.started_at.naive_utc()),
            completed_at: Set(flow.completed_at.map(|dt| dt.naive_utc())),
            duration_ms: Set(flow.duration_ms),
            created_at: Set(Utc::now().naive_utc()),
        }
    }
}

impl From<compass_flow_steps::Model> for CompassFlowStep {
    fn from(model: compass_flow_steps::Model) -> Self {
        let step_name = match model.step_name.as_str() {
            "credential_validation" => FlowStepName::CredentialValidation,
            "mfa_challenge" => FlowStepName::MfaChallenge,
            "token_exchange" => FlowStepName::TokenExchange,
            "idp_redirect" => FlowStepName::IdpRedirect,
            "idp_callback" => FlowStepName::IdpCallback,
            "finalize" => FlowStepName::Finalize,
            _ => FlowStepName::Authorize,
        };

        let status = match model.status.as_str() {
            "failure" => StepStatus::Failure,
            "skipped" => StepStatus::Skipped,
            _ => StepStatus::Success,
        };

        CompassFlowStep {
            id: FlowStepId::from(model.id),
            flow_id: FlowId::from(model.flow_id),
            step_name,
            status,
            duration_ms: model.duration_ms,
            error_code: model.error_code,
            error_message: model.error_message,
            started_at: Utc.from_utc_datetime(&model.started_at),
        }
    }
}

impl From<CompassFlowStep> for compass_flow_steps::ActiveModel {
    fn from(step: CompassFlowStep) -> Self {
        compass_flow_steps::ActiveModel {
            id: Set(step.id.into()),
            flow_id: Set(step.flow_id.into()),
            step_name: Set(step.step_name.to_string()),
            status: Set(step.status.to_string()),
            duration_ms: Set(step.duration_ms),
            error_code: Set(step.error_code),
            error_message: Set(step.error_message),
            started_at: Set(step.started_at.naive_utc()),
            created_at: Set(Utc::now().naive_utc()),
        }
    }
}
