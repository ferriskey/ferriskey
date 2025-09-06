use crate::application::common::services::ServiceBundle;
use crate::application::user::use_cases::get_users_use_case::GetUsersUseCase;
use crate::application::user::use_cases::{
    assign_role_use_case::AssignRoleUseCase, bulk_delete_user::BulkDeleteUserUseCase,
    create_user_use_case::CreateUserUseCase, delete_user_use_case::DeleteUserUseCase,
    get_user_roles_use_case::GetUserRolesUseCase, get_user_use_case::GetUserUseCase,
    unassign_role_use_case::UnassignRoleUseCase, update_user_use_case::UpdateUserUseCase,
};

pub mod assign_role_use_case;
pub mod bulk_delete_user;
pub mod create_user_use_case;
pub mod delete_user_use_case;
pub mod get_user_roles_use_case;
pub mod get_user_use_case;
pub mod get_users_use_case;
pub mod unassign_role_use_case;
pub mod update_user_use_case;

pub struct UserUseCase {
    pub assign_role_use_case: AssignRoleUseCase,
    pub bulk_delete_user_use_case: BulkDeleteUserUseCase,
    pub create_user_use_case: CreateUserUseCase,
    pub delete_user_use_case: DeleteUserUseCase,
    pub get_user_roles_use_case: GetUserRolesUseCase,
    pub unassign_role_use_case: UnassignRoleUseCase,
    pub update_user_use_case: UpdateUserUseCase,
    pub get_user_use_case: GetUserUseCase,
    pub get_users_use_case: GetUsersUseCase,
}

impl UserUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        let assign_role_use_case = AssignRoleUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_role_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let bulk_delete_user_use_case = BulkDeleteUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let create_user_use_case = CreateUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let delete_user_use_case = DeleteUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let get_user_roles_use_case = GetUserRolesUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let unassign_role_use_case = UnassignRoleUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.user_role_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let update_user_use_case = UpdateUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.webhook_notifier_service.clone(),
        );

        let get_user_use_case = GetUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let get_users_use_case = GetUsersUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        Self {
            assign_role_use_case,
            bulk_delete_user_use_case,
            create_user_use_case,
            delete_user_use_case,
            get_user_roles_use_case,
            unassign_role_use_case,
            update_user_use_case,
            get_user_use_case,
            get_users_use_case,
        }
    }
}
