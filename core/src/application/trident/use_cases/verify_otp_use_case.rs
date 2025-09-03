use uuid::Uuid;

use crate::{
    application::common::services::{DefaultCredentialService, DefaultUserService},
    domain::{
        authentication::value_objects::Identity,
        credential::ports::CredentialService,
        trident::{
            entities::{TotpError, TotpSecret},
            ports::TotpService,
            services::OauthTotpService,
        },
        user::ports::UserService,
    },
};

pub struct VerifyOtpUseCaseInput {
    pub identity: Identity,
    pub secret: String,
    pub code: String,
    pub label: Option<String>,
}

pub struct VerifyOtpUseCaseOutput {
    pub message: String,
    pub user_id: Uuid,
}

#[derive(Clone)]
pub struct VerifyOtpUseCase {
    totp_service: OauthTotpService,
    credential_service: DefaultCredentialService,
    user_service: DefaultUserService,
}

impl VerifyOtpUseCase {
    pub fn new(
        totp_service: OauthTotpService,
        credential_service: DefaultCredentialService,
        user_service: DefaultUserService,
    ) -> Self {
        Self {
            totp_service,
            credential_service,
            user_service,
        }
    }

    pub async fn execute(
        &self,
        input: VerifyOtpUseCaseInput,
    ) -> Result<VerifyOtpUseCaseOutput, TotpError> {
        let decoded = base32::decode(base32::Alphabet::Rfc4648 { padding: false }, &input.secret)
            .ok_or(TotpError::InvalidSecretFormat)?;

        if decoded.len() != 20 {
            return Err(TotpError::InvalidSecretFormat);
        }

        let user = match input.identity {
            Identity::User(user) => user,
            _ => return Err(TotpError::InvalidUser),
        };

        let secret = TotpSecret::from_base32(&input.secret);

        let is_valid = self
            .totp_service
            .verify(&secret, &input.code)
            .map_err(|e| TotpError::VerificationFailed(e.to_string()))?;

        if !is_valid {
            tracing::error!("invalid OTP code");

            return Err(TotpError::InvalidSecretFormat);
        }

        let credential_data = serde_json::json!({
          "subType": "totp",
          "digits": 6,
          "counter": 0,
          "period": 30,
          "algorithm": "HmacSha256",
        });

        self.credential_service
            .create_custom_credential(
                user.id,
                "otp".to_string(),
                secret.base32_encoded().to_string(),
                input.label,
                credential_data,
            )
            .await
            .map_err(|e| TotpError::GenerationFailed(e.to_string()))?;

        self.user_service
            .remove_required_action(
                user.id,
                crate::domain::user::entities::RequiredAction::ConfigureOtp,
            )
            .await
            .map_err(|e| TotpError::GenerationFailed(e.to_string()))?;

        Ok(VerifyOtpUseCaseOutput {
            message: "OTP verified successfully".to_string(),
            user_id: user.id,
        })
    }
}
