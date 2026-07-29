use ferriskey_core::domain::authentication::entities::{
    AuthenticateOutput, AuthenticationStepStatus,
};
use ferriskey_core::domain::user::entities::RequiredAction;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub enum AuthenticationStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
pub struct AuthenticateResponse {
    pub status: AuthenticationStatus,
    pub url: Option<String>,
    pub required_actions: Option<Vec<RequiredAction>>,
    pub token: Option<String>,
    pub message: Option<String>,
}

impl From<AuthenticateOutput> for AuthenticateResponse {
    fn from(result: AuthenticateOutput) -> Self {
        match result.status {
            AuthenticationStepStatus::Success => AuthenticateResponse {
                status: AuthenticationStatus::Success,
                url: result.redirect_url,
                required_actions: None,
                token: None,
                message: Some("Authentication successful".to_string()),
            },
            AuthenticationStepStatus::RequiresActions => AuthenticateResponse {
                status: AuthenticationStatus::RequiresActions,
                url: None,
                required_actions: if result.required_actions.is_empty() {
                    None
                } else {
                    Some(result.required_actions)
                },
                token: result.temporary_token,
                message: Some("Additional actions required before login".to_string()),
            },
            AuthenticationStepStatus::RequiresOtpChallenge => AuthenticateResponse {
                status: AuthenticationStatus::RequiresOtpChallenge,
                url: None,
                required_actions: None,
                token: result.temporary_token,
                message: Some("OTP verification required".to_string()),
            },
            AuthenticationStepStatus::Failed => AuthenticateResponse {
                status: AuthenticationStatus::Failed,
                url: None,
                required_actions: None,
                token: None,
                message: Some("Authentication failed".to_string()),
            },
        }
    }
}
