use std::sync::Arc;

use crate::domain::{
    client::ports::ClientService, credential::ports::CredentialService, realm::ports::RealmService,
};

#[derive(Debug, Clone)]
pub struct AppState<R, C, CR>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
{
    pub realm_service: Arc<R>,
    pub client_service: Arc<C>,
    pub credential_service: Arc<CR>,
}

impl<R, C, CR> AppState<R, C, CR>
where
    R: RealmService,
    C: ClientService,
    CR: CredentialService,
{
    pub fn new(realm_service: Arc<R>, client_service: Arc<C>, credential_service: Arc<CR>) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
        }
    }
}
