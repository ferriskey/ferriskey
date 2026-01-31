use std::future::Future;
use uuid::Uuid;

use crate::domain::account::entities::AccountHint;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

pub trait AccountHintService: Send + Sync {
    fn create_account_hint(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        display_name: &str,
        avatar_url: Option<String>,
    ) -> impl Future<Output = Result<AccountHint, CoreError>> + Send;

    fn update_account_hint(
        &self,
        account_hint: AccountHint,
    ) -> impl Future<Output = Result<AccountHint, CoreError>> + Send;

    fn get_account_hints(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<AccountHint>, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait AccountHintRepository: Send + Sync {
    fn update_hint(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<AccountHint, CoreError>> + Send;
    fn add_hint(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
        display_name: &str,
        avatar_url: &Option<String>,
    ) -> impl Future<Output = Result<AccountHint, CoreError>> + Send;
    fn get_hint(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<AccountHint, CoreError>> + Send;
    fn get_hints(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<Vec<AccountHint>, CoreError>> + Send;
}
