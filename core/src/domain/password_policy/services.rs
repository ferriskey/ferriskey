use std::sync::Arc;
use uuid::Uuid;
use crate::domain::{
    common::entities::app_errors::CoreError,
    password_policy::{
        entities::{PasswordPolicy, UpdatePasswordPolicy},
        ports::{PasswordPolicyRepository, PasswordPolicyService},
    },
};

pub struct PasswordPolicyServiceImpl<R>
where
    R: PasswordPolicyRepository,
{
    pub(crate) repository: Arc<R>,
}

impl<R> PasswordPolicyServiceImpl<R>
where
    R: PasswordPolicyRepository,
{
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

impl<R> std::fmt::Debug for PasswordPolicyServiceImpl<R>
where
    R: PasswordPolicyRepository,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordPolicyServiceImpl").finish()
    }
}

impl<R> Clone for PasswordPolicyServiceImpl<R>
where
    R: PasswordPolicyRepository,
{
    fn clone(&self) -> Self {
        Self {
            repository: self.repository.clone(),
        }
    }
}

impl<R> PasswordPolicyService for PasswordPolicyServiceImpl<R>
where
    R: PasswordPolicyRepository,
{
    async fn get_policy(&self, realm_id: Uuid) -> Result<PasswordPolicy, CoreError> {
        self.repository
            .find_by_realm_id(realm_id)
            .await?
            .ok_or(CoreError::InvalidRealm)
    }

    async fn update_policy(
        &self,
        realm_id: Uuid,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        self.repository.upsert(realm_id, update).await
    }

    fn validate_password(&self, password: &str, policy: &PasswordPolicy) -> Result<(), CoreError> {
        if password.len() < policy.min_length as usize {
            return Err(CoreError::BadRequest("Password is too short".to_string()));
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(CoreError::BadRequest(
                "Password must contain at least one uppercase letter".to_string(),
            ));
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(CoreError::BadRequest(
                "Password must contain at least one lowercase letter".to_string(),
            ));
        }

        if policy.require_number && !password.chars().any(|c| c.is_numeric()) {
            return Err(CoreError::BadRequest(
                "Password must contain at least one number".to_string(),
            ));
        }

        if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(CoreError::BadRequest(
                "Password must contain at least one special character".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::entities::RealmId;
    use mockall::predicate::*;
    use mockall::mock;
    use futures::future::BoxFuture;

    mock! {
        pub PasswordPolicyRepository {}
        impl PasswordPolicyRepository for PasswordPolicyRepository {
            fn find_by_realm_id(&self, realm_id: Uuid) -> BoxFuture<'static, Result<Option<PasswordPolicy>, CoreError>>;
            fn upsert(&self, realm_id: Uuid, policy: UpdatePasswordPolicy) -> BoxFuture<'static, Result<PasswordPolicy, CoreError>>;
        }
    }

    #[test]
    fn test_validate_password_min_length() {
        let repo = Arc::new(MockPasswordPolicyRepository::new());
        let service = PasswordPolicyServiceImpl::new(repo);
        let realm_id = RealmId::default();
        let mut policy = PasswordPolicy::default(realm_id);
        policy.min_length = 8;

        assert!(service.validate_password("1234567", &policy).is_err());
        assert!(service.validate_password("12345678", &policy).is_ok());
    }

    #[test]
    fn test_validate_password_uppercase() {
        let repo = Arc::new(MockPasswordPolicyRepository::new());
        let service = PasswordPolicyServiceImpl::new(repo);
        let realm_id = RealmId::default();
        let mut policy = PasswordPolicy::default(realm_id);
        policy.require_uppercase = true;

        assert!(service.validate_password("password", &policy).is_err());
        assert!(service.validate_password("Password", &policy).is_ok());
    }
}
