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
            client_repository::PostgresClientRepository,
            credential_repository::PostgresCredentialRepository,
            keystore_repository::PostgresKeyStoreRepository,
            realm_repository::PostgresRealmRepository,
            redirect_uri_repository::PostgresRedirectUriRepository,
            refresh_token_repository::PostgresRefreshTokenRepository,
            role_repository::PostgresRoleRepository,
            user_session_repository::PostgresUserSessionRepository,
        },
        user::{
            repositories::{
                user_required_action_repository::PostgresUserRequiredActionRepository,
                user_role_repository::PostgresUserRoleRepository,
            },
            repository::PostgresUserRepository,
        },
    },
};

pub type DefaultUserService = UserServiceImpl<
    PostgresUserRepository,
    PostgresRealmRepository,
    PostgresUserRoleRepository,
    PostgresUserRequiredActionRepository,
>;

pub type DefaultRealmService = RealmServiceImpl<
    PostgresRealmRepository,
    PostgresClientRepository,
    PostgresRoleRepository,
    PostgresUserRepository,
    PostgresUserRoleRepository,
>;

pub type DefaultAuthSessionService = AuthSessionServiceImpl<PostgresAuthSessionRepository>;
pub type DefaultGrantTypeService = GrantTypeServiceImpl<
    PostgresRefreshTokenRepository,
    PostgresKeyStoreRepository,
    PostgresRealmRepository,
    PostgresClientRepository,
    PostgresUserRepository,
    PostgresUserRoleRepository,
    PostgresUserRequiredActionRepository,
    PostgresCredentialRepository,
    Argon2HasherRepository,
    PostgresAuthSessionRepository,
>;

pub type DefaultClientService =
    ClientServiceImpl<PostgresClientRepository, PostgresUserRepository, PostgresRealmRepository>;

pub type DefaultCredentialService =
    CredentialServiceImpl<PostgresCredentialRepository, Argon2HasherRepository>;

pub type DefaultCryptoService = CryptoServiceImpl<Argon2HasherRepository>;

pub type DefaultRoleService = RoleServiceImpl<PostgresRoleRepository>;

pub type DefaultUserRoleService = UserRoleServiceImpl<
    PostgresUserRepository,
    PostgresRoleRepository,
    PostgresRealmRepository,
    PostgresUserRoleRepository,
>;

pub type DefaultUserSessionService = UserSessionServiceImpl<PostgresUserSessionRepository>;

pub type DefaultJwtService = JwtServiceImpl<
    PostgresRefreshTokenRepository,
    PostgresKeyStoreRepository,
    PostgresRealmRepository,
>;

pub type DefaultRedirectUriService = RedirectUriServiceImpl<
    PostgresRealmRepository,
    PostgresRedirectUriRepository,
    PostgresClientRepository,
>;
