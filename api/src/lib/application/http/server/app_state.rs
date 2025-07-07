use std::sync::Arc;

use crate::{
    domain::{
        authentication::service::{
            auth_session::DefaultAuthSessionService, authentication::DefaultAuthenticationService,
        },
        client::services::{
            client_service::DefaultClientService, redirect_uri_service::DefaultRedirectUriService,
        },
        credential::services::credential_service::DefaultCredentialService,
        jwt::services::jwt_service::DefaultJwtService,
        mediator::services::mediator_service::DefaultMediatorService,
        realm::services::realm_service::DefaultRealmService,
        role::services::DefaultRoleService,
        user::services::{
            user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
        },
    },
    env::Env,
};

#[derive(Clone)]
pub struct AppState {
    pub realm_service: DefaultRealmService,
    pub client_service: DefaultClientService,
    pub credential_service: DefaultCredentialService,
    pub authentication_service: DefaultAuthenticationService,
    pub auth_session_service: DefaultAuthSessionService,
    pub user_service: DefaultUserService,
    pub jwt_service: DefaultJwtService,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub role_service: DefaultRoleService,
    pub user_role_service: DefaultUserRoleService,
    pub mediator_service: DefaultMediatorService,
    pub env: Arc<Env>,
}
