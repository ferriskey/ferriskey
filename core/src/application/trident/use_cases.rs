use crate::application::{
    common::services::ServiceBundle,
    trident::use_cases::update_password_use_case::UpdatePasswordUseCase,
};

pub mod update_password_use_case;

pub struct TridentUseCase {
    pub update_password_use_case: UpdatePasswordUseCase,
}

impl TridentUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            update_password_use_case: UpdatePasswordUseCase::new(
                service_bundle.credential_service.clone(),
                service_bundle.user_service.clone(),
            ),
        }
    }
}
