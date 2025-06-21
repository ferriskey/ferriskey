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
    pub mediator_service: Arc<DefaultMediatorService>,
    pub env: Arc<Env>,
}
