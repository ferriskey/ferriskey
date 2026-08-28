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
            "saml_authn_request" => FlowStepName::SamlAuthnRequest,
            "saml_assertion" => FlowStepName::SamlAssertion,
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

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    fn wire_name(step_name: &FlowStepName) -> &'static str {
        match step_name {
            FlowStepName::Authorize => "authorize",
            FlowStepName::CredentialValidation => "credential_validation",
            FlowStepName::MfaChallenge => "mfa_challenge",
            FlowStepName::TokenExchange => "token_exchange",
            FlowStepName::IdpRedirect => "idp_redirect",
            FlowStepName::IdpCallback => "idp_callback",
            FlowStepName::Finalize => "finalize",
            FlowStepName::SamlAuthnRequest => "saml_authn_request",
            FlowStepName::SamlAssertion => "saml_assertion",
        }
    }

    fn stored_step(step_name: &str) -> compass_flow_steps::Model {
        compass_flow_steps::Model {
            id: Uuid::new_v4(),
            flow_id: Uuid::new_v4(),
            step_name: step_name.to_string(),
            status: "success".to_string(),
            duration_ms: None,
            error_code: None,
            error_message: None,
            started_at: Utc::now().naive_utc(),
            created_at: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn a_step_reads_back_as_the_step_that_was_written() {
        let step_names = [
            FlowStepName::Authorize,
            FlowStepName::CredentialValidation,
            FlowStepName::MfaChallenge,
            FlowStepName::TokenExchange,
            FlowStepName::IdpRedirect,
            FlowStepName::IdpCallback,
            FlowStepName::Finalize,
            FlowStepName::SamlAuthnRequest,
            FlowStepName::SamlAssertion,
        ];

        for step_name in step_names {
            let written = step_name.to_string();
            assert_eq!(written, wire_name(&step_name));

            let read = CompassFlowStep::from(stored_step(&written));
            assert_eq!(
                read.step_name, step_name,
                "`{written}` must not read back as another step"
            );
        }
    }
}
