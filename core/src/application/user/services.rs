use crate::{
    application::common::{FerriskeyService, policies::ensure_policy},
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        credential::{
            entities::{CredentialOverview, GetCredentialsInput},
            ports::{CredentialRepository, CredentialService},
        },
        realm::ports::RealmRepository,
        user::ports::UserPolicy,
    },
    infrastructure::user,
};

impl CredentialService for FerriskeyService {
    async fn create_custom_credential(
        &self,
        user_id: uuid::Uuid,
        credential_type: String,
        secret_data: String,
        label: Option<String>,
        credential_data: serde_json::Value,
    ) -> Result<
        crate::domain::credential::entities::Credential,
        crate::domain::credential::entities::CredentialError,
    > {
        unimplemented!()
    }

    async fn create_password_credential(
        &self,
        user_id: uuid::Uuid,
        password: String,
        label: String,
    ) -> Result<
        crate::domain::credential::entities::Credential,
        crate::domain::credential::entities::CredentialError,
    > {
        unimplemented!()
    }

    async fn delete_by_id(
        &self,
        credential_id: uuid::Uuid,
    ) -> Result<(), crate::domain::credential::entities::CredentialError> {
        unimplemented!()
    }

    async fn get_credentials(
        &self,
        identity: Identity,
        input: GetCredentialsInput,
    ) -> Result<Vec<CredentialOverview>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(identity, realm).await,
            "insufficient permissions",
        )?;

        let credentials = self
            .credential_repository
            .get_credentials_by_user_id(input.user_id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        Ok(credentials
            .into_iter()
            .map(CredentialOverview::from)
            .collect())
    }

    async fn get_credentials_by_user_id(
        &self,
        user_id: uuid::Uuid,
    ) -> Result<
        Vec<crate::domain::credential::entities::Credential>,
        crate::domain::credential::entities::CredentialError,
    > {
        unimplemented!()
    }

    async fn reset_password(
        &self,
        user_id: uuid::Uuid,
        password: String,
        temporary: bool,
    ) -> Result<(), crate::domain::credential::entities::CredentialError> {
        unimplemented!()
    }

    async fn verify_password(
        &self,
        user_id: uuid::Uuid,
        password: String,
    ) -> Result<bool, crate::domain::credential::entities::CredentialError> {
        unimplemented!()
    }

    async fn delete_credential(
        &self,
        identity: Identity,
        input: crate::domain::credential::entities::DeleteCredentialInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_user(identity, realm).await,
            "insufficient permissions",
        )?;

        self.credential_repository
            .delete_by_id(input.credential_id)
            .await
            .map_err(|_| CoreError::DeleteCredentialError)?;

        // @TODO: implement webhook notifier

        Ok(())
    }
}
