use crate::{
    domain::{realm::services::RealmServiceImpl, user::services::user_service::UserServiceImpl},
    infrastructure::{
        repositories::{
            client_repository::PostgresClientRepository, realm_repository::PostgresRealmRepository,
            role_repository::PostgresRoleRepository,
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
