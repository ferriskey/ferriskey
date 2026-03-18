use std::sync::Arc;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    realm::entities::Realm,
    user::ports::{UserRepository, UserRoleRepository},
};

use super::entity::{PasswordPolicy, UpdatePasswordPolicy};
use super::error::PasswordPolicyError;
use super::ports::PasswordPolicyPolicy;
use super::repository::PasswordPolicyRepository;

#[derive(Debug, Clone)]
pub struct PasswordPolicyService<R, U, C, UR>
where
    R: PasswordPolicyRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    repository: Arc<R>,
    policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR> PasswordPolicyService<R, U, C, UR>
where
    R: PasswordPolicyRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
{
    pub fn new(repository: Arc<R>, policy: Arc<FerriskeyPolicy<U, C, UR>>) -> Self {
        Self { repository, policy }
    }

    pub async fn get_policy(
        &self,
        identity: Identity,
        realm: &Realm,
    ) -> Result<PasswordPolicy, CoreError> {
        // Check authorization
        ensure_policy(
            self.policy.can_view_password_policy(&identity, realm).await,
            "view realm password policy",
        )?;

        self.repository
            .find_by_realm_id(realm.id.into())
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn update_policy(
        &self,
        identity: Identity,
        realm: &Realm,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        // Check authorization
        ensure_policy(
            self.policy
                .can_update_password_policy(&identity, realm)
                .await,
            "update realm password policy",
        )?;

        self.repository.upsert(realm.id.into(), update).await
    }

    pub fn validate_password(
        password: &str,
        policy: &PasswordPolicy,
    ) -> Result<(), Vec<PasswordPolicyError>> {
        let mut errors = Vec::new();

        // Check minimum length
        if password.len() < policy.min_length as usize {
            errors.push(PasswordPolicyError::TooShort {
                min: policy.min_length,
                actual: password.len(),
            });
        }

        // Check uppercase requirement
        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push(PasswordPolicyError::MissingUppercase);
        }

        // Check lowercase requirement
        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push(PasswordPolicyError::MissingLowercase);
        }

        // Check number requirement
        if policy.require_number && !password.chars().any(|c| c.is_numeric()) {
            errors.push(PasswordPolicyError::MissingNumber);
        }

        // Check special character requirement
        if policy.require_special
            && !password
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;':\"\",./<>?".contains(c))
        {
            errors.push(PasswordPolicyError::MissingSpecialCharacter);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    // Mock types for testing
    use crate::domain::{
        client::ports::MockClientRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
    };

    fn create_test_policy(
        min_length: i32,
        require_uppercase: bool,
        require_lowercase: bool,
        require_number: bool,
        require_special: bool,
    ) -> PasswordPolicy {
        PasswordPolicy {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            min_length,
            require_uppercase,
            require_lowercase,
            require_number,
            require_special,
            max_age_days: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_password_too_short() {
        let policy = create_test_policy(8, false, false, false, false);
        let result = PasswordPolicyService::<
            MockRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
        >::validate_password("abc", &policy);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            PasswordPolicyError::TooShort { min: 8, actual: 3 }
        ));
    }

    #[test]
    fn test_password_missing_uppercase() {
        let policy = create_test_policy(8, true, false, false, false);
        let result = PasswordPolicyService::<
            MockRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
        >::validate_password("password", &policy);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingUppercase))
        );
    }

    #[test]
    fn test_password_meets_all_requirements() {
        let policy = create_test_policy(8, true, true, true, true);
        let result = PasswordPolicyService::<
            MockRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
        >::validate_password("Password1!", &policy);

        assert!(result.is_ok());
    }

    #[test]
    fn test_password_multiple_violations() {
        let policy = create_test_policy(8, true, true, true, true);
        let result = PasswordPolicyService::<
            MockRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
        >::validate_password("PASS", &policy);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 4);
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::TooShort { .. }))
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingLowercase))
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingNumber))
        );
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingSpecialCharacter))
        );
    }

    // Mock repository for tests
    struct MockRepository;
    impl PasswordPolicyRepository for MockRepository {
        async fn find_by_realm_id(
            &self,
            _realm_id: Uuid,
        ) -> Result<Option<PasswordPolicy>, CoreError> {
            Ok(None)
        }

        async fn upsert(
            &self,
            _realm_id: Uuid,
            _update: UpdatePasswordPolicy,
        ) -> Result<PasswordPolicy, CoreError> {
            Err(CoreError::NotFound)
        }
    }
}
