use std::sync::Arc;

use crate::domain::{
    authentication::service::{
        auth_session::DefaultAuthSessionService, authentication::DefaultAuthenticationService,
    },
    client::services::client_service::DefaultClientService,
    credential::services::credential_service::DefaultCredentialService,
    realm::services::realm_service::DefaultRealmService,
    user::services::user_service::DefaultUserService,
};

#[derive(Clone)]
pub struct AppState {
    pub realm_service: Arc<DefaultRealmService>,
    pub client_service: Arc<DefaultClientService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub authentication_service: Arc<DefaultAuthenticationService>,
    pub auth_session_service: Arc<DefaultAuthSessionService>,
    pub user_service: Arc<DefaultUserService>,
}

impl AppState {
    pub fn new(
        realm_service: Arc<DefaultRealmService>,
        client_service: Arc<DefaultClientService>,
        credential_service: Arc<DefaultCredentialService>,
        authentication_service: Arc<DefaultAuthenticationService>,
        auth_session_service: Arc<DefaultAuthSessionService>,
        user_service: Arc<DefaultUserService>,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
            authentication_service,
            auth_session_service,
            user_service,
        }
    }
}
