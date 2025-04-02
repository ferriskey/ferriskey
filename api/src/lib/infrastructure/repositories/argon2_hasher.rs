use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::domain::{
    credential::entities::model::CredentialData,
    crypto::ports::{HashResult, HasherRepository},
};

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
    async fn hash_password(&self, password: &str) -> Result<HashResult, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let params = argon2::Params::new(
            32 * 1024, // Memory cost (32MB)
            3,         // Number of iterations
            1,         // Parallelism degree
            Some(32),  // Output length
        )
        .map_err(|e| anyhow::anyhow!("Error during Argon2 configuration: {}", e))?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2d, Version::V0x13, params.clone());

        let credential_data =
            CredentialData::new(params.t_cost(), argon2::Algorithm::Argon2d.to_string());

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Error during password hashing: {}", e))?
            .to_string();

        let hash_result = HashResult::new(password_hash, salt.to_string(), credential_data);
        Ok(hash_result)
    }

    async fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        credential_data: &CredentialData,
        _salt: &str,
    ) -> Result<bool, anyhow::Error> {
        let algorithm = match credential_data.algorithm.as_str() {
            "argon2i" => Algorithm::Argon2i,
            "argon2d" => Algorithm::Argon2d,
            _ => Algorithm::Argon2id, // Par défaut, utiliser Argon2id
        };

        let argon2 = Argon2::new(
            algorithm,
            Version::V0x13,
            Params::new(65536, credential_data.hash_iterations, 4, None)
                .map_err(|e| anyhow::anyhow!("Erreur de configuration des paramètres: {}", e))?,
        );

        let parsed_hash = PasswordHash::new(secret_data)
            .map_err(|e| anyhow::anyhow!("Error parsing hash: {}", e))?;

        let result = argon2.verify_password(password.as_bytes(), &parsed_hash);

        Ok(result.is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_hash_password_success() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let result = hasher.hash_password(password).await;

        assert!(result.is_ok(), "Password hashing should succeed");

        let hash_result = result.unwrap();
        assert!(!hash_result.hash.is_empty(), "Hash should not be empty");
        assert!(!hash_result.salt.is_empty(), "Salt should not be empty");
        assert!(
            !hash_result.credential_data.algorithm.is_empty(),
            "Credential data should not be empty"
        );

        assert!(
            hash_result.hash.starts_with("$argon2"),
            "Hash should start with '$argon2'"
        );
    }

    #[test]
    async fn test_verify_password_success() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let hash_result = hasher.hash_password(password).await.unwrap();

        let result = hasher
            .verify_password(
                password,
                &hash_result.hash,
                &hash_result.credential_data,
                &hash_result.salt,
            )
            .await;

        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "Password should be verified");
    }

    #[test]
    async fn test_verify_password_wrong_password() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";
        let wrong_password = "bad_password";

        let hash_result = hasher.hash_password(password).await.unwrap();

        let result = hasher
            .verify_password(
                wrong_password,
                &hash_result.hash,
                &hash_result.credential_data,
                &hash_result.salt,
            )
            .await;

        assert!(result.is_ok(), "Verification should not fail with an error");
        assert!(!result.unwrap(), "Bad password should not be verified");
    }

    #[test]
    async fn test_verify_password_invalid_hash() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";
        let invalid_hash = "invalid_hash";

        let result = hasher
            .verify_password(
                password,
                invalid_hash,
                &CredentialData::new(1, "argon2d".to_string()),
                invalid_hash,
            )
            .await;

        assert!(
            result.is_err(),
            "Verification should fail with an invalid hash"
        );
    }

    #[test]
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

    #[test]
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
