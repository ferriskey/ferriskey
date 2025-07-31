use crate::application::authentication::use_cases::authenticate_use_case::AuthenticateUseCase;
use crate::application::authentication::use_cases::exchange_token_use_case::ExchangeTokenUseCase;
use crate::application::authentication::use_cases::get_certs_use_case::GetCertsUseCase;
use crate::application::client::use_cases::create_client_use_case::CreateClientUseCase;
use crate::application::client::use_cases::create_redirect_uri_use_case::CreateRedirectUriUseCase;
use crate::application::client::use_cases::create_role_use_case::CreateRoleUseCase;
use crate::application::client::use_cases::delete_client_use_case::DeleteClientUseCase;
use crate::application::realm::use_cases::create_realm_use_case::CreateRealmUseCase;
use crate::application::role::use_cases::get_roles_use_case::GetRolesUseCase;
use crate::application::role::use_cases::update_role_permissions_use_case::UpdateRolePermissionsUseCase;
use crate::application::role::use_cases::update_role_use_case::UpdateRoleUseCase;
use crate::application::user::use_cases::assign_role_use_case::AssignRoleUseCase;
use crate::application::user::use_cases::bulk_delete_user::BulkDeleteUserUseCase;
use crate::application::user::use_cases::create_user_use_case::CreateUserUseCase;
use crate::application::user::use_cases::delete_user_use_case::DeleteUserUseCase;
use crate::application::user::use_cases::get_user_roles_use_case::GetUserRolesUseCase;
use crate::application::user::use_cases::unassign_role_use_case::UnassignRoleUseCase;
use crate::application::user::use_cases::update_user_use_case::UpdateUserUseCase;
use crate::infrastructure::common::factories::service_factory::ServiceBundle;

#[derive(Clone)]
pub struct UseCaseFactory;

#[derive(Clone)]
pub struct UseCaseBundle {
    // Auth (use-cases)
    pub exchange_token_use_case: ExchangeTokenUseCase,
    pub get_certs_use_case: GetCertsUseCase,
    pub authenticate_use_case: AuthenticateUseCase,

    // Realm (use-cases
    pub create_realm_use_case: CreateRealmUseCase,

    // Client (use-cases)
    pub create_client_use_case: CreateClientUseCase,
    pub create_redirect_uri_use_case: CreateRedirectUriUseCase,
    pub create_role_use_case: CreateRoleUseCase,
    pub delete_client_use_case: DeleteClientUseCase,

    // User (use-cases)
    pub assign_role_use_case: AssignRoleUseCase,
    pub bulk_delete_user_use_case: BulkDeleteUserUseCase,
    pub create_user_use_case: CreateUserUseCase,
    pub delete_user_use_case: DeleteUserUseCase,
    pub get_user_roles_use_case: GetUserRolesUseCase,
    pub unassign_role_use_case: UnassignRoleUseCase,
    pub update_user_use_case: UpdateUserUseCase,

    // Role (use-cases)
    pub get_roles_use_case: GetRolesUseCase,
    pub update_role_use_case: UpdateRoleUseCase,
    pub update_role_permissions_use_case: UpdateRolePermissionsUseCase,
}

impl UseCaseFactory {
    pub fn new(service_bundle: ServiceBundle) -> UseCaseBundle {
        // Auth (use-cases)

        let exchange_token_use_case = ExchangeTokenUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.grant_type_service.clone(),
        );
        let get_certs_use_case = GetCertsUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.jwt_service.clone(),
        );
        let authenticate_use_case = AuthenticateUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.jwt_service.clone(),
            service_bundle.auth_session_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.credential_service.clone(),
            service_bundle.user_service.clone(),
        );

        // Realm (use-cases)
        let create_realm_use_case = CreateRealmUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        // Client (use-cases)
        let create_client_use_case = CreateClientUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.client_service.clone(),
        );

        let create_redirect_uri_use_case =
            CreateRedirectUriUseCase::new(service_bundle.redirect_uri_service.clone());

        let create_role_use_case = CreateRoleUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.role_service.clone(),
        );

        let delete_client_use_case = DeleteClientUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        // User (use-cases)
        let assign_role_use_case = AssignRoleUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_role_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let bulk_delete_user_use_case = BulkDeleteUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let create_user_use_case = CreateUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let delete_user_use_case = DeleteUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
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
        );

        let update_user_use_case = UpdateUserUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        // Role (use-cases)
        let get_roles_use_case = GetRolesUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.role_service.clone(),
        );

        let update_role_use_case = UpdateRoleUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.role_service.clone(),
        );

        let update_role_permissions_use_case = UpdateRolePermissionsUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
            service_bundle.role_service.clone(),
        );

        UseCaseBundle {
            // Auth (use-cases)
            exchange_token_use_case,
            get_certs_use_case,
            authenticate_use_case,

            // Realm (use-cases)
            create_realm_use_case,

            // Client (use-cases)
            create_client_use_case,
            create_redirect_uri_use_case,
            create_role_use_case,
            delete_client_use_case,

            // User (use-cases)
            assign_role_use_case,
            bulk_delete_user_use_case,
            create_user_use_case,
            delete_user_use_case,
            get_user_roles_use_case,
            unassign_role_use_case,
            update_user_use_case,

            // Role (use-cases)
            get_roles_use_case,
            update_role_use_case,
            update_role_permissions_use_case,
        }
    }
}
