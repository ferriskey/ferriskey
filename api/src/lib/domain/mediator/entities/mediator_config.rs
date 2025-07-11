use std::sync::Arc;

use crate::{
    domain::{
        client::services::{
            client_service::DefaultClientService, redirect_uri_service::DefaultRedirectUriService,
        },
        credential::services::credential_service::DefaultCredentialService,
        jwt::services::jwt_service::DefaultJwtService,
        realm::services::realm_service::DefaultRealmService,
        role::services::DefaultRoleService,
        user::services::{
            user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
        },
    },
    env::Env,
};

pub struct MediatorConfig {
    pub env: Arc<Env>,
    pub client_service: DefaultClientService,
    pub realm_service: DefaultRealmService,
    pub user_service: DefaultUserService,
    pub credential_service: DefaultCredentialService,
    pub redirect_uri_service: DefaultRedirectUriService,
    pub role_service: DefaultRoleService,
    pub user_role_service: DefaultUserRoleService,
    pub jwt_service: DefaultJwtService,
}
