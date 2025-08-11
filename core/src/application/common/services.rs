use crate::domain::health::services::HealthCheckServiceImpl;
use crate::infrastructure::client::ClientRepoAny;
use crate::infrastructure::health::repositories::PostgresHealthCheckRepository;
use crate::infrastructure::realm::RealmRepoAny;
use crate::infrastructure::user::UserRepoAny;
use crate::{
    domain::{
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
        session::services::UserSessionServiceImpl,
        user::services::{user_role_service::UserRoleServiceImpl, user_service::UserServiceImpl},
    },
    infrastructure::{
        repositories::{
            argon2_hasher::Argon2HasherRepository,
            auth_session_repository::PostgresAuthSessionRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            redirect_uri_repository::PostgresRedirectUriRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
            role_repository::PostgresRoleRepository,
            user_session_repository::PostgresUserSessionRepository,
        },
        user::repositories::{
            user_required_action_repository::PostgresUserRequiredActionRepository,
            user_role_repository::PostgresUserRoleRepository,
        },
    },
};

pub type DefaultUserService = UserServiceImpl<
    UserRepoAny,
    RealmRepoAny,
    PostgresUserRoleRepository,
    PostgresUserRequiredActionRepository,
>;

pub type DefaultRealmService = RealmServiceImpl<
    RealmRepoAny,
    ClientRepoAny,
    PostgresRoleRepository,
    UserRepoAny,
    PostgresUserRoleRepository,
>;

pub type DefaultAuthSessionService = AuthSessionServiceImpl<PostgresAuthSessionRepository>;
pub type DefaultGrantTypeService = GrantTypeServiceImpl<
    PostgresRefreshTokenRepository,
    PostgresKeyStoreRepository,
    RealmRepoAny,
    ClientRepoAny,
    UserRepoAny,
    PostgresUserRoleRepository,
    PostgresUserRequiredActionRepository,
    PostgresCredentialRepository,
    Argon2HasherRepository,
    PostgresAuthSessionRepository,
>;

pub type DefaultClientService = ClientServiceImpl<ClientRepoAny, UserRepoAny, RealmRepoAny>;

pub type DefaultCredentialService =
    CredentialServiceImpl<PostgresCredentialRepository, Argon2HasherRepository>;

pub type DefaultCryptoService = CryptoServiceImpl<Argon2HasherRepository>;

pub type DefaultRoleService = RoleServiceImpl<PostgresRoleRepository>;

pub type DefaultUserRoleService = UserRoleServiceImpl<
    UserRepoAny,
    PostgresRoleRepository,
    RealmRepoAny,
    PostgresUserRoleRepository,
>;

pub type DefaultUserSessionService = UserSessionServiceImpl<PostgresUserSessionRepository>;

pub type DefaultJwtService =
    JwtServiceImpl<PostgresRefreshTokenRepository, PostgresKeyStoreRepository, RealmRepoAny>;

pub type DefaultRedirectUriService =
    RedirectUriServiceImpl<RealmRepoAny, PostgresRedirectUriRepository, ClientRepoAny>;

pub type DefaultHealthCheckService = HealthCheckServiceImpl<PostgresHealthCheckRepository>;
