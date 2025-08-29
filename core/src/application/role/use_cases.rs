use crate::application::common::services::ServiceBundle;
use crate::application::role::use_cases::get_role_use_case::GetRoleUseCase;
use crate::application::role::use_cases::get_roles_use_case::GetRolesUseCase;
use crate::application::role::use_cases::update_role_permissions_use_case::UpdateRolePermissionsUseCase;
use crate::application::role::use_cases::update_role_use_case::UpdateRoleUseCase;

pub mod delete_role_use_case;
pub mod get_role_use_case;
pub mod get_roles_use_case;
pub mod update_role_permissions_use_case;
pub mod update_role_use_case;

pub struct RoleUseCase {
    pub get_role_use_case: GetRoleUseCase,
    pub get_roles_use_case: GetRolesUseCase,
    pub update_role_permissions_use_case: UpdateRolePermissionsUseCase,
    pub update_role_use_case: UpdateRoleUseCase,
}

impl RoleUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            get_role_use_case: GetRoleUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
            ),
            get_roles_use_case: GetRolesUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
            ),
            update_role_permissions_use_case: UpdateRolePermissionsUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
                service_bundle.webhook_notifier_service.clone(),
            ),
            update_role_use_case: UpdateRoleUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
                service_bundle.webhook_notifier_service.clone(),
            ),
        }
    }
}
