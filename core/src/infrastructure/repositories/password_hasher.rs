use std::str::FromStr;

use bcrypt::{BcryptError, HashParts};
use tokio::task;
use tracing::instrument;

use ferriskey_security::SecurityError;

use crate::domain::crypto::{HashResult, HasherRepository};
use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;

const BCRYPT_ALGORITHM: &str = "bcrypt";
const BCRYPT_PREFIXES: [&str; 3] = ["$2a$", "$2b$", "$2y$"];
const BCRYPT_COSTS: std::ops::RangeInclusive<u32> = 4..=14;

#[derive(Debug, Clone, Default)]
pub struct PasswordHasherRepository {
    argon2: Argon2HasherRepository,
}

impl PasswordHasherRepository {
    pub fn new() -> Self {
        Self {
            argon2: Argon2HasherRepository::new(),
        }
    }

    async fn verify_bcrypt(password: &str, secret_data: &str) -> Result<bool, SecurityError> {
        let password = password.to_string();
        let secret_data = secret_data.to_string();

        task::spawn_blocking(move || {
            match bcrypt::non_truncating_verify(password.as_bytes(), &secret_data) {
                Ok(is_valid) => Ok(is_valid),
                Err(BcryptError::Truncation(_)) => Ok(false),
                Err(e) => Err(SecurityError::HashingError(e.to_string())),
            }
        })
        .await
        .map_err(|e| SecurityError::HashingError(format!("bcrypt verify task join error: {e}")))?
    }

    fn validate_bcrypt(secret_data: &str, hash_iterations: u32) -> Result<(), SecurityError> {
        if !BCRYPT_PREFIXES
            .iter()
            .any(|prefix| secret_data.starts_with(prefix))
        {
            return Err(SecurityError::UnsupportedHash(
                "secret_data must start with $2a$, $2b$ or $2y$".to_string(),
            ));
        }

        let parts = HashParts::from_str(secret_data)
            .map_err(|e| SecurityError::UnsupportedHash(e.to_string()))?;

        if !BCRYPT_COSTS.contains(&parts.get_cost()) {
            return Err(SecurityError::UnsupportedHash(format!(
                "bcrypt cost {} is outside {}..={}",
                parts.get_cost(),
                BCRYPT_COSTS.start(),
                BCRYPT_COSTS.end()
            )));
        }

        if parts.get_cost() != hash_iterations {
            return Err(SecurityError::UnsupportedHash(format!(
                "hash_iterations {hash_iterations} does not match the hash cost {}",
                parts.get_cost()
            )));
        }

        Ok(())
    }
}

impl HasherRepository for PasswordHasherRepository {
    async fn hash_password(&self, password: &str) -> Result<HashResult, SecurityError> {
        self.argon2.hash_password(password).await
    }

    #[instrument(
        skip(self, password, secret_data, salt),
        fields(
            algorithm = %algorithm,
            hash_len = secret_data.len()
        )
    )]
    async fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> Result<bool, SecurityError> {
        if algorithm == BCRYPT_ALGORITHM {
            return Self::verify_bcrypt(password, secret_data).await;
        }

        self.argon2
            .verify_password(password, secret_data, hash_iterations, algorithm, salt)
            .await
    }

    fn needs_rehash(&self, algorithm: &str) -> bool {
        self.argon2.needs_rehash(algorithm)
    }

    fn validate_hash(
        &self,
        algorithm: &str,
        secret_data: &str,
        hash_iterations: u32,
    ) -> Result<(), SecurityError> {
        if algorithm == BCRYPT_ALGORITHM {
            return Self::validate_bcrypt(secret_data, hash_iterations);
        }

        self.argon2
            .validate_hash(algorithm, secret_data, hash_iterations)
    }

    async fn hash_magic_token(&self, token: &str) -> Result<HashResult, SecurityError> {
        self.argon2.hash_magic_token(token).await
    }

    async fn verify_magic_token(
        &self,
        token: &str,
        secret_data: &str,
    ) -> Result<bool, SecurityError> {
        self.argon2.verify_magic_token(token, secret_data).await
    }
}

#[cfg(test)]
mod tests {
    use bcrypt::Version;

    use super::*;

    const PASSWORD: &str = "correct horse battery staple";

    fn bcrypt_hash(password: &str, version: Version) -> String {
        bcrypt::hash_with_result(password, 4)
            .expect("bcrypt hash")
            .format_for_version(version)
    }

    async fn verify_bcrypt(hasher: &PasswordHasherRepository, password: &str, hash: &str) -> bool {
        hasher
            .verify_password(password, hash, 4, BCRYPT_ALGORITHM, "")
            .await
            .expect("bcrypt verification should not error")
    }

    #[tokio::test]
    async fn verifies_every_bcrypt_version_prefix() {
        let hasher = PasswordHasherRepository::new();

        for version in [Version::TwoA, Version::TwoB, Version::TwoY] {
            let hash = bcrypt_hash(PASSWORD, version);
            assert!(verify_bcrypt(&hasher, PASSWORD, &hash).await, "{hash}");
        }
    }

    #[tokio::test]
    async fn rejects_wrong_password_against_bcrypt_hash() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoA);

        assert!(!verify_bcrypt(&hasher, "wrong password", &hash).await);
    }

    #[tokio::test]
    async fn rejects_password_longer_than_bcrypt_limit_instead_of_truncating() {
        let hasher = PasswordHasherRepository::new();
        let prefix = "a".repeat(72);
        let hash = bcrypt_hash(&prefix, Version::TwoB);
        let longer = format!("{prefix}suffix");

        assert!(!verify_bcrypt(&hasher, &longer, &hash).await);
    }

    #[tokio::test]
    async fn errors_on_malformed_bcrypt_hash() {
        let hasher = PasswordHasherRepository::new();

        let result = hasher
            .verify_password(PASSWORD, "not-a-bcrypt-hash", 10, BCRYPT_ALGORITHM, "")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn verifies_argon2id_hash_through_argon2() {
        let hasher = PasswordHasherRepository::new();
        let hash = hasher.hash_password(PASSWORD).await.expect("argon2 hash");

        let is_valid = hasher
            .verify_password(
                PASSWORD,
                &hash.hash,
                hash.hash_iterations,
                &hash.algorithm,
                &hash.salt,
            )
            .await
            .expect("argon2 verification should not error");

        assert!(is_valid);
    }

    #[tokio::test]
    async fn hashes_new_passwords_with_argon2id() {
        let hasher = PasswordHasherRepository::new();

        let hash = hasher.hash_password(PASSWORD).await.expect("argon2 hash");

        assert_eq!(hash.algorithm, "argon2id");
        assert!(hash.hash.starts_with("$argon2id$"));
        assert!(!hasher.needs_rehash(&hash.algorithm));
    }

    #[tokio::test]
    async fn bcrypt_hash_under_argon2_label_is_rejected() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoB);

        let result = hasher
            .verify_password(PASSWORD, &hash, 10, "argon2id", "")
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn flags_bcrypt_credentials_for_rehash() {
        let hasher = PasswordHasherRepository::new();

        assert!(hasher.needs_rehash(BCRYPT_ALGORITHM));
    }

    #[test]
    fn validates_every_supported_bcrypt_prefix() {
        let hasher = PasswordHasherRepository::new();

        for version in [Version::TwoA, Version::TwoB, Version::TwoY] {
            let hash = bcrypt_hash(PASSWORD, version);
            let result = hasher.validate_hash(BCRYPT_ALGORITHM, &hash, 4);
            assert!(result.is_ok(), "{hash}: {result:?}");
        }
    }

    #[test]
    fn refuses_the_2x_bcrypt_variant() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoX);

        assert!(matches!(
            hasher.validate_hash(BCRYPT_ALGORITHM, &hash, 4),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[test]
    fn refuses_a_bcrypt_cost_that_disagrees_with_hash_iterations() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoB);

        assert!(matches!(
            hasher.validate_hash(BCRYPT_ALGORITHM, &hash, 10),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[test]
    fn refuses_a_truncated_bcrypt_hash() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoB);

        assert!(matches!(
            hasher.validate_hash(BCRYPT_ALGORITHM, &hash[..59], 4),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[tokio::test]
    async fn delegates_argon2_validation() {
        let hasher = PasswordHasherRepository::new();
        let hash = hasher.hash_password(PASSWORD).await.expect("argon2 hash");

        assert!(
            hasher
                .validate_hash(&hash.algorithm, &hash.hash, hash.hash_iterations)
                .is_ok()
        );
        assert!(matches!(
            hasher.validate_hash("pbkdf2-sha256", &hash.hash, hash.hash_iterations),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[test]
    fn refuses_a_bcrypt_cost_above_the_operational_ceiling() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoB).replacen("$04$", "$15$", 1);

        assert!(matches!(
            hasher.validate_hash(BCRYPT_ALGORITHM, &hash, 15),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[test]
    fn accepts_the_highest_allowed_bcrypt_cost() {
        let hasher = PasswordHasherRepository::new();
        let hash = bcrypt_hash(PASSWORD, Version::TwoB).replacen("$04$", "$14$", 1);

        assert!(hasher.validate_hash(BCRYPT_ALGORITHM, &hash, 14).is_ok());
    }
}
