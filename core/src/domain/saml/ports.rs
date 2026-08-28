use uuid::Uuid;

use crate::domain::authentication::entities::AuthOutput;
use crate::domain::client::entities::saml::{ClientSamlConfig, SamlAttributeMapper, SpEntityId};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::saml::entities::{FinishSsoInput, SamlAssertionDelivery, StartSsoInput};

#[cfg_attr(test, mockall::automock)]
pub trait SamlServiceProviderRepository: Send + Sync {
    fn get_by_entity_id(
        &self,
        realm_id: RealmId,
        sp_entity_id: SpEntityId,
    ) -> impl Future<Output = Result<Option<ClientSamlConfig>, CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Option<ClientSamlConfig>, CoreError>> + Send;

    fn get_attribute_mappers(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<SamlAttributeMapper>, CoreError>> + Send;
}

pub trait SamlService: Send + Sync {
    fn start_sso(
        &self,
        input: StartSsoInput,
    ) -> impl Future<Output = Result<AuthOutput, CoreError>> + Send;

    fn finish_sso(
        &self,
        input: FinishSsoInput,
    ) -> impl Future<Output = Result<SamlAssertionDelivery, CoreError>> + Send;

    fn idp_signing_certificate(
        &self,
        realm_name: String,
    ) -> impl Future<Output = Result<String, CoreError>> + Send;
}
