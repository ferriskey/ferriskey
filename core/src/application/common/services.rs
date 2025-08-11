use crate::domain::health::services::HealthCheckServiceImpl;
use crate::domain::{
    authentication::services::{
        auth_session_service::AuthSessionServiceImpl, grant_type_service::GrantTypeServiceImpl,
    },
    client::services::{
        client_service::ClientServiceImpl, redirect_uri_service::RedirectUriServiceImpl,
    },
    credential::services::CredentialServiceImpl,
    crypto::services::CryptoServiceImpl,
    jwt::services::JwtServiceImpl,
    realm::services::RealmServiceImpl,
    role::services::RoleServiceImpl,
    user::services::{user_role_service::UserRoleServiceImpl, user_service::UserServiceImpl},
};
use crate::infrastructure::auth_session::AuthSessionRepoAny;
use crate::infrastructure::client::ClientRepoAny;
use crate::infrastructure::client::repositories::RedirectUriRepoAny;
use crate::infrastructure::credential::CredentialRepoAny;
use crate::infrastructure::hasher::HasherRepoAny;
use crate::infrastructure::health::HealthCheckRepoAny;
use crate::infrastructure::jwt::KeyStoreRepoAny;
use crate::infrastructure::realm::RealmRepoAny;
use crate::infrastructure::refresh_token::RefreshTokenRepoAny;
use crate::infrastructure::role::RoleRepoAny;
use crate::infrastructure::user::UserRepoAny;
use crate::infrastructure::user::repositories::user_required_action_repository::UserRequiredActionRepoAny;
use crate::infrastructure::user::repositories::user_role_repository::UserRoleRepoAny;

pub type DefaultUserService =
    UserServiceImpl<UserRepoAny, RealmRepoAny, UserRoleRepoAny, UserRequiredActionRepoAny>;

pub type DefaultRealmService =
    RealmServiceImpl<RealmRepoAny, ClientRepoAny, RoleRepoAny, UserRepoAny, UserRoleRepoAny>;

pub type DefaultAuthSessionService = AuthSessionServiceImpl<AuthSessionRepoAny>;
pub type DefaultGrantTypeService = GrantTypeServiceImpl<
    RefreshTokenRepoAny,
    KeyStoreRepoAny,
    RealmRepoAny,
    ClientRepoAny,
    UserRepoAny,
    UserRoleRepoAny,
    UserRequiredActionRepoAny,
    CredentialRepoAny,
    HasherRepoAny,
    AuthSessionRepoAny,
>;

pub type DefaultClientService = ClientServiceImpl<ClientRepoAny, UserRepoAny, RealmRepoAny>;

pub type DefaultCredentialService = CredentialServiceImpl<CredentialRepoAny, HasherRepoAny>;

pub type DefaultCryptoService = CryptoServiceImpl<HasherRepoAny>;

pub type DefaultRoleService = RoleServiceImpl<RoleRepoAny>;

pub type DefaultUserRoleService =
    UserRoleServiceImpl<UserRepoAny, RoleRepoAny, RealmRepoAny, UserRoleRepoAny>;

pub type DefaultJwtService = JwtServiceImpl<RefreshTokenRepoAny, KeyStoreRepoAny, RealmRepoAny>;

pub type DefaultRedirectUriService =
    RedirectUriServiceImpl<RealmRepoAny, RedirectUriRepoAny, ClientRepoAny>;

pub type DefaultHealthCheckService = HealthCheckServiceImpl<HealthCheckRepoAny>;
