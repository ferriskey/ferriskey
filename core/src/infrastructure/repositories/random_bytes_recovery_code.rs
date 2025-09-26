use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::crypto::entities::HashResult;
use crate::domain::crypto::ports::HasherRepository;
use crate::domain::trident::entities::MfaRecoveryCode;
use crate::domain::trident::ports::RecoveryCodeRepository;

use rand::prelude::*;

/// MFA code of L bytes generated randomly.
/// You generally don't want to use this directly but rather variants of RecoveryCodeRepoAny
/// as different byte length/formatter combos aren't always user friendly for display
#[derive(Clone)]
pub struct RandBytesRecoveryCodeRepository<const L: usize, H: HasherRepository> {
    hasher: H,
}

impl<const L: usize, H: HasherRepository> RandBytesRecoveryCodeRepository<L, H> {
    pub fn new(hasher: H) -> Self {
        RandBytesRecoveryCodeRepository { hasher }
    }
}

impl<const L: usize, H: HasherRepository> RecoveryCodeRepository
    for RandBytesRecoveryCodeRepository<L, H>
{
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
