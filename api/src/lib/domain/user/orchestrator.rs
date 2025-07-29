use crate::{
    application::auth::Identity,
    domain::{
        client::services::client_service::DefaultClientService,
        realm::services::realm_service::DefaultRealmService,
        role::entities::models::Role,
        user::{
            entities::{error::UserError, model::User},
            services::{
                user_role_service::DefaultUserRoleService, user_service::DefaultUserService,
            },
            use_cases::{
                assign_role_use_case::{AssignRoleUseCase, AssignRoleUseCaseParams},
                bulk_delete_user::{BulkDeleteUserUseCase, BulkDeleteUserUseCaseParams},
                create_user_use_case::{CreateUserUseCase, CreateUserUseCaseParams},
                delete_user_use_case::{DeleteUserUseCase, DeleteUserUseCaseParams},
                get_user_roles_use_case::{GetUserRolesUseCase, GetUserRolesUseCaseParams},
                unassign_role_use_case::{UnassignRoleUseCase, UnassignRoleUseCaseParams},
                update_user_use_case::{UpdateUserUseCase, UpdateUserUseCaseParams},
            },
        },
    },
};

#[derive(Clone)]
pub struct UserOrchestrator {
    assign_role_use_case: AssignRoleUseCase,
    bulk_delete_user_use_case: BulkDeleteUserUseCase,
    create_user_use_case: CreateUserUseCase,
    delete_user_use_case: DeleteUserUseCase,
    get_user_roles_use_case: GetUserRolesUseCase,
    unassign_role_use_case: UnassignRoleUseCase,
    update_user_use_case: UpdateUserUseCase,
}

impl UserOrchestrator {
    pub fn new(
        realm_service: DefaultRealmService,
        user_role_service: DefaultUserRoleService,
        user_service: DefaultUserService,
        client_service: DefaultClientService,
    ) -> Self {
        let assign_role_use_case = AssignRoleUseCase::new(
            realm_service.clone(),
            user_role_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        let bulk_delete_user_use_case = BulkDeleteUserUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        let create_user_use_case = CreateUserUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        let delete_user_use_case = DeleteUserUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        let get_user_roles_use_case = GetUserRolesUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        let unassign_role_use_case = UnassignRoleUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
            user_role_service.clone(),
        );

        let update_user_use_case = UpdateUserUseCase::new(
            realm_service.clone(),
            user_service.clone(),
            client_service.clone(),
        );

        Self {
            assign_role_use_case,
            bulk_delete_user_use_case,
            create_user_use_case,
            delete_user_use_case,
            get_user_roles_use_case,
            unassign_role_use_case,
            update_user_use_case,
        }
    }

    pub async fn assign_role(
        &self,
        identity: Identity,
        params: AssignRoleUseCaseParams,
    ) -> Result<(), UserError> {
        self.assign_role_use_case.execute(identity, params).await
    }

    pub async fn bulk_delete_user(
        &self,
        identity: Identity,
        params: BulkDeleteUserUseCaseParams,
    ) -> Result<u64, UserError> {
        self.bulk_delete_user_use_case
            .execute(identity, params)
            .await
    }

    pub async fn create_user(
        &self,
        identity: Identity,
        params: CreateUserUseCaseParams,
    ) -> Result<User, UserError> {
        self.create_user_use_case.execute(identity, params).await
    }

    pub async fn delete_user(
        &self,
        identity: Identity,
        params: DeleteUserUseCaseParams,
    ) -> Result<u64, UserError> {
        self.delete_user_use_case.execute(identity, params).await
    }

    pub async fn get_user_roles(
        &self,
        identity: Identity,
        params: GetUserRolesUseCaseParams,
    ) -> Result<Vec<Role>, UserError> {
        self.get_user_roles_use_case.execute(identity, params).await
    }

    pub async fn unassign_role(
        &self,
        identity: Identity,
        params: UnassignRoleUseCaseParams,
    ) -> Result<(), UserError> {
        self.unassign_role_use_case.execute(identity, params).await
    }

    pub async fn update_user(
        &self,
        identity: Identity,
        params: UpdateUserUseCaseParams,
    ) -> Result<User, UserError> {
        self.update_user_use_case.execute(identity, params).await
    }
}
