use chrono::Utc;
use uuid::Uuid;

use crate::ApplicationService;
use crate::domain::authentication::ports::{
    AccountSelectionService, AttachedDevice, ListAccountsInput, SelectAccountInput,
    SelectAccountOutcome,
};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::device_session::entities::AccountEntry;
use crate::domain::device_session::ports::AccountChooserService;
use crate::domain::realm::entities::RealmScope;

impl ApplicationService {
    async fn scope_of_realm(&self, realm_name: &str) -> Result<RealmScope, CoreError> {
        RealmScope::resolve(self.auth_service.realm_repository.as_ref(), realm_name).await
    }
}

fn chooser_enabled(scope: &RealmScope) -> bool {
    scope
        .realm()
        .settings
        .as_ref()
        .is_some_and(|settings| settings.account_chooser_enabled)
}

fn is_refusal(error: &CoreError) -> bool {
    matches!(
        error,
        CoreError::ReauthenticationRequired | CoreError::Forbidden(_) | CoreError::AccountLocked
    )
}

impl AccountSelectionService for ApplicationService {
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
                input.max_age,
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

        let Some(client_id) = self
            .auth_service
            .client_uuid_of(&scope, &input.client_id)
            .await
        else {
            return Ok(Vec::new());
        };

        self.account_chooser_service
            .list_entries(
                input.device_token.as_deref(),
                &scope,
                input.active_sso_token.as_deref(),
                input.max_age,
                client_id,
            )
            .await
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
