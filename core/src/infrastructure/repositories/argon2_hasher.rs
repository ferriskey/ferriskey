use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::task;
use tracing::instrument;

use ferriskey_security::SecurityError;

use crate::domain::crypto::{HashResult, HasherRepository};

#[derive(Debug, Clone)]
pub struct Argon2HasherRepository {}

impl Argon2HasherRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Argon2HasherRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl HasherRepository for Argon2HasherRepository {
    async fn hash_password(&self, password: &str) -> Result<HashResult, SecurityError> {
        let salt = SaltString::generate(&mut OsRng);
        let params = argon2::Params::new(
            7168,     // Memory cost (KiB) aligned with Keycloak profile
            5,        // Number of iterations aligned with Keycloak profile
            1,        // Parallelism degree
            Some(32), // Output length
        )
        .map_err(|e| SecurityError::HashingError(e.to_string()))?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params.clone());

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| SecurityError::HashingError(e.to_string()))?
            .to_string();

        let hash_result = HashResult::new(
            password_hash,
            salt.to_string(),
            params.t_cost(),
            argon2::Algorithm::Argon2id.to_string(),
        );
        Ok(hash_result)
    }

    #[instrument(
        skip(self, password, secret_data, _salt),
        fields(
            algorithm = %algorithm,
            hash_len = secret_data.len()
        )
    )]
    async fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        _hash_iterations: u32,
        algorithm: &str,
        _salt: &str,
    ) -> Result<bool, SecurityError> {
        let algorithm = match algorithm {
            "argon2i" => Algorithm::Argon2i,
            "argon2d" => Algorithm::Argon2d,
            _ => Algorithm::Argon2id, // Par défaut, utiliser Argon2id
        };

        let password = password.to_string();
        let secret_data = secret_data.to_string();
        let is_valid = task::spawn_blocking(move || {
            let argon2 = Argon2::new(algorithm, Version::V0x13, Params::default());
            let parsed_hash = PasswordHash::new(&secret_data)
                .map_err(|e| SecurityError::HashingError(e.to_string()))?;
            let is_valid = argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok();
            Ok::<bool, SecurityError>(is_valid)
        })
        .await
        .map_err(|e| {
            SecurityError::HashingError(format!("argon2 verify task join error: {e}"))
        })??;

        Ok(is_valid)
    }

    fn needs_rehash(&self, algorithm: &str) -> bool {
        algorithm != Algorithm::Argon2id.as_str()
    }

    fn validate_hash(
        &self,
        algorithm: &str,
        secret_data: &str,
        hash_iterations: u32,
    ) -> Result<(), SecurityError> {
        let expected = match algorithm {
            "argon2id" => Algorithm::Argon2id,
            "argon2i" => Algorithm::Argon2i,
            "argon2d" => Algorithm::Argon2d,
            other => {
                return Err(SecurityError::UnsupportedHash(format!(
                    "unsupported algorithm `{other}`"
                )));
            }
        };

        let parsed = PasswordHash::new(secret_data).map_err(|_| {
            SecurityError::UnsupportedHash("secret_data is not a PHC argon2 string".to_string())
        })?;

        if parsed.algorithm != expected.ident() {
            return Err(SecurityError::UnsupportedHash(format!(
                "secret_data is not an {algorithm} hash"
            )));
        }

        let params = Params::try_from(&parsed).map_err(|_| {
            SecurityError::UnsupportedHash("secret_data has invalid argon2 parameters".to_string())
        })?;

        if params.t_cost() != hash_iterations {
            return Err(SecurityError::UnsupportedHash(format!(
                "hash_iterations {hash_iterations} does not match the hash cost {}",
                params.t_cost()
            )));
        }

        Ok(())
    }

    async fn hash_magic_token(&self, token: &str) -> Result<HashResult, SecurityError> {
        self.hash_password(token).await
    }

    async fn verify_magic_token(
        &self,
        token: &str,
        secret_data: &str,
    ) -> Result<bool, SecurityError> {
        let result = self
            .verify_password(token, secret_data, 3, "argon2d", "")
            .await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_password_success() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let result = hasher.hash_password(password).await;
        assert!(result.is_ok(), "Password hashing should succeed");

        let hash_result = result.unwrap();
        assert!(!hash_result.hash.is_empty(), "Hash should not be empty");
        assert!(!hash_result.salt.is_empty(), "Salt should not be empty");
        assert!(
            !hash_result.algorithm.is_empty(),
            "Algorithm should not be empty"
        );
        assert!(
            hash_result.hash_iterations > 0,
            "Hash iterations should be set"
        );

        assert!(
            hash_result.hash.starts_with("$argon2"),
            "Hash should start with '$argon2'"
        );
    }

    #[tokio::test]
    async fn test_verify_password_success() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let hash_result = hasher.hash_password(password).await.unwrap();

        let result = hasher
            .verify_password(
                password,
                &hash_result.hash,
                hash_result.hash_iterations,
                &hash_result.algorithm,
                &hash_result.salt,
            )
            .await;

        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "Password should be verified");
    }

    #[tokio::test]
    async fn test_verify_password_wrong_password() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";
        let wrong_password = "bad_password";

        let hash_result = hasher.hash_password(password).await.unwrap();
        let result = hasher
            .verify_password(
                wrong_password,
                &hash_result.hash,
                hash_result.hash_iterations,
                &hash_result.algorithm,
                &hash_result.salt,
            )
            .await;

        assert!(result.is_ok(), "Verification should not fail with an error");
        assert!(!result.unwrap(), "Bad password should not be verified");
    }

    #[tokio::test]
    async fn test_verify_password_invalid_hash() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";
        let invalid_hash = "invalid_hash";
        let hash_iterations = 1;
        let algorithm = "argon2d";

        let result = hasher
            .verify_password(
                password,
                invalid_hash,
                hash_iterations,
                algorithm,
                invalid_hash,
            )
            .await;

        assert!(
            result.is_err(),
            "Verification should fail with an invalid hash"
        );
    }

    #[test]
    fn test_needs_rehash_only_for_non_argon2id() {
        let hasher = Argon2HasherRepository::new();

        assert!(!hasher.needs_rehash("argon2id"));
        assert!(hasher.needs_rehash("argon2i"));
        assert!(hasher.needs_rehash("bcrypt"));
    }

    #[tokio::test]
    async fn validate_hash_accepts_its_own_argon2id_output() {
        let hasher = Argon2HasherRepository::new();
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash(&hash.algorithm, &hash.hash, hash.hash_iterations);

        assert!(result.is_ok(), "{result:?}");
    }

    #[tokio::test]
    async fn validate_hash_rejects_mismatched_iterations() {
        let hasher = Argon2HasherRepository::new();
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash(&hash.algorithm, &hash.hash, hash.hash_iterations + 1);

        assert!(matches!(result, Err(SecurityError::UnsupportedHash(_))));
    }

    #[tokio::test]
    async fn validate_hash_rejects_an_algorithm_label_that_contradicts_the_hash() {
        let hasher = Argon2HasherRepository::new();
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash("argon2i", &hash.hash, hash.hash_iterations);

        assert!(matches!(result, Err(SecurityError::UnsupportedHash(_))));
    }

    #[test]
    fn validate_hash_rejects_non_phc_strings_and_unknown_algorithms() {
        let hasher = Argon2HasherRepository::new();

        assert!(matches!(
            hasher.validate_hash("argon2id", "not-a-hash", 5),
            Err(SecurityError::UnsupportedHash(_))
        ));
        assert!(matches!(
            hasher.validate_hash("md5", "5f4dcc3b5aa765d61d8327deb882cf99", 1),
            Err(SecurityError::UnsupportedHash(_))
        ));
    }

    #[tokio::test]
    async fn test_different_passwords_different_hashes() {
        let hasher = Argon2HasherRepository::new();
        let password1 = "first_password";
        let password2 = "second_password";

        let hash_result1 = hasher.hash_password(password1).await.unwrap();
        let hash_result2 = hasher.hash_password(password2).await.unwrap();

        assert_ne!(
            hash_result1.hash, hash_result2.hash,
            "Two different passwords should have different hashes"
        );
    }

    #[tokio::test]
    async fn test_same_password_different_hashes() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let hash_result1 = hasher.hash_password(password).await.unwrap();
        let hash_result2 = hasher.hash_password(password).await.unwrap();

        assert_ne!(
            hash_result1.hash, hash_result2.hash,
            "Same password should have different hashes due to the random salt"
        );
    }
}
