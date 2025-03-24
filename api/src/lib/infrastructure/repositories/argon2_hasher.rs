use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::domain::crypto::ports::HasherRepository;

#[derive(Debug, Clone)]
pub struct Argon2HasherRepository;

impl Argon2HasherRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl HasherRepository for Argon2HasherRepository {
    async fn hash_password(&self, password: &str) -> Result<(String, String), anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Erreur lors du hachage du mot de passe: {}", e))?
            .to_string();

        Ok((password_hash, salt.to_string()))
    }

    async fn verify_password(
        &self,
        password: &str,
        _secret_data: &str,
        credential_data: &str,
    ) -> Result<bool, anyhow::Error> {
        let parsed_hash = PasswordHash::new(credential_data)
            .map_err(|e| anyhow::anyhow!("Erreur lors de l'analyse du hash: {}", e))?;

        let argon2 = Argon2::default();
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

        let (secret, credential) = result.unwrap();
        assert!(!secret.is_empty(), "Secret should not be empty");
        assert!(!credential.is_empty(), "Credential should not be empty");

        assert!(
            credential.starts_with("$argon2"),
            "Hash should start with '$argon2'"
        );
    }

    #[test]
    async fn test_verify_password_success() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let (secret, credential) = hasher.hash_password(password).await.unwrap();

        let result = hasher.verify_password(password, &secret, &credential).await;

        assert!(result.is_ok(), "Verification should succeed");
        assert!(result.unwrap(), "Password should be verified");
    }

    #[test]
    async fn test_verify_password_wrong_password() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";
        let wrong_password = "bad_password";

        let (secret, credential) = hasher.hash_password(password).await.unwrap();

        let result = hasher
            .verify_password(wrong_password, &secret, &credential)
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
            .verify_password(password, invalid_hash, invalid_hash)
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

        let (_, credential1) = hasher.hash_password(password1).await.unwrap();
        let (_, credential2) = hasher.hash_password(password2).await.unwrap();

        assert_ne!(
            credential1, credential2,
            "Two different passwords should have different hashes"
        );
    }

    #[test]
    async fn test_same_password_different_hashes() {
        let hasher = Argon2HasherRepository::new();
        let password = "my_password";

        let (_, credential1) = hasher.hash_password(password).await.unwrap();
        let (_, credential2) = hasher.hash_password(password).await.unwrap();

        assert_ne!(
            credential1, credential2,
            "Same password should have different hashes due to the random salt"
        );
    }
}
