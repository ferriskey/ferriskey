use crate::domain::{
    authentication::ports::AuthSessionRepository,
    client::ports::{ClientRepository, RedirectUriRepository},
    common::policies::FerriskeyPolicy,
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::KeyStoreRepository,
    realm::ports::RealmRepository,
    role::ports::RoleRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::ports::{WebhookNotifierRepository, WebhookRepository},
};

#[derive(Clone)]
pub struct Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN>
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
    WN: WebhookNotifierRepository,
{
    pub(crate) realm_repository: R,
    pub(crate) client_repository: C,
    pub(crate) user_repository: U,
    pub(crate) credential_repository: CR,
    pub(crate) hasher_repository: H,
    pub(crate) auth_session_repository: AS,
    pub(crate) redirect_uri_repository: RU,
    pub(crate) role_repository: RO,
    pub(crate) keystore_repository: KS,
    pub(crate) user_role_repository: UR,
    pub(crate) user_required_action_repository: URA,
    pub(crate) health_check_repository: HC,
    pub(crate) webhook_repository: W,
    pub(crate) webhook_notifier_repository: WN,

    pub(crate) policy: FerriskeyPolicy<U, C, UR>,
}

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN>
    Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, WN>
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
    WN: WebhookNotifierRepository,
{
    pub fn new(
        realm_repository: R,
        client_repository: C,
        user_repository: U,
        credential_repository: CR,
        hasher_repository: H,
        auth_session_repository: AS,
        redirect_uri_repository: RU,
        role_repository: RO,
        keystore_repository: KS,
        user_role_repository: UR,
        user_required_action_repository: URA,
        health_check_repository: HC,
        webhook_repository: W,
        webhook_notifier_repository: WN,
    ) -> Self {
        let policy = FerriskeyPolicy::new(
            user_repository.clone(),
            client_repository.clone(),
            user_role_repository.clone(),
        );

        Service {
            realm_repository,
            client_repository,
            user_repository,
            credential_repository,
            hasher_repository,
            auth_session_repository,
            redirect_uri_repository,
            role_repository,
            keystore_repository,
            user_role_repository,
            user_required_action_repository,
            health_check_repository,
            webhook_repository,
            webhook_notifier_repository,

            policy,
        }
    }
}
