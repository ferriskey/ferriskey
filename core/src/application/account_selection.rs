use chrono::Utc;
use uuid::Uuid;

use crate::ApplicationService;
use crate::domain::authentication::ports::{
    AccountSelectionService, AttachedDevice, AuthorizeAccounts, ListAccountsInput,
    SelectAccountInput, SelectAccountOutcome,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::device_session::entities::AccountEntry;
use crate::domain::device_session::ports::AccountChooserService;
use crate::domain::realm::entities::RealmScope;

impl ApplicationService {
    async fn scope_of_realm(&self, realm_name: &str) -> Result<RealmScope, CoreError> {
        RealmScope::resolve(self.auth_service.realm_repository.as_ref(), realm_name).await
    }

    async fn entries_in_scope(
        &self,
        scope: &RealmScope,
        input: ListAccountsInput,
    ) -> Result<Vec<AccountEntry>, CoreError> {
        let Some(client_id) = self
            .auth_service
            .client_uuid_of(scope, &input.client_id)
            .await
        else {
            return Ok(Vec::new());
        };

        let stored_max_age = match input.session_code {
            Some(session_code) => self
                .auth_service
                .open_selection_auth_session(scope, session_code)
                .await
                .ok()
                .filter(|auth_session| auth_session.client_id == client_id)
                .and_then(|auth_session| auth_session.max_age),
            None => None,
        };

        self.account_chooser_service
            .list_entries(
                input.device_token.as_deref(),
                scope,
                input.active_sso_token.as_deref(),
                stricter_max_age(stored_max_age, input.max_age),
                client_id,
            )
            .await
    }
}

fn chooser_enabled(scope: &RealmScope) -> bool {
    scope
        .realm()
        .settings
        .as_ref()
        .is_some_and(|settings| settings.account_chooser_enabled)
}

fn stricter_max_age(stored: Option<i64>, requested: Option<i64>) -> Option<i64> {
    match (stored, requested) {
        (Some(stored), Some(requested)) => Some(stored.min(requested)),
        (stored, requested) => stored.or(requested),
    }
}

fn is_refusal(error: &CoreError) -> bool {
    matches!(
        error,
        CoreError::ReauthenticationRequired | CoreError::Forbidden(_) | CoreError::AccountLocked
    )
}

impl AccountSelectionService for ApplicationService {
    async fn authorize_accounts(
        &self,
        input: ListAccountsInput,
    ) -> Result<AuthorizeAccounts, CoreError> {
        let scope = self.scope_of_realm(&input.realm_name).await?;

        if !chooser_enabled(&scope) {
            return Ok(AuthorizeAccounts {
                chooser_enabled: false,
                entries: Vec::new(),
            });
        }

        Ok(AuthorizeAccounts {
            chooser_enabled: true,
            entries: self.entries_in_scope(&scope, input).await?,
        })
    }

    async fn select_account(
        &self,
        input: SelectAccountInput,
    ) -> Result<SelectAccountOutcome, CoreError> {
        let scope = self.scope_of_realm(&input.realm_name).await?;

        if !chooser_enabled(&scope) {
            return Err(CoreError::DeviceSessionNotFound);
        }

        let auth_session = self
            .auth_service
            .open_selection_auth_session(&scope, input.session_code)
            .await?;

        let selected = match self
            .account_chooser_service
            .select_entry(
                input.device_token.as_deref(),
                &scope,
                input.session_id,
                stricter_max_age(auth_session.max_age, input.max_age),
                auth_session.client_id,
            )
            .await
        {
            Ok(selected) => selected,
            Err(error) if is_refusal(&error) => {
                let login_hint = self
                    .auth_service
                    .remember_reauthentication_into(&scope, input.session_code, input.session_id)
                    .await?;

                return Ok(SelectAccountOutcome::RequiresCredentials { login_hint });
            }
            Err(error) => return Err(error),
        };

        let output = self
            .auth_service
            .finalize_selected_session(
                &scope,
                auth_session,
                input.session_code,
                selected,
                &input.webapp_base_url,
            )
            .await?;

        Ok(SelectAccountOutcome::Resumed(Box::new(output)))
    }

    async fn list_accounts(
        &self,
        input: ListAccountsInput,
    ) -> Result<Vec<AccountEntry>, CoreError> {
        let scope = self.scope_of_realm(&input.realm_name).await?;

        self.entries_in_scope(&scope, input).await
    }

    async fn detach_account(
        &self,
        realm_name: String,
        device_token: Option<String>,
        session_id: Uuid,
    ) -> Result<(), CoreError> {
        let scope = self.scope_of_realm(&realm_name).await?;

        if !chooser_enabled(&scope) {
            return Err(CoreError::DeviceSessionNotFound);
        }

        self.account_chooser_service
            .detach_entry(device_token.as_deref(), &scope, session_id)
            .await
    }

    async fn attach_login_to_device(
        &self,
        realm_name: String,
        device_token: Option<String>,
        sso_cookie: String,
    ) -> Result<Option<AttachedDevice>, CoreError> {
        let scope = self.scope_of_realm(&realm_name).await?;

        if !chooser_enabled(&scope) {
            return Ok(None);
        }

        let Some(session_id) = self
            .auth_service
            .session_id_of_sso_secret(&scope, &sso_cookie)
            .await
        else {
            return Ok(None);
        };

        let (device, token) = self
            .account_chooser_service
            .attach_session_to_device(device_token.as_deref(), &scope, session_id)
            .await?;

        Ok(Some(AttachedDevice {
            token,
            max_age_secs: (device.expires_at - Utc::now()).num_seconds().max(0),
        }))
    }

    async fn session_id_of_sso_cookie(
        &self,
        realm_name: String,
        sso_cookie: String,
    ) -> Result<Option<Uuid>, CoreError> {
        let scope = self.scope_of_realm(&realm_name).await?;

        Ok(self
            .auth_service
            .session_id_of_sso_secret(&scope, &sso_cookie)
            .await)
    }
}

#[cfg(test)]
mod tests {
    use super::stricter_max_age;

    #[test]
    fn no_limit_on_either_side_means_no_limit() {
        assert_eq!(stricter_max_age(None, None), None);
    }

    #[test]
    fn a_stored_limit_survives_an_omitted_request_limit() {
        assert_eq!(stricter_max_age(Some(300), None), Some(300));
    }

    #[test]
    fn a_request_limit_applies_when_nothing_is_stored() {
        assert_eq!(stricter_max_age(None, Some(60)), Some(60));
    }

    #[test]
    fn the_smaller_limit_wins() {
        assert_eq!(stricter_max_age(Some(300), Some(60)), Some(60));
        assert_eq!(stricter_max_age(Some(300), Some(86_400)), Some(300));
    }

    #[test]
    fn zero_is_the_strictest_limit() {
        assert_eq!(stricter_max_age(Some(0), Some(300)), Some(0));
        assert_eq!(stricter_max_age(Some(300), Some(0)), Some(0));
        assert_eq!(stricter_max_age(None, Some(0)), Some(0));
    }
}
