use crate::{
    domain::{
        client::services::ClientServiceImpl, credential::services::CredentialServiceImpl,
        realm::services::RealmServiceImpl, seawatch::services::SecurityEventServiceImpl,
    },
    infrastructure::{
        client::repositories::{
            client_postgres_repository::PostgresClientRepository,
            redirect_uri_postgres_repository::PostgresRedirectUriRepository,
        },
        realm::repositories::realm_postgres_repository::PostgresRealmRepository,
        repositories::credential_repository::PostgresCredentialRepository,
        role::repositories::role_postgres_repository::PostgresRoleRepository,
        seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository,
        user::{
            repositories::user_role_repository::PostgresUserRoleRepository,
            repository::PostgresUserRepository,
        },
        webhook::repositories::webhook_repository::PostgresWebhookRepository,
    },
};

type RealmRepo = PostgresRealmRepository;
type ClientRepo = PostgresClientRepository;
type UserRepo = PostgresUserRepository;
type UserRoleRepo = PostgresUserRoleRepository;
type SecurityEventRepo = PostgresSecurityEventRepository;
type CredentialRepo = PostgresCredentialRepository;
type WebhookRepo = PostgresWebhookRepository;
type RedirectUriRepo = PostgresRedirectUriRepository;
type RoleRepo = PostgresRoleRepository;

pub struct ApplicationService {
    pub(crate) security_event_service:
        SecurityEventServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SecurityEventRepo>,
    pub(crate) credential_service:
        CredentialServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, CredentialRepo>,
    pub(crate) client_service: ClientServiceImpl<
        RealmRepo,
        UserRepo,
        ClientRepo,
        UserRoleRepo,
        WebhookRepo,
        RedirectUriRepo,
        RoleRepo,
        SecurityEventRepo,
    >,
    pub(crate) realm_service:
        RealmServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, RoleRepo, WebhookRepo>,
}
