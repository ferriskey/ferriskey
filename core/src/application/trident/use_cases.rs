use crate::application::{
    common::services::ServiceBundle,
    trident::use_cases::{
        challenge_otp_use_case::ChallengeOtpUseCase, setup_otp_use_case::SetupOtpUseCase,
        update_password_use_case::UpdatePasswordUseCase, verify_otp_use_case::VerifyOtpUseCase,
    },
};

pub mod challenge_otp_use_case;
pub mod setup_otp_use_case;
pub mod update_password_use_case;
pub mod verify_otp_use_case;

pub struct TridentUseCase {
    pub update_password_use_case: UpdatePasswordUseCase,
    pub verify_otp_use_case: VerifyOtpUseCase,
    pub setup_otp_use_case: SetupOtpUseCase,
    pub challenge_otp_use_case: ChallengeOtpUseCase,
}

impl TridentUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            update_password_use_case: UpdatePasswordUseCase::new(
                service_bundle.credential_service.clone(),
                service_bundle.user_service.clone(),
            ),
            verify_otp_use_case: VerifyOtpUseCase::new(
                service_bundle.totp_service.clone(),
                service_bundle.credential_service.clone(),
                service_bundle.user_service.clone(),
            ),
            setup_otp_use_case: SetupOtpUseCase::new(service_bundle.totp_service.clone()),
            challenge_otp_use_case: ChallengeOtpUseCase::new(
                service_bundle.auth_session_service.clone(),
                service_bundle.credential_service.clone(),
                service_bundle.totp_service.clone(),
            ),
        }
    }
}
