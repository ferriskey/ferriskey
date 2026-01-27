use crate::domain::account::entities::{AccountError, AccountHint};
use crate::domain::realm::entities::RealmId;
use uuid::Uuid;

pub trait AccountHintService: Send + Sync {
    fn update_last_used_at(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<AccountHint, AccountError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait AccountHintRepository: Send + Sync {
    fn update(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<AccountHint, AccountError>> + Send;
    fn add_account(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
        display_name: &str,
        avatar_url: &Option<String>,
    ) -> impl Future<Output = Result<AccountHint, AccountError>> + Send;
    fn get_account(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<AccountHint, AccountError>> + Send;
    fn get_accounts(
        &self,
        user_id: &Uuid,
        realm_id: &RealmId,
    ) -> impl Future<Output = Result<Vec<AccountHint>, AccountError>> + Send;
}
