use std::net::IpAddr;

use crate::domain::client_registration::entities::{
    ClientRegistrationRequest, ClientRegistrationResponse, RegistrationError,
};

pub trait ClientRegistrationService: Send + Sync {
    fn register(
        &self,
        realm_name: &str,
        source_ip: Option<IpAddr>,
        request: ClientRegistrationRequest,
    ) -> impl Future<Output = Result<ClientRegistrationResponse, RegistrationError>> + Send;
}
