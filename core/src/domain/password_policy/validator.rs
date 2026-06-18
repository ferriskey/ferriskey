use super::{entity::PasswordPolicy, error::PasswordPolicyError};

/// Top-100 most common passwords (lowercase). Any password in this list is rejected when
/// `forbid_common` is enabled, regardless of case.
const COMMON_PASSWORDS: &[&str] = &[
    "123456",
    "password",
    "123456789",
    "12345678",
    "12345",
    "1234567",
    "1234567890",
    "qwerty",
    "abc123",
    "111111",
    "123123",
    "admin",
    "letmein",
    "welcome",
    "monkey",
    "login",
    "princess",
    "solo",
    "master",
    "dragon",
    "passw0rd",
    "pass@word1",
    "iloveyou",
    "sunshine",
    "shadow",
    "superman",
    "michael",
    "jessica",
    "thomas",
    "charlie",
    "robert",
    "football",
    "baseball",
    "soccer",
    "hockey",
    "basketball",
    "batman",
    "superman",
    "trustno1",
    "qwerty123",
    "password1",
    "password123",
    "hunter2",
    "hunter",
    "696969",
    "mustang",
    "access",
    "hello",
    "secret",
    "whatever",
    "qazwsx",
    "zxcvbnm",
    "1q2w3e4r",
    "1qaz2wsx",
    "q1w2e3r4",
    "asdfgh",
    "asdfghjkl",
    "zxcvbn",
    "1234",
    "12341234",
    "1111",
    "0000",
    "7777777",
    "666666",
    "555555",
    "444444",
    "333333",
    "222222",
    "121212",
    "654321",
    "987654321",
    "pass",
    "test",
    "root",
    "toor",
    "temp",
    "changeme",
    "qwert",
    "asdf",
    "zxcv",
    "1password",
    "2password",
    "abc",
    "abcd",
    "abcdef",
    "abcdefg",
    "abcdefgh",
    "abcdefghi",
    "1234abcd",
    "a1b2c3",
    "aaa",
    "aaaaaa",
    "aaaa",
    "bbb",
    "111",
    "000",
    "qqqqqq",
    "pppppp",
    "iloveyou1",
    "lovely",
    "monkey123",
];

/// Pool sizes for character classes used in entropy estimation.
const POOL_LOWERCASE: f64 = 26.0;
const POOL_UPPERCASE: f64 = 26.0;
const POOL_DIGIT: f64 = 10.0;
/// Printable ASCII specials: !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~ plus space = 33 characters.
const POOL_SPECIAL: f64 = 33.0;

/// Estimate password entropy using the charset-pool model:
/// entropy = length × log₂(pool_size)
/// where pool_size = sum of sizes of each character class present in the password.
pub fn estimate_entropy_bits(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password
        .chars()
        .any(|c| c.is_ascii() && !c.is_alphanumeric());

    let pool: f64 = (if has_lower { POOL_LOWERCASE } else { 0.0 })
        + (if has_upper { POOL_UPPERCASE } else { 0.0 })
        + (if has_digit { POOL_DIGIT } else { 0.0 })
        + (if has_special { POOL_SPECIAL } else { 0.0 });

    if pool == 0.0 {
        return 0.0;
    }
    password.len() as f64 * pool.log2()
}

/// Validate a password against a policy.
///
/// `username` and `email_local` are used when `forbid_common` is enabled:
/// a password equal (case-insensitively) to the username or the local part of the email
/// is rejected.
///
/// Returns `Ok(())` if the password satisfies all enabled rules, or
/// `Err(violations)` listing every rule that was violated.
pub fn validate(
    password: &str,
    policy: &PasswordPolicy,
    username: Option<&str>,
    email_local: Option<&str>,
) -> Result<(), Vec<PasswordPolicyError>> {
    let mut errors = Vec::new();

    if password.len() < policy.min_length as usize {
        errors.push(PasswordPolicyError::TooShort {
            min: policy.min_length,
            actual: password.len(),
        });
    }

    if policy.require_uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push(PasswordPolicyError::MissingUppercase);
    }

    if policy.require_lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push(PasswordPolicyError::MissingLowercase);
    }

    if policy.require_number && !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push(PasswordPolicyError::MissingNumber);
    }

    if policy.require_special
        && !password
            .chars()
            .any(|c| c.is_ascii() && !c.is_alphanumeric())
    {
        errors.push(PasswordPolicyError::MissingSpecialCharacter);
    }

    let entropy = estimate_entropy_bits(password);
    let min_entropy = policy.min_entropy_bits as f64;
    if entropy < min_entropy {
        errors.push(PasswordPolicyError::InsufficientEntropy {
            min_bits: min_entropy,
            actual_bits: entropy,
        });
    }

    if policy.forbid_common {
        let lower = password.to_ascii_lowercase();
        let is_common = COMMON_PASSWORDS.contains(&lower.as_str());
        let matches_username = username
            .map(|u| lower == u.to_ascii_lowercase())
            .unwrap_or(false);
        let matches_email_local = email_local
            .map(|e| lower == e.to_ascii_lowercase())
            .unwrap_or(false);

        if is_common || matches_username || matches_email_local {
            errors.push(PasswordPolicyError::CommonPassword);
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::password_policy::entity::PasswordPolicy;
    use chrono::Utc;
    use uuid::Uuid;

    fn cnil_policy() -> PasswordPolicy {
        PasswordPolicy::default(Uuid::new_v4())
    }

    fn permissive_policy() -> PasswordPolicy {
        PasswordPolicy {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4(),
            min_length: 1,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
            max_age_days: None,
            min_entropy_bits: 0,
            forbid_common: false,
            check_breached: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn strong_password_passes_cnil_policy() {
        // 14 chars: lower + upper + digit + special → pool=95, entropy ≈ 92 bits
        let result = validate("Tr0ub4dor&3XY!", &cnil_policy(), None, None);
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[test]
    fn password_too_short_fails() {
        let result = validate("Tr0ub!", &cnil_policy(), None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::TooShort { min: 12, actual: 6 }))
        );
    }

    #[test]
    fn missing_uppercase_fails() {
        let mut p = cnil_policy();
        p.min_entropy_bits = 0;
        let result = validate("tr0ub4dor&3xy!", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingUppercase)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn missing_lowercase_fails() {
        let mut p = cnil_policy();
        p.min_entropy_bits = 0;
        let result = validate("TR0UB4DOR&3XY!", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingLowercase)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn missing_digit_fails() {
        let mut p = cnil_policy();
        p.min_entropy_bits = 0;
        let result = validate("TrOubAdOr&XYzA!", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingNumber)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn missing_special_fails() {
        let mut p = cnil_policy();
        p.min_entropy_bits = 0;
        let result = validate("Tr0ub4dor3XYzA", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::MissingSpecialCharacter)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn entropy_boundary_79_bits_fails() {
        // pool=62 (lower+upper+digit), entropy = len * log2(62) ≈ len * 5.954
        // to reach exactly ≥80 bits we need len ≥ ceil(80/5.954) = 14 chars
        // 13 chars with only alphanum → ~77.4 bits < 80
        let mut p = permissive_policy();
        p.min_length = 13;
        p.min_entropy_bits = 80;
        // 13-char alphanumeric: pool=62
        let result = validate("Abcdefg1234567", &p, None, None);
        // 14 chars → pool=62, entropy≈83.4 → should pass
        // but let's test 13 chars → should fail
        let result13 = validate("Abcdefg123456", &p, None, None);
        let errors13 = result13.unwrap_err();
        assert!(
            errors13
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::InsufficientEntropy { .. })),
            "expected InsufficientEntropy, got {:?}",
            errors13
        );
        // 14 chars → passes entropy
        assert!(
            result.is_ok(),
            "expected 14-char password to pass, got {:?}",
            result
        );
    }

    #[test]
    fn entropy_boundary_80_bits_passes() {
        // 14 lower+upper+digit chars → pool=62, entropy ≈ 83.4 bits ≥ 80
        let mut p = permissive_policy();
        p.min_entropy_bits = 80;
        let result = validate("Abcdefg12345678", &p, None, None);
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[test]
    fn common_password_rejected() {
        let mut p = permissive_policy();
        p.forbid_common = true;
        let result = validate("password", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::CommonPassword)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn common_password_case_insensitive() {
        let mut p = permissive_policy();
        p.forbid_common = true;
        let result = validate("PASSWORD", &p, None, None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::CommonPassword)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn password_equal_to_username_rejected() {
        let mut p = permissive_policy();
        p.forbid_common = true;
        let result = validate("johndoe", &p, Some("johndoe"), None);
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::CommonPassword)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn password_equal_to_email_local_rejected() {
        let mut p = permissive_policy();
        p.forbid_common = true;
        let result = validate("alice", &p, None, Some("alice"));
        let errors = result.unwrap_err();
        assert!(
            errors
                .iter()
                .any(|e| matches!(e, PasswordPolicyError::CommonPassword)),
            "{:?}",
            errors
        );
    }

    #[test]
    fn unique_password_not_in_common_list_accepted() {
        let mut p = permissive_policy();
        p.forbid_common = true;
        let result = validate("xK9#mR2$pL7@nQ5", &p, Some("alice"), Some("alice"));
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
    }

    #[test]
    fn multiple_violations_reported_together() {
        let result = validate("abc", &cnil_policy(), None, None);
        let errors = result.unwrap_err();
        // TooShort, MissingUppercase, MissingNumber, MissingSpecialCharacter, InsufficientEntropy
        assert!(
            errors.len() >= 3,
            "expected multiple errors, got {:?}",
            errors
        );
    }

    #[test]
    fn estimate_entropy_empty_password() {
        assert_eq!(estimate_entropy_bits(""), 0.0);
    }

    #[test]
    fn estimate_entropy_digits_only() {
        // pool=10, len=6 → 6 * log2(10) ≈ 19.9
        let e = estimate_entropy_bits("123456");
        assert!(e > 19.0 && e < 21.0, "got {}", e);
    }

    #[test]
    fn estimate_entropy_all_classes() {
        // pool=95, len=14 → 14 * log2(95) ≈ 91.9
        let e = estimate_entropy_bits("Abc1!Def2@Gh3#");
        assert!(e > 90.0 && e < 95.0, "got {}", e);
    }
}
