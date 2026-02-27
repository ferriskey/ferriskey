use uuid::Uuid;

use crate::domain::account::{
    entities::AccountHint,
    ports::{AccountHintRepository, AccountHintService},
};
use crate::domain::common::entities::app_errors::CoreError;
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
    async fn create_account_hint(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
        display_name: &str,
        avatar_url: Option<String>,
    ) -> Result<AccountHint, CoreError> {
        self.account_hint_repository
            .add_hint(&user_id, &realm_id, display_name, &avatar_url)
            .await
    }

    async fn update_account_hint(
        &self,
        account_hint: AccountHint,
    ) -> Result<AccountHint, CoreError> {
        self.account_hint_repository
            .get_hints(&account_hint.user_id, &account_hint.realm_id)
            .await?;

        self.account_hint_repository
            .update_hint(&account_hint.user_id, &account_hint.realm_id)
            .await
    }

    async fn get_account_hints(
        &self,
        user_id: Uuid,
        realm_id: RealmId,
    ) -> Result<Vec<AccountHint>, CoreError> {
        match self
            .account_hint_repository
            .get_hints(&user_id, &realm_id)
            .await
        {
            Ok(account_hints) => Ok(account_hints),
            Err(CoreError::HintsNotFound) => Err(CoreError::HintsNotFound),
            Err(e) => Err(e),
        }
    }
}
