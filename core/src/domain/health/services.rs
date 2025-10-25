use crate::domain::{
    authentication::ports::AuthSessionRepository,
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::{
        entities::DatabaseHealthStatus,
        ports::{HealthCheckRepository, HealthCheckService},
    },
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::WebhookRepository,
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC> HealthCheckService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
    async fn readness(&self) -> Result<DatabaseHealthStatus, CoreError> {
        self.health_check_repository.readness().await
    }

    async fn health(&self) -> Result<u64, CoreError> {
        self.health_check_repository.health().await
    }
}
