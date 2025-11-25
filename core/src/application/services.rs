use crate::{
    domain::seawatch::services::SecurityEventServiceImpl,
    infrastructure::{
        client::repositories::client_postgres_repository::PostgresClientRepository,
        realm::repositories::realm_postgres_repository::PostgresRealmRepository,
        seawatch::repositories::security_event_postgres_repository::PostgresSecurityEventRepository,
        user::{
            repositories::user_role_repository::PostgresUserRoleRepository,
            repository::PostgresUserRepository,
        },
    },
};

type RealmRepo = PostgresRealmRepository;
type ClientRepo = PostgresClientRepository;
type UserRepo = PostgresUserRepository;
type UserRoleRepo = PostgresUserRoleRepository;
type SecurityEventRepo = PostgresSecurityEventRepository;

pub struct ApplicationService {
    pub(crate) security_event_service:
        SecurityEventServiceImpl<RealmRepo, UserRepo, ClientRepo, UserRoleRepo, SecurityEventRepo>,
}
