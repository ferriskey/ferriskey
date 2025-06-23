use std::sync::Arc;

use crate::{
    domain::{
        authentication::{
            ports::auth_session::AuthSessionRepository,
            service::{
                auth_session::DefaultAuthSessionService,
                authentication::DefaultAuthenticationService,
            },
        },
        client::{
            ports::{
                client_repository::ClientRepository, redirect_uri_repository::RedirectUriRepository,
            },
            services::{
                client_service::DefaultClientService,
                redirect_uri_service::DefaultRedirectUriService,
            },
        },
        credential::{
            ports::credential_repository::CredentialRepository,
            services::credential_service::DefaultCredentialService,
        },
        crypto::{
            ports::hasher_repository::HasherRepository,
            services::crypto_service::DefaultCryptoService,
        },
        jwt::{
            ports::{
                jwt_repository::RefreshTokenRepository, keystore_repository::KeyStoreRepository,
            },
            services::jwt_service::DefaultJwtService,
        },
        mediator::{
            entities::mediator_config::MediatorConfig,
            services::mediator_service::DefaultMediatorService,
        },
        realm::{
            ports::realm_repository::RealmRepository, services::realm_service::DefaultRealmService,
        },
        role::{ports::RoleRepository, services::DefaultRoleService},
        user::{
            ports::{user_repository::UserRepository, user_role_repository::UserRoleRepository},
            services::{
                user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
            },
        },
    },
    env::Env,
    infrastructure::{
        db::postgres::Postgres,
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
        },
        user::{
            repositories::user_role_repository::PostgresUserRoleRepository,
            repository::PostgresUserRepository,
        },
    },
};

use super::http::server::app_state::AppState;

pub struct AppServer<R, C, U, CR, H, AS, RR, RU, RO, K, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RR: RefreshTokenRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    K: KeyStoreRepository,
    UR: UserRoleRepository,
{
    pub realm_repository: R,
    pub client_repository: C,
    pub user_repository: U,
    pub credential_repository: CR,
    pub hasher_repository: H,
    pub auth_session_repository: AS,
    pub refresh_token_repository: RR,
    pub redirect_uri_repository: RU,
    pub role_repository: RO,
    pub keystore_repository: K,
    pub user_role_repository: UR,
}

impl
    AppServer<
        PostgresRealmRepository,
        PostgresClientRepository,
        PostgresUserRepository,
        PostgresCredentialRepository,
        Argon2HasherRepository,
        PostgresAuthSessionRepository,
        PostgresRefreshTokenRepository,
        PostgresRedirectUriRepository,
        PostgresRoleRepository,
        PostgresKeyStoreRepository,
        PostgresUserRoleRepository,
    >
{
    pub async fn new(env: Arc<Env>) -> Result<Self, anyhow::Error> {
        let postgres = Postgres::new(Arc::clone(&env)).await?;
        let realm_repository = PostgresRealmRepository::new(postgres.get_db());
        let client_repository = PostgresClientRepository::new(postgres.get_db());
        let user_repository = PostgresUserRepository::new(postgres.get_db());
        let credential_repository = PostgresCredentialRepository::new(postgres.get_db());
        let hasher_repository = Argon2HasherRepository::new();
        let auth_session_repository = PostgresAuthSessionRepository::new(postgres.get_db());
        let refresh_token_repository = PostgresRefreshTokenRepository::new(postgres.get_db());
        let redirect_uri_repository = PostgresRedirectUriRepository::new(postgres.get_db());
        let role_repository = PostgresRoleRepository::new(postgres.get_db());
        let keystore_repository = PostgresKeyStoreRepository::new(postgres.get_db());
        let user_role_repository = PostgresUserRoleRepository::new(postgres.get_db());

        Ok(Self {
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            auth_session_repository,
            refresh_token_repository,
            redirect_uri_repository,
            role_repository,
            keystore_repository,
            user_role_repository,
        })
    }

    pub fn create_app_state(&self, env: Arc<Env>) -> AppState {
        let realm_service = Arc::new(DefaultRealmService::new(
            self.realm_repository.clone(),
            self.client_repository.clone(),
            self.role_repository.clone(),
            self.user_repository.clone(),
            self.user_role_repository.clone(),
        ));
        let client_service = Arc::new(DefaultClientService::new(
            self.client_repository.clone(),
            self.user_repository.clone(),
            realm_service.clone(),
        ));

        let redirect_uri_service = DefaultRedirectUriService::new(
            self.redirect_uri_repository.clone(),
            realm_service.clone(),
            client_service.clone(),
        );

        let user_service = Arc::new(DefaultUserService::new(
            self.user_repository.clone(),
            self.realm_repository.clone(),
            self.user_role_repository.clone(),
        ));

        let crypto_service = Arc::new(DefaultCryptoService::new(self.hasher_repository.clone()));

        let credential_service = Arc::new(DefaultCredentialService::new(
            self.credential_repository.clone(),
            crypto_service.clone(),
        ));

        let jwt_service = Arc::new(DefaultJwtService::new(
            self.refresh_token_repository.clone(),
            self.keystore_repository.clone(),
            self.realm_repository.clone(),
        ));

        let auth_session_service = Arc::new(DefaultAuthSessionService::new(
            self.auth_session_repository.clone(),
        ));

        let role_service = DefaultRoleService::new(self.role_repository.clone());

        let authentication_service = Arc::new(DefaultAuthenticationService::new(
            realm_service.clone(),
            client_service.clone(),
            credential_service.clone(),
            user_service.clone(),
            jwt_service.clone(),
            auth_session_service.clone(),
        ));

        let user_role_service = DefaultUserRoleService::new(
            self.user_repository.clone(),
            self.role_repository.clone(),
            self.realm_repository.clone(),
            self.user_role_repository.clone(),
        );

        let mediator_config = MediatorConfig {
            env: Arc::clone(&env),
            client_service: client_service.clone(),
            realm_service: realm_service.clone(),
            user_service: user_service.clone(),
            credential_service: credential_service.clone(),
            redirect_uri_service: redirect_uri_service.clone(),
            role_service: role_service.clone(),
            user_role_service: user_role_service.clone(),
            jwt_service: jwt_service.clone(),
        };

        let mediator_service = Arc::new(DefaultMediatorService::new(mediator_config));

        AppState {
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
            mediator_service,
            env: Arc::clone(&env),
        }
    }
}
