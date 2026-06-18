use std::fmt::Display;

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
            PasswordPolicyError::InsufficientEntropy {
                min_bits,
                actual_bits,
            } => {
                write!(
                    f,
                    "Password entropy is too low: {:.1} bits (minimum {:.1} bits required)",
                    actual_bits, min_bits
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
