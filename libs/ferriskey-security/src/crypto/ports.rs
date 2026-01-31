use crate::{SecurityError, crypto::entities::HashResult};

pub trait CryptoService: Send + Sync {
    fn hash_password(
        &self,
        password: &str,
    ) -> impl Future<Output = Result<HashResult, SecurityError>> + Send;
    fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> impl Future<Output = Result<bool, SecurityError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait HasherRepository: Send + Sync {
    fn hash_password(
        &self,
        password: &str,
    ) -> impl Future<Output = Result<HashResult, SecurityError>> + Send;
    fn verify_password(
        &self,
        password: &str,
        secret_data: &str,
        hash_iterations: u32,
        algorithm: &str,
        salt: &str,
    ) -> impl Future<Output = Result<bool, SecurityError>> + Send;
}
