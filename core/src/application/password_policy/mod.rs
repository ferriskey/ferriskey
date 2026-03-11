use crate::{
    ApplicationService,
    domain::{
        common::entities::app_errors::CoreError,
        password_policy::{
            entities::{PasswordPolicy, UpdatePasswordPolicy},
            ports::PasswordPolicyService,
        },
    },
};

impl PasswordPolicyService for ApplicationService {
    async fn get_policy(&self, realm_id: uuid::Uuid) -> Result<PasswordPolicy, CoreError> {
        self.password_policy_service.get_policy(realm_id).await
    }

    async fn update_policy(
        &self,
        realm_id: uuid::Uuid,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        self.password_policy_service
            .update_policy(realm_id, update)
            .await
    }

    fn validate_password(&self, password: &str, policy: &PasswordPolicy) -> Result<(), CoreError> {
        self.password_policy_service
            .validate_password(password, policy)
    }
}
