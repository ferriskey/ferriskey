use std::net::IpAddr;

use crate::application::services::ApplicationService;
use crate::domain::client_registration::entities::{
    ClientRegistrationRequest, ClientRegistrationResponse, RegistrationError,
};
use crate::domain::client_registration::ports::ClientRegistrationService;

impl ApplicationService {
    pub async fn register_client(
        &self,
        realm_name: &str,
        source_ip: Option<IpAddr>,
        request: ClientRegistrationRequest,
    ) -> Result<ClientRegistrationResponse, RegistrationError> {
        self.client_registration_service
            .register(realm_name, source_ip, request)
            .await
    }
}
