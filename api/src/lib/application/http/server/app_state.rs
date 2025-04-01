use std::sync::Arc;

use crate::domain::{
    authentication::ports::AuthenticationService, client::ports::ClientService,
    credential::ports::CredentialService, realm::ports::RealmService,
};

#[derive(Debug, Clone)]
pub struct AppState<R, C, CR, A>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
    A: AuthenticationService,
{
    pub realm_service: Arc<R>,
    pub client_service: Arc<C>,
    pub credential_service: Arc<CR>,
    pub authentication_service: Arc<A>,
}

impl<R, C, CR, A> AppState<R, C, CR, A>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
    A: AuthenticationService,
{
    pub fn new(
        realm_service: Arc<R>,
        client_service: Arc<C>,
        credential_service: Arc<CR>,
        authentication_service: Arc<A>,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
            authentication_service,
        }
    }
}
