use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordPolicyError {
    TooShort { min: i32, actual: usize },
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialCharacter,
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
        }
    }
}

impl std::error::Error for PasswordPolicyError {}
