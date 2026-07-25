use std::future::Future;

use uuid::Uuid;

use ferriskey_domain::common::app_errors::CoreError;

use super::entity::{PasswordPolicy, UpdatePasswordPolicy};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait PasswordPolicyRepository: Send + Sync {
    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PasswordPolicy>, CoreError>> + Send;

    fn upsert(
        &self,
        realm_id: Uuid,
        update: UpdatePasswordPolicy,
    ) -> impl Future<Output = Result<PasswordPolicy, CoreError>> + Send;
}
