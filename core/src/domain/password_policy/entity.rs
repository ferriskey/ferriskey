use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

use super::error::PasswordPolicyError;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PasswordPolicy {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub min_length: i32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
    pub max_age_days: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PasswordPolicy {
    pub fn default(realm_id: Uuid) -> Self {
        Self {
            id: Uuid::now_v7(),
            realm_id,
            min_length: 8,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn validate(&self, password: &str) -> Result<(), Vec<PasswordPolicyError>> {
        let mut errors = Vec::new();

        if password.len() < self.min_length as usize {
            errors.push(PasswordPolicyError::TooShort {
                min: self.min_length,
                actual: password.len(),
            });
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push(PasswordPolicyError::MissingUppercase);
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push(PasswordPolicyError::MissingLowercase);
        }

        if self.require_number && !password.chars().any(|c| c.is_numeric()) {
            errors.push(PasswordPolicyError::MissingNumber);
        }

        if self.require_special
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_password_too_short() {
        let policy = make_policy(8, false, false, false, false);
        let result = policy.validate("abc");

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
        let result = policy.validate("password");

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
        let result = policy.validate("Password1!");

        assert!(result.is_ok());
    }

    #[test]
    fn test_password_multiple_violations() {
        let policy = make_policy(8, true, true, true, true);
        let result = policy.validate("PASS");

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
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePasswordPolicy {
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
    pub max_age_days: Option<i32>,
}
