use crate::{
    application::common::services::{DefaultCredentialService, DefaultUserService},
    domain::{
        authentication::value_objects::Identity,
        credential::ports::CredentialService,
        user::{
            entities::{RequiredAction, UserError},
            ports::UserService,
        },
    },
};

#[derive(Clone)]
pub struct UpdatePasswordUseCase {
    credential_service: DefaultCredentialService,
    user_service: DefaultUserService,
}

pub struct UpdatePasswordUseCaseParams {
    pub realm_name: String,
    pub value: String,
}

impl UpdatePasswordUseCase {
    pub fn new(
        credential_service: DefaultCredentialService,
        user_service: DefaultUserService,
    ) -> Self {
        Self {
            credential_service,
            user_service,
        }
    }

    pub async fn execute(
        &self,
        identity: Identity,
        params: UpdatePasswordUseCaseParams,
    ) -> Result<(), UserError> {
        let user = match identity {
            Identity::User(user) => user,
            _ => return Err(UserError::Forbidden("is not user".to_string())),
        };

        self.credential_service
            .reset_password(user.id, params.value, false)
            .await
            .map_err(|_| UserError::InternalServerError)?;

        self.user_service
            .remove_required_action(user.id, RequiredAction::UpdatePassword)
            .await?;

        Ok(())
    }
}
