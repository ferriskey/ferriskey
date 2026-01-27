use crate::domain::account::{
    entities::{AccountError, AccountHint},
    ports::{AccountHintRepository, AccountHintService},
};
use crate::domain::realm::entities::RealmId;

#[derive(Clone)]
pub struct AccountHintServiceImpl<A>
where
    A: AccountHintRepository,
{
    pub account_hint_repository: A,
}

impl<A> AccountHintServiceImpl<A>
where
    A: AccountHintRepository,
{
    pub fn new(account_hint_repository: A) -> Self {
        Self {
            account_hint_repository,
        }
    }
}

impl<A> AccountHintService for AccountHintServiceImpl<A>
where
    A: AccountHintRepository,
{
    async fn update_last_used_at(
        &self,
        user_id: uuid::Uuid,
        realm_id: RealmId,
    ) -> Result<AccountHint, AccountError> {
        let account = self
            .account_hint_repository
            .update(&user_id, &realm_id)
            .await?;

        Ok(account)
    }
}
