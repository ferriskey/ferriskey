use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

use super::entities::EmailVerificationToken;

#[derive(Debug, Clone)]
pub struct CreateEmailVerificationTokenInput {
    pub user_id: Uuid,
    pub realm_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[cfg_attr(test, mockall::automock)]
pub trait EmailVerificationTokenRepository: Send + Sync {
    fn create(
        &self,
        input: CreateEmailVerificationTokenInput,
    ) -> impl std::future::Future<Output = Result<EmailVerificationToken, CoreError>> + Send;

    fn find_valid_by_hash(
        &self,
        token_hash: &str,
    ) -> impl std::future::Future<Output = Result<Option<EmailVerificationToken>, CoreError>> + Send;

    fn mark_used(
        &self,
        id: Uuid,
    ) -> impl std::future::Future<Output = Result<(), CoreError>> + Send;

    fn delete_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl std::future::Future<Output = Result<u64, CoreError>> + Send;
}

pub trait EmailVerificationService: Send + Sync {
    fn send_verification_email(
        &self,
        user_id: Uuid,
        realm_name: String,
        base_url: String,
    ) -> impl std::future::Future<Output = Result<(), CoreError>> + Send;

    fn verify_email(
        &self,
        token: String,
    ) -> impl std::future::Future<Output = Result<VerifyEmailResult, CoreError>> + Send;
}

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct VerifyEmailResult {
    pub user_id: Uuid,
    pub verified: bool,
}
