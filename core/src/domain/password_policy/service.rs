use std::sync::Arc;

use uuid::Uuid;

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
use super::error::{PasswordPolicyError, PasswordPolicyViolation};
use super::ports::PasswordPolicyPolicy;
use super::repository::PasswordPolicyRepository;
use super::validator;

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
        ensure_policy(
            self.policy.can_view_password_policy(&identity, realm).await,
            "view realm password policy",
        )?;

        self.repository
            .find_by_realm_id(realm.id.into())
            .await?
            .map(Ok)
            .unwrap_or_else(|| Ok(PasswordPolicy::default(realm.id.into())))
    }

    pub async fn update_policy(
        &self,
        identity: Identity,
        realm: &Realm,
        update: UpdatePasswordPolicy,
    ) -> Result<PasswordPolicy, CoreError> {
        ensure_policy(
            self.policy
                .can_update_password_policy(&identity, realm)
                .await,
            "update realm password policy",
        )?;

        self.repository.upsert(realm.id.into(), update).await
    }

    pub async fn enforce(&self, realm_id: Uuid, password: &str) -> Result<(), CoreError> {
        let policy = self
            .repository
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(realm_id));
        // Use the full validator (entropy + common-password checks) so the pre-flight
        // check matches the enforcement done later in `reset_password`. Username/email
        // context is unavailable here, so those similarity checks run only in the
        // credential flow where the target user is known.
        validator::validate(password, &policy, None, None).map_err(|errors| {
            let violations: Vec<PasswordPolicyViolation> = errors.iter().map(Into::into).collect();
            CoreError::PasswordPolicyViolation(
                serde_json::to_string(&violations).unwrap_or_default(),
            )
        })
    }

    pub async fn get_policy_public(&self, realm_id: Uuid) -> Result<PasswordPolicy, CoreError> {
        Ok(self
            .repository
            .find_by_realm_id(realm_id)
            .await?
            .unwrap_or_else(|| PasswordPolicy::default(realm_id)))
    }

    /// Validate a password against a policy (character class + entropy + common-password rules).
    ///
    /// Returns `Ok(())` on success, or an error whose message lists every violated rule.
    pub fn validate_password(
        password: &str,
        policy: &PasswordPolicy,
    ) -> Result<(), Vec<PasswordPolicyError>> {
        validator::validate(password, policy, None, None)
    }

    /// Validate a password against a policy, providing user context for the
    /// `forbid_common` username/email check.
    pub fn validate_password_with_context(
        password: &str,
        policy: &PasswordPolicy,
        username: Option<&str>,
        email_local: Option<&str>,
    ) -> Result<(), Vec<PasswordPolicyError>> {
        validator::validate(password, policy, username, email_local)
    }
}

/// Convert a list of policy violations into a single [`CoreError`] whose message
/// enumerates all failed rules, separated by "; ".
pub fn violations_to_core_error(violations: Vec<PasswordPolicyError>) -> CoreError {
    let msg = violations
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    CoreError::PasswordPolicyViolation(msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    use crate::domain::{
        client::ports::MockClientRepository,
        user::ports::{MockUserRepository, MockUserRoleRepository},
    };

    fn make_policy(
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
            min_entropy_bits: 0,
            forbid_common: false,
            check_breached: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_password_too_short() {
        let policy = make_policy(8, false, false, false, false);
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
        let policy = make_policy(8, true, false, false, false);
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
        let policy = make_policy(8, true, true, true, true);
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
        let policy = make_policy(8, true, true, true, true);
        let result = PasswordPolicyService::<
            MockRepository,
            MockUserRepository,
            MockClientRepository,
            MockUserRoleRepository,
        >::validate_password("PASS", &policy);

        assert!(result.is_err());
        let errors = result.unwrap_err();
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

    #[test]
    fn violations_to_core_error_formats_message() {
        let violations = vec![
            PasswordPolicyError::MissingUppercase,
            PasswordPolicyError::MissingNumber,
        ];
        let err = violations_to_core_error(violations);
        assert!(matches!(err, CoreError::PasswordPolicyViolation(_)));
        if let CoreError::PasswordPolicyViolation(msg) = err {
            assert!(msg.contains("uppercase"));
            assert!(msg.contains("number"));
        }
    }

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
