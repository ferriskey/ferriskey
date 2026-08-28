use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::saml::entities::{FinishSsoInput, SamlAssertionDelivery, StartSsoInput};
use crate::domain::saml::ports::SamlService;

use super::services::ApplicationService;

impl SamlService for ApplicationService {
    async fn start_sso(
        &self,
        input: StartSsoInput,
    ) -> Result<crate::domain::authentication::entities::AuthOutput, CoreError> {
        self.saml_service.start_sso(input).await
    }

    async fn idp_signing_certificate(&self, realm_name: String) -> Result<String, CoreError> {
        self.saml_service.idp_signing_certificate(realm_name).await
    }

    async fn finish_sso(&self, input: FinishSsoInput) -> Result<SamlAssertionDelivery, CoreError> {
        self.saml_service.finish_sso(input).await
    }
}
