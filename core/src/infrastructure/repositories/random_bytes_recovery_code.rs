use std::sync::Arc;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::crypto::{HashResult, HasherRepository};
use crate::domain::trident::entities::MfaRecoveryCode;
use crate::domain::trident::ports::RecoveryCodeRepository;

use rand::prelude::*;
use sha2::{Digest, Sha256};

/// Number of hex characters (from SHA-256 of the code's hex form) stored as the
/// fast lookup key. 16 hex chars = 64 bits of entropy, far more than enough to
/// uniquely address a single candidate among the handful of codes a user owns,
/// while remaining a constant-time comparison that avoids scanning every row.
const LOOKUP_KEY_LEN: usize = 16;

/// MFA code of L bytes generated randomly.
/// You generally don't want to use this directly but rather variants of RecoveryCodeRepoAny
/// as different byte length/formatter combos aren't always user friendly for display
#[derive(Clone, Debug)]
pub struct RandBytesRecoveryCodeRepository<const L: usize, H: HasherRepository> {
    hasher: Arc<H>,
}

impl<const L: usize, H: HasherRepository> RandBytesRecoveryCodeRepository<L, H> {
    pub fn new(hasher: Arc<H>) -> Self {
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

    async fn secure_for_storage(
        &self,
        code: &MfaRecoveryCode,
    ) -> Result<(HashResult, String), CoreError> {
        let hex = code
            .0
            .iter()
            .fold(String::with_capacity(code.0.len() * 2), |accu, byte| {
                format!("{accu}{byte:x?}")
            });

        let hash = self
            .hasher
            .hash_password(hex.as_str())
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let lookup = Self::lookup_from_hex(&hex);

        Ok((hash, lookup))
    }

    fn lookup_of(&self, code: &MfaRecoveryCode) -> String {
        let hex = code
            .0
            .iter()
            .fold(String::with_capacity(code.0.len() * 2), |accu, byte| {
                format!("{accu}{byte:x?}")
            });
        Self::lookup_from_hex(&hex)
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

impl<const L: usize, H: HasherRepository> RandBytesRecoveryCodeRepository<L, H> {
    /// Derive the fast lookup key from a recovery code's hex representation:
    /// the first `LOOKUP_KEY_LEN` hex chars of SHA-256(hex). Cheap to compute
    /// and constant-time comparable, so verification can locate the single
    /// candidate row without running Argon2 against every stored code.
    fn lookup_from_hex(hex: &str) -> String {
        let digest = Sha256::digest(hex.as_bytes());
        let full = format!("{digest:x}");
        full[..LOOKUP_KEY_LEN.min(full.len())].to_string()
    }
}
