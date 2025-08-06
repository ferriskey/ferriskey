use crate::application::authentication::use_cases::authenticate_use_case::AuthenticateUseCase;
use crate::application::authentication::use_cases::exchange_token_use_case::ExchangeTokenUseCase;
use crate::application::authentication::use_cases::get_certs_use_case::GetCertsUseCase;
use crate::application::client::use_cases::ClientUseCase;
use crate::application::client::use_cases::create_client_use_case::CreateClientUseCase;
use crate::application::client::use_cases::create_redirect_uri_use_case::CreateRedirectUriUseCase;
use crate::application::client::use_cases::create_role_use_case::CreateRoleUseCase;
use crate::application::client::use_cases::delete_client_use_case::DeleteClientUseCase;
use crate::application::client::use_cases::delete_redirect_uri_use_case::DeleteRedirectUriUseCase;
use crate::application::client::use_cases::get_client_roles_use_case::GetClientRolesUseCase;
use crate::application::client::use_cases::get_client_use_case::GetClientUseCase;
use crate::application::client::use_cases::get_clients_use_case::GetClientsUseCase;
use crate::application::client::use_cases::get_redirect_uris_use_case::GetRedirectUrisUseCase;
use crate::application::client::use_cases::update_client_use_case::UpdateClientUseCase;
use crate::application::client::use_cases::update_redirect_uri_use_case::UpdateRedirectUriUseCase;
use crate::application::health::use_cases::HealthCheckUseCase;
use crate::application::realm::use_cases::create_realm_use_case::CreateRealmUseCase;
use crate::application::realm::use_cases::delete_realm_use_case::DeleteRealmUseCase;
use crate::application::realm::use_cases::get_realm_use_case::GetRealmUseCase;
use crate::application::realm::use_cases::get_user_realms_use_case::GetUserRealmsUseCase;
use crate::application::realm::use_cases::update_realm_settings_use_case::UpdateRealmSettingsUseCase;
use crate::application::realm::use_cases::update_realm_use_case::UpdateRealmUseCase;
use crate::application::role::use_cases::RoleUseCase;
use crate::application::role::use_cases::get_role_use_case::GetRoleUseCase;
use crate::application::role::use_cases::get_roles_use_case::GetRolesUseCase;
use crate::application::role::use_cases::update_role_permissions_use_case::UpdateRolePermissionsUseCase;
use crate::application::role::use_cases::update_role_use_case::UpdateRoleUseCase;
use crate::application::user::use_cases::UserUseCase;
use crate::application::user::use_cases::assign_role_use_case::AssignRoleUseCase;
use crate::application::user::use_cases::bulk_delete_user::BulkDeleteUserUseCase;
use crate::application::user::use_cases::create_user_use_case::CreateUserUseCase;
use crate::application::user::use_cases::delete_credential_use_case::DeleteCredentialUseCase;
use crate::application::user::use_cases::delete_user_use_case::DeleteUserUseCase;
use crate::application::user::use_cases::get_credentials_use_case::GetCredentialsUseCase;
use crate::application::user::use_cases::get_user_roles_use_case::GetUserRolesUseCase;
use crate::application::user::use_cases::get_user_use_case::GetUserUseCase;
use crate::application::user::use_cases::get_users_use_case::GetUsersUseCase;
use crate::application::user::use_cases::reset_password_use_case::ResetPasswordUseCase;
use crate::application::user::use_cases::unassign_role_use_case::UnassignRoleUseCase;
use crate::application::user::use_cases::update_user_use_case::UpdateUserUseCase;
use crate::infrastructure::common::factories::service_factory::ServiceBundle;

#[derive(Clone)]
pub struct UseCaseBundle {
    // Auth (use-cases)
    pub exchange_token_use_case: ExchangeTokenUseCase,
    pub get_certs_use_case: GetCertsUseCase,
    pub authenticate_use_case: AuthenticateUseCase,

    // Realm (use-cases
    pub create_realm_use_case: CreateRealmUseCase,
    pub delete_realm_use_case: DeleteRealmUseCase,
    pub get_realm_use_case: GetRealmUseCase,
    pub get_user_realms_use_case: GetUserRealmsUseCase,
    pub update_realm_use_case: UpdateRealmUseCase,
    pub update_realm_settings_use_case: UpdateRealmSettingsUseCase,

    // Client (use-cases)
    pub create_client_use_case: CreateClientUseCase,
    pub create_redirect_uri_use_case: CreateRedirectUriUseCase,
    pub create_role_use_case: CreateRoleUseCase,
    pub delete_client_use_case: DeleteClientUseCase,
    pub delete_redirect_uri_use_case: DeleteRedirectUriUseCase,
    pub get_client_use_case: GetClientUseCase,
    pub get_client_roles_use_case: GetClientRolesUseCase,
    pub get_clients_use_case: GetClientsUseCase,
    pub get_redirect_uris_use_case: GetRedirectUrisUseCase,
    pub update_client_use_case: UpdateClientUseCase,
    pub update_redirect_uri_use_case: UpdateRedirectUriUseCase,

    // User (use-cases)
    pub assign_role_use_case: AssignRoleUseCase,
    pub bulk_delete_user_use_case: BulkDeleteUserUseCase,
    pub create_user_use_case: CreateUserUseCase,
    pub delete_user_use_case: DeleteUserUseCase,
    pub get_user_roles_use_case: GetUserRolesUseCase,
    pub unassign_role_use_case: UnassignRoleUseCase,
    pub update_user_use_case: UpdateUserUseCase,
    pub delete_credential_use_case: DeleteCredentialUseCase,
    pub get_credentials_use_case: GetCredentialsUseCase,
    pub get_user_use_case: GetUserUseCase,
    pub get_users_use_case: GetUsersUseCase,
    pub reset_password_use_case: ResetPasswordUseCase,

    // Role (use-cases)
    pub get_roles_use_case: GetRolesUseCase,
    pub get_role_use_case: GetRoleUseCase,
    pub update_role_use_case: UpdateRoleUseCase,
    pub update_role_permissions_use_case: UpdateRolePermissionsUseCase,

    pub health_check_use_case: HealthCheckUseCase,
}

impl UseCaseBundle {
    pub fn new(service_bundle: ServiceBundle) -> Self {
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

        let delete_realm_use_case = DeleteRealmUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let get_realm_use_case = GetRealmUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let get_user_realms_use_case =
            GetUserRealmsUseCase::new(service_bundle.user_service.clone());

        let update_realm_use_case = UpdateRealmUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        let update_realm_settings_use_case = UpdateRealmSettingsUseCase::new(
            service_bundle.realm_service.clone(),
            service_bundle.user_service.clone(),
            service_bundle.client_service.clone(),
        );

        // Client (use-cases)
        let client_use_case = ClientUseCase::new(service_bundle.clone());

        // User (use-cases)
        let user_use_case = UserUseCase::new(service_bundle.clone());

        // Role (use-cases)
        let role_use_case = RoleUseCase::new(service_bundle.clone());

        let health_check_use_case =
            HealthCheckUseCase::new(service_bundle.health_check_service.clone());

        Self {
            // Auth (use-cases)
            exchange_token_use_case,
            get_certs_use_case,
            authenticate_use_case,

            // Realm (use-cases)
            create_realm_use_case,
            delete_realm_use_case,
            get_realm_use_case,
            get_user_realms_use_case,
            update_realm_use_case,
            update_realm_settings_use_case,

            // Client (use-cases)
            create_client_use_case: client_use_case.create_client_use_case,
            create_redirect_uri_use_case: client_use_case.create_redirect_uri_use_case,
            create_role_use_case: client_use_case.create_role_use_case,
            delete_client_use_case: client_use_case.delete_client_use_case,
            delete_redirect_uri_use_case: client_use_case.delete_redirect_uri_use_case,
            get_client_use_case: client_use_case.get_client_use_case,
            get_client_roles_use_case: client_use_case.get_client_roles_use_case,
            get_clients_use_case: client_use_case.get_clients_use_case,
            get_redirect_uris_use_case: client_use_case.get_redirect_uris_use_case,
            update_client_use_case: client_use_case.update_client_use_case,
            update_redirect_uri_use_case: client_use_case.update_redirect_uri_use_case,

            // User (use-cases)
            assign_role_use_case: user_use_case.assign_role_use_case,
            bulk_delete_user_use_case: user_use_case.bulk_delete_user_use_case,
            create_user_use_case: user_use_case.create_user_use_case,
            delete_user_use_case: user_use_case.delete_user_use_case,
            get_user_roles_use_case: user_use_case.get_user_roles_use_case,
            unassign_role_use_case: user_use_case.unassign_role_use_case,
            update_user_use_case: user_use_case.update_user_use_case,
            delete_credential_use_case: user_use_case.delete_credential_use_case,
            get_credentials_use_case: user_use_case.get_credentials_use_case,
            get_user_use_case: user_use_case.get_user_use_case,
            get_users_use_case: user_use_case.get_users_use_case,
            reset_password_use_case: user_use_case.reset_password_use_case,

            // Role (use-cases)
            get_roles_use_case: role_use_case.get_roles_use_case,
            get_role_use_case: role_use_case.get_role_use_case,
            update_role_use_case: role_use_case.update_role_use_case,
            update_role_permissions_use_case: role_use_case.update_role_permissions_use_case,

            health_check_use_case,
        }
    }
}
