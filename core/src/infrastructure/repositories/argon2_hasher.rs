use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use tokio::task;
use tracing::instrument;

use ferriskey_security::SecurityError;

use crate::domain::crypto::{HashResult, HasherRepository};
use ferriskey_domain::crypto::PasswordHashingCost;

const MAX_IMPORTED_M_COST: u32 = 262_144;
const MAX_IMPORTED_T_COST: u32 = 16;
const MAX_IMPORTED_P_COST: u32 = 8;

#[derive(Debug, Clone)]
pub struct Argon2HasherRepository {
    cost: PasswordHashingCost,
}

impl Argon2HasherRepository {
    pub fn new(cost: PasswordHashingCost) -> Self {
        Self { cost }
    }

    fn params(&self) -> Result<Params, SecurityError> {
        Params::new(
            self.cost.memory_kib,
            self.cost.iterations,
            self.cost.parallelism,
            Some(32),
        )
        .map_err(|e| SecurityError::HashingError(e.to_string()))
    }

    fn stored_cost_matches(&self, secret_data: &str) -> bool {
        PasswordHash::new(secret_data)
            .ok()
            .and_then(|parsed| Params::try_from(&parsed).ok())
            .is_some_and(|params| {
                params.m_cost() == self.cost.memory_kib
                    && params.t_cost() == self.cost.iterations
                    && params.p_cost() == self.cost.parallelism
            })
    }
}

impl Default for Argon2HasherRepository {
    fn default() -> Self {
        Self::new(PasswordHashingCost::default())
    }
}

pub fn verify_argon2_password(
    password: &str,
    secret_data: &str,
    algorithm: &str,
) -> Result<bool, SecurityError> {
    let algorithm = match algorithm {
        "argon2i" => Algorithm::Argon2i,
        "argon2d" => Algorithm::Argon2d,
        _ => Algorithm::Argon2id,
    };
    let argon2 = Argon2::new(algorithm, Version::V0x13, Params::default());
    let parsed_hash =
        PasswordHash::new(secret_data).map_err(|e| SecurityError::HashingError(e.to_string()))?;

    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

impl HasherRepository for Argon2HasherRepository {
    async fn hash_password(&self, password: &str) -> Result<HashResult, SecurityError> {
        let salt = SaltString::generate(&mut OsRng);
        let params = self.params()?;
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
        let password = password.to_string();
        let secret_data = secret_data.to_string();
        let algorithm = algorithm.to_string();
        let is_valid = task::spawn_blocking(move || {
            verify_argon2_password(&password, &secret_data, &algorithm)
        })
        .await
        .map_err(|e| {
            SecurityError::HashingError(format!("argon2 verify task join error: {e}"))
        })??;

        Ok(is_valid)
    }

    fn needs_rehash(&self, algorithm: &str, secret_data: &str) -> bool {
        algorithm != Algorithm::Argon2id.as_str() || !self.stored_cost_matches(secret_data)
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

        if params.m_cost() > MAX_IMPORTED_M_COST
            || params.t_cost() > MAX_IMPORTED_T_COST
            || params.p_cost() > MAX_IMPORTED_P_COST
        {
            return Err(SecurityError::UnsupportedHash(format!(
                "argon2 parameters m={},t={},p={} exceed m={MAX_IMPORTED_M_COST},t={MAX_IMPORTED_T_COST},p={MAX_IMPORTED_P_COST}",
                params.m_cost(),
                params.t_cost(),
                params.p_cost()
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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
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

    #[tokio::test]
    async fn a_hash_made_with_the_configured_cost_needs_no_rehash() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let hash = hasher.hash_password("secret").await.expect("hash");

        assert!(!hasher.needs_rehash("argon2id", &hash.hash));
        assert!(hasher.needs_rehash("argon2i", &hash.hash));
        assert!(hasher.needs_rehash("bcrypt", "$2b$10$abcdefghijklmnopqrstuu"));
    }

    #[tokio::test]
    async fn a_hash_made_with_another_cost_is_rehashed_on_login() {
        let yesterday = Argon2HasherRepository::new(PasswordHashingCost {
            memory_kib: 7_168,
            iterations: 5,
            parallelism: 1,
        });
        let today = Argon2HasherRepository::new(PasswordHashingCost {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        });
        let old_hash = yesterday.hash_password("secret").await.expect("hash");

        assert!(today.needs_rehash("argon2id", &old_hash.hash));
        assert!(
            today
                .verify_password("secret", &old_hash.hash, 5, "argon2id", "")
                .await
                .expect("verify")
        );
    }

    #[tokio::test]
    async fn a_new_hash_carries_the_configured_cost() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost {
            memory_kib: 19_456,
            iterations: 2,
            parallelism: 1,
        });
        let hash = hasher.hash_password("secret").await.expect("hash");
        let parsed = PasswordHash::new(&hash.hash).expect("phc string");
        let params = Params::try_from(&parsed).expect("params");

        assert_eq!(
            (params.m_cost(), params.t_cost(), params.p_cost()),
            (19_456, 2, 1)
        );
        assert_eq!(hash.hash_iterations, 2);
    }

    #[tokio::test]
    async fn validate_hash_accepts_its_own_argon2id_output() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash(&hash.algorithm, &hash.hash, hash.hash_iterations);

        assert!(result.is_ok(), "{result:?}");
    }

    #[tokio::test]
    async fn validate_hash_rejects_mismatched_iterations() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash(&hash.algorithm, &hash.hash, hash.hash_iterations + 1);

        assert!(matches!(result, Err(SecurityError::UnsupportedHash(_))));
    }

    #[tokio::test]
    async fn validate_hash_rejects_an_algorithm_label_that_contradicts_the_hash() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let hash = hasher.hash_password("my_password").await.unwrap();

        let result = hasher.validate_hash("argon2i", &hash.hash, hash.hash_iterations);

        assert!(matches!(result, Err(SecurityError::UnsupportedHash(_))));
    }

    #[tokio::test]
    async fn validate_hash_rejects_parameters_above_the_operational_ceilings() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let hash = hasher.hash_password("my_password").await.unwrap();
        let own = "m=7168,t=5,p=1";
        assert!(hash.hash.contains(own), "{}", hash.hash);

        for (params, iterations) in [
            ("m=262145,t=5,p=1", 5),
            ("m=7168,t=17,p=1", 17),
            ("m=7168,t=5,p=9", 5),
        ] {
            let inflated = hash.hash.replacen(own, params, 1);
            assert!(
                matches!(
                    hasher.validate_hash("argon2id", &inflated, iterations),
                    Err(SecurityError::UnsupportedHash(_))
                ),
                "{params} should be rejected"
            );
        }

        let ceiling = hash.hash.replacen(own, "m=262144,t=16,p=8", 1);
        assert!(hasher.validate_hash("argon2id", &ceiling, 16).is_ok());
    }

    #[test]
    fn validate_hash_rejects_non_phc_strings_and_unknown_algorithms() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());

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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
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
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let password = "my_password";

        let hash_result1 = hasher.hash_password(password).await.unwrap();
        let hash_result2 = hasher.hash_password(password).await.unwrap();

        assert_ne!(
            hash_result1.hash, hash_result2.hash,
            "Same password should have different hashes due to the random salt"
        );
    }

    #[tokio::test]
    async fn the_blocking_verification_agrees_with_the_asynchronous_one() {
        let hasher = Argon2HasherRepository::new(PasswordHashingCost::default());
        let stored = hasher.hash_password("secret").await.expect("hash");

        assert!(verify_argon2_password("secret", &stored.hash, &stored.algorithm).expect("verify"));
        assert!(!verify_argon2_password("other", &stored.hash, &stored.algorithm).expect("verify"));
        assert!(verify_argon2_password("secret", "not a phc string", "argon2id").is_err());
    }
}
