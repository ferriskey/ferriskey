use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::crypto::entities::HashResult;
use crate::domain::crypto::ports::HasherRepository;
use crate::domain::trident::entities::MfaRecoveryCode;
use crate::domain::trident::ports::RecoveryCodeRepository;
use crate::infrastructure::repositories::HasherRepoAny;
use rand::prelude::*;

/// MFA code of L bytes generated randomly.
/// You generally don't want to use this directly but rather variants of RecoveryCodeRepoAny
/// as different byte length/formatter combos aren't always user friendly for display
#[derive(Clone)]
pub struct RandBytesRecoveryCodeRepository<const L: usize> {
    hasher: HasherRepoAny,
}

impl<const L: usize> RandBytesRecoveryCodeRepository<L> {
    pub fn new(hasher: HasherRepoAny) -> Self {
        RandBytesRecoveryCodeRepository { hasher }
    }
}

impl<const L: usize> RecoveryCodeRepository for RandBytesRecoveryCodeRepository<L> {
    fn generate_recovery_code(&self) -> MfaRecoveryCode {
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; L];
        rng.try_fill_bytes(&mut bytes)
            .expect("Thread rng failed to fill byte slice");
        MfaRecoveryCode::from_bytes(&bytes)
    }

    async fn secure_for_storage(&self, code: &MfaRecoveryCode) -> Result<HashResult, CoreError> {
        let hex = code
            .0
            .iter()
            .fold(String::with_capacity(code.0.len() * 2), |accu, byte| {
                format!("{accu}{byte:x?}")
            });

        self.hasher
            .hash_password(hex.as_str())
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn verify(
        &self,
        in_code: &MfaRecoveryCode,
        secret_data: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> Result<bool, CoreError> {
        let in_code = in_code
            .0
            .iter()
            .fold(String::with_capacity(in_code.0.len() * 2), |accu, byte| {
                format!("{accu}{byte:x?}")
            });

        self.hasher.verify_password(
            in_code.as_str(),
            secret_data,
            hash_iterations,
            algorithm,
            salt
        )
        .await
        .map_err(|_e| {
            tracing::debug!("An error occured while verifying password. The error message is intentionally left empty as it may contain sensitive data");
            CoreError::VerifyPasswordError(String::from(""))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::hasher::HasherRepoAny;
    use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;

    const HASHER_REPO: HasherRepoAny = HasherRepoAny::Argon2(Argon2HasherRepository {});

    #[test]
    fn test_random_bytes_recovery_code_generate() {
        let repo = RandBytesRecoveryCodeRepository::<10>::new(HASHER_REPO);
        // Test byte length
        let code = repo.generate_recovery_code();
        assert_eq!(
            code.0.len(),
            10,
            "The generated code length doesn't match the generic parameter"
        );

        // Test randomness
        let mut prev_code = repo.generate_recovery_code();
        for _ in 1..=10 {
            let code = repo.generate_recovery_code();
            assert_ne!(
                prev_code, code,
                "Two successive generated codes are identical. While this doesn't mean the test is invalid, it should only fail extremly rarely. Try re-running tests."
            );
            prev_code = code;
        }
    }
}
