use std::sync::Arc;

use crate::domain::{
    authentication::service::{
        auth_session::DefaultAuthSessionService, authentication::DefaultAuthenticationService,
    },
    client::services::{
        client_service::DefaultClientService, redirect_uri_service::DefaultRedirectUriService,
    },
    credential::services::credential_service::DefaultCredentialService,
    jwt::services::jwt_service::DefaultJwtService,
    realm::services::realm_service::DefaultRealmService,
    role::services::DefaultRoleService,
    user::services::{user_role_service::DefaultUserRoleService, user_service::DefaultUserService},
};

#[derive(Clone)]
pub struct AppState {
    pub realm_service: Arc<DefaultRealmService>,
    pub client_service: Arc<DefaultClientService>,
    pub credential_service: Arc<DefaultCredentialService>,
    pub authentication_service: Arc<DefaultAuthenticationService>,
    pub auth_session_service: Arc<DefaultAuthSessionService>,
    pub user_service: Arc<DefaultUserService>,
    pub jwt_service: Arc<DefaultJwtService>,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub role_service: DefaultRoleService,
    pub user_role_service: DefaultUserRoleService,
}

impl AppState {
    pub fn new(
        realm_service: Arc<DefaultRealmService>,
        client_service: Arc<DefaultClientService>,
        credential_service: Arc<DefaultCredentialService>,
        authentication_service: Arc<DefaultAuthenticationService>,
        auth_session_service: Arc<DefaultAuthSessionService>,
        user_service: Arc<DefaultUserService>,
        jwt_service: Arc<DefaultJwtService>,
        redirect_uri_service: DefaultRedirectUriService,
        role_service: DefaultRoleService,
        user_role_service: DefaultUserRoleService,
    ) -> Self {
        Self {
            realm_service,
            client_service,
            credential_service,
            authentication_service,
            auth_session_service,
            user_service,
            jwt_service,
            redirect_uri_service,
            role_service,
            user_role_service,
        }
    }
}
