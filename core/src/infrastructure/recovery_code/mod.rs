pub mod formatters;

use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::crypto::entities::HashResult;
use crate::domain::trident::entities::MfaRecoveryCode;
use crate::domain::trident::ports::{RecoveryCodeFormatter, RecoveryCodeRepository};
use crate::infrastructure::recovery_code::formatters::{
    B32Split4RecoveryCodeFormatter, RecoveryCodeFormat,
};
use crate::infrastructure::repositories::argon2_hasher::Argon2HasherRepository;
use crate::infrastructure::repositories::random_bytes_recovery_code::RandBytesRecoveryCodeRepository;

#[derive(Clone)]
pub enum RecoveryCodeRepoAny {
    RandomBytes10(RandBytesRecoveryCodeRepository<10, Argon2HasherRepository>),
}

impl RecoveryCodeRepository for RecoveryCodeRepoAny {
    fn generate_recovery_code(&self) -> MfaRecoveryCode {
        match self {
            RecoveryCodeRepoAny::RandomBytes10(repo) => repo.generate_recovery_code(),
        }
    }

    async fn secure_for_storage(&self, code: &MfaRecoveryCode) -> Result<HashResult, CoreError> {
        match self {
            RecoveryCodeRepoAny::RandomBytes10(repo) => repo.secure_for_storage(code).await,
        }
    }

    async fn verify(
        &self,
        in_code: &MfaRecoveryCode,
        secret_code: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> Result<bool, CoreError> {
        match self {
            RecoveryCodeRepoAny::RandomBytes10(repo) => {
                repo.verify(in_code, secret_code, hash_iterations, algorithm, salt)
                    .await
            }
        }
    }
}

impl RecoveryCodeRepoAny {
    /// Formats the code in human-readable format
    pub fn format_code(&self, code: &MfaRecoveryCode, format: RecoveryCodeFormat) -> String {
        match format {
            RecoveryCodeFormat::B32Split4 => B32Split4RecoveryCodeFormatter::format(code),
        }
    }

    /// Decodes a human-readable formatted code into an MfaRecoveryCode
    pub fn decode_string(
        &self,
        code: String,
        format: RecoveryCodeFormat,
    ) -> Result<MfaRecoveryCode, CoreError> {
        match format {
            RecoveryCodeFormat::B32Split4 => B32Split4RecoveryCodeFormatter::decode(code),
        }
    }
}
