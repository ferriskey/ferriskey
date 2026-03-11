use std::sync::Arc;
use uuid::Uuid;
use crate::domain::common::entities::app_errors::CoreError;
use super::entities::{PasswordPolicy, UpdatePasswordPolicy};

pub trait PasswordPolicyRepository: Send + Sync {
    fn find_by_realm_id(&self, realm_id: Uuid) -> impl Future<Output = Result<Option<PasswordPolicy>, CoreError>> + Send;
    fn upsert(&self, realm_id: Uuid, policy: UpdatePasswordPolicy) -> impl Future<Output = Result<PasswordPolicy, CoreError>> + Send;
}

pub trait PasswordPolicyService: Send + Sync {
    fn get_policy(&self, realm_id: Uuid) -> impl Future<Output = Result<PasswordPolicy, CoreError>> + Send;
    fn update_policy(&self, realm_id: Uuid, update: UpdatePasswordPolicy) -> impl Future<Output = Result<PasswordPolicy, CoreError>> + Send;
    fn validate_password(&self, password: &str, policy: &PasswordPolicy) -> Result<(), CoreError>;
}
