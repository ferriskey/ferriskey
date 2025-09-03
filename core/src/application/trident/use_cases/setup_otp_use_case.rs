use crate::domain::{
    authentication::value_objects::Identity,
    trident::{entities::TotpError, ports::TotpService, services::OauthTotpService},
};

pub struct SetupOtpUseCaseInput {
    pub identity: Identity,
    pub issuer: String,
}

pub struct SetupOtpUseCaseOutput {
    pub secret: String,
    pub otpauth_url: String,
}

#[derive(Clone)]
pub struct SetupOtpUseCase {
    totp_service: OauthTotpService,
}

impl SetupOtpUseCase {
    pub fn new(totp_service: OauthTotpService) -> Self {
        Self { totp_service }
    }

    pub async fn execute(
        &self,
        input: SetupOtpUseCaseInput,
    ) -> Result<SetupOtpUseCaseOutput, TotpError> {
        let user = match input.identity {
            Identity::User(user) => user,
            _ => return Err(TotpError::InvalidUser),
        };

        let secret = self
            .totp_service
            .generate_secret()
            .map_err(|e| TotpError::GenerationFailed(e.to_string()))?;
        let otpauth_url =
            self.totp_service
                .generate_otpauth_uri(&input.issuer, &user.email, &secret);

        Ok(SetupOtpUseCaseOutput {
            otpauth_url,
            secret: secret.base32_encoded().to_string(),
        })
    }
}
