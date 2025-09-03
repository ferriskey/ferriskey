use uuid::Uuid;

use crate::{
    application::common::services::{DefaultAuthSessionService, DefaultCredentialService},
    domain::{
        authentication::{ports::AuthSessionService, value_objects::Identity},
        common::generate_random_string,
        credential::ports::CredentialService,
        trident::{
            entities::{TotpError, TotpSecret},
            ports::TotpService,
            services::OauthTotpService,
        },
    },
};

pub struct ChallengeOtpUseCaseInput {
    pub identity: Identity,
    pub session_code: String,
    pub code: String,
}
pub struct ChallengeOtpUseCaseOutput {
    pub login_url: String,
}

#[derive(Clone)]
pub struct ChallengeOtpUseCase {
    auth_session_service: DefaultAuthSessionService,
    credential_service: DefaultCredentialService,
    totp_service: OauthTotpService,
}

impl ChallengeOtpUseCase {
    pub fn new(
        auth_session_service: DefaultAuthSessionService,
        credential_service: DefaultCredentialService,
        totp_service: OauthTotpService,
    ) -> Self {
        Self {
            auth_session_service,
            credential_service,
            totp_service,
        }
    }

    pub async fn execute(
        &self,
        input: ChallengeOtpUseCaseInput,
    ) -> Result<ChallengeOtpUseCaseOutput, TotpError> {
        let session_code = Uuid::parse_str(&input.session_code)
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        let user = match input.identity {
            Identity::User(user) => user,
            _ => {
                return Err(TotpError::VerificationFailed(
                    "Invalid identity".to_string(),
                ));
            }
        };

        let auth_session = self
            .auth_session_service
            .get_by_session_code(session_code)
            .await
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        let user_credentials = self
            .credential_service
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        let otp_credential = user_credentials
            .iter()
            .find(|cred| cred.credential_type == "otp")
            .ok_or_else(|| {
                TotpError::VerificationFailed("User has not OTP configured".to_string())
            })?;

        let secret = TotpSecret::from_base32(&otp_credential.secret_data);

        let is_valid = self
            .totp_service
            .verify(&secret, &input.code)
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        if !is_valid {
            tracing::error!("invalid OTP code for user: {}", user.email);
            return Err(TotpError::VerificationFailed(
                "failed to verify OTP".to_string(),
            ));
        }

        let authorization_code = generate_random_string();

        self.auth_session_service
            .update_code(session_code, authorization_code.clone(), user.id)
            .await
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        let current_state = auth_session.state.ok_or(TotpError::VerificationFailed(
            "invalid session state".to_string(),
        ))?;

        let login_url = format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, current_state
        );

        Ok(ChallengeOtpUseCaseOutput { login_url })
    }
}
