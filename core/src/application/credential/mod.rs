use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        credential::{
            entities::{CredentialOverview, DeleteCredentialInput, GetCredentialsInput},
            ports::CredentialService,
        },
    },
};

impl CredentialService for ApplicationService {
    async fn delete_credential(
        &self,
        identity: Identity,
        input: DeleteCredentialInput,
    ) -> Result<(), CoreError> {
        self.credential_service
            .delete_credential(identity, input)
            .await
    }

    async fn get_credentials(
        &self,
        identity: Identity,
        input: GetCredentialsInput,
    ) -> Result<Vec<CredentialOverview>, CoreError> {
        self.credential_service
            .get_credentials(identity, input)
            .await
    }
}
