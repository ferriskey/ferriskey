use std::fmt::Display;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordPolicyError {
    TooShort { min: i32, actual: usize },
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialCharacter,
    InsufficientEntropy { min_bits: f64, actual_bits: f64 },
    CommonPassword,
    BreachedPassword,
}

impl PasswordPolicyError {
    pub fn code(&self) -> &'static str {
        match self {
            PasswordPolicyError::TooShort { .. } => "too_short",
            PasswordPolicyError::MissingUppercase => "missing_uppercase",
            PasswordPolicyError::MissingLowercase => "missing_lowercase",
            PasswordPolicyError::MissingNumber => "missing_number",
            PasswordPolicyError::MissingSpecialCharacter => "missing_special",
            PasswordPolicyError::InsufficientEntropy { .. } => "insufficient_entropy",
            PasswordPolicyError::CommonPassword => "common_password",
            PasswordPolicyError::BreachedPassword => "breached_password",
        }
    }
}

impl Display for PasswordPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordPolicyError::TooShort { min, actual } => {
                write!(
                    f,
                    "Password is too short: {} characters (minimum {} required)",
                    actual, min
                )
            }
            PasswordPolicyError::MissingUppercase => {
                write!(f, "Password must contain at least one uppercase letter")
            }
            PasswordPolicyError::MissingLowercase => {
                write!(f, "Password must contain at least one lowercase letter")
            }
            PasswordPolicyError::MissingNumber => {
                write!(f, "Password must contain at least one number")
            }
            PasswordPolicyError::MissingSpecialCharacter => {
                write!(f, "Password must contain at least one special character")
            }
            // Deliberately jargon-free: this string is rendered as-is on the
            // login and update-password screens. The bit counts stay on the
            // variant for logs and tests, but telling an end user their
            // password has "47.6 bits of entropy" tells them nothing about
            // what to type instead — see issue #1302.
            PasswordPolicyError::InsufficientEntropy { .. } => {
                write!(
                    f,
                    "Password is not strong enough. Make it longer, or mix uppercase and lowercase letters, numbers and symbols"
                )
            }
            PasswordPolicyError::CommonPassword => {
                write!(
                    f,
                    "Password is too common or matches user credentials; please choose a stronger password"
                )
            }
            PasswordPolicyError::BreachedPassword => {
                write!(
                    f,
                    "Password has appeared in a data breach; please choose a different password"
                )
            }
        }
    }
}

impl std::error::Error for PasswordPolicyError {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PasswordPolicyViolation {
    pub code: String,
    pub message: String,
}

impl From<&PasswordPolicyError> for PasswordPolicyViolation {
    fn from(e: &PasswordPolicyError) -> Self {
        Self {
            code: e.code().to_string(),
            message: e.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PasswordPolicyError;

    /// Policy messages are rendered verbatim to end users on the login /
    /// update-password screens. "Entropy" and "bits" are cryptographic
    /// vocabulary that means nothing to them, and the numbers are an
    /// implementation detail of our estimator — see issue #1302.
    #[test]
    fn insufficient_entropy_message_avoids_cryptographic_jargon() {
        let message = PasswordPolicyError::InsufficientEntropy {
            min_bits: 80.0,
            actual_bits: 47.6,
        }
        .to_string();

        let lowered = message.to_lowercase();
        assert!(
            !lowered.contains("entropy"),
            "message leaks the word entropy: {message}"
        );
        assert!(
            !lowered.contains("bits"),
            "message leaks a bit count: {message}"
        );
        assert!(
            lowered.contains("strong"),
            "message should tell the user what is wrong: {message}"
        );
    }

    /// The `code` is the machine-readable half of the contract: clients may
    /// branch on it, so rewording the human message must not move it.
    #[test]
    fn insufficient_entropy_code_is_stable() {
        assert_eq!(
            PasswordPolicyError::InsufficientEntropy {
                min_bits: 80.0,
                actual_bits: 47.6,
            }
            .code(),
            "insufficient_entropy"
        );
    }
}
