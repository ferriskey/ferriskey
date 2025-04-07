use std::sync::Arc;

use crate::domain::{
    authentication::ports::{
        auth_session::AuthSessionService, authentication::AuthenticationService,
    },
    client::ports::client_service::ClientService,
    credential::ports::credential_service::CredentialService,
    realm::ports::realm_service::RealmService,
};

#[derive(Clone)]
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
    pub auth_session_service: Arc<dyn AuthSessionService>,
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
        auth_session_service: Arc<dyn AuthSessionService>,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
            authentication_service,
            auth_session_service,
        }
    }
}
