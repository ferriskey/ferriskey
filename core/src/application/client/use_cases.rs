use crate::application::client::use_cases::{
    create_client_use_case::CreateClientUseCase,
    create_redirect_uri_use_case::CreateRedirectUriUseCase,
    create_role_use_case::CreateRoleUseCase, delete_client_use_case::DeleteClientUseCase,
    delete_redirect_uri_use_case::DeleteRedirectUriUseCase,
    get_client_roles_use_case::GetClientRolesUseCase, get_client_use_case::GetClientUseCase,
    get_clients_use_case::GetClientsUseCase, get_redirect_uris_use_case::GetRedirectUrisUseCase,
    update_client_use_case::UpdateClientUseCase,
    update_redirect_uri_use_case::UpdateRedirectUriUseCase,
};
use crate::application::common::services::ServiceBundle;

pub mod create_client_use_case;
pub mod create_redirect_uri_use_case;
pub mod create_role_use_case;
pub mod delete_client_use_case;
pub mod delete_redirect_uri_use_case;
pub mod get_client_roles_use_case;
pub mod get_client_use_case;
pub mod get_clients_use_case;
pub mod get_redirect_uris_use_case;
pub mod update_client_use_case;
pub mod update_redirect_uri_use_case;

pub struct ClientUseCase {
    pub create_client_use_case: CreateClientUseCase,
    pub create_redirect_uri_use_case: CreateRedirectUriUseCase,
    pub create_role_use_case: CreateRoleUseCase,
    pub delete_client_use_case: DeleteClientUseCase,
    pub delete_redirect_uri_use_case: DeleteRedirectUriUseCase,
    pub get_client_roles_use_case: GetClientRolesUseCase,
    pub get_client_use_case: GetClientUseCase,
    pub get_clients_use_case: GetClientsUseCase,
    pub get_redirect_uris_use_case: GetRedirectUrisUseCase,
    pub update_client_use_case: UpdateClientUseCase,
    pub update_redirect_uri_use_case: UpdateRedirectUriUseCase,
}

impl ClientUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            create_client_use_case: CreateClientUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.user_service.clone(),
            ),
            create_redirect_uri_use_case: CreateRedirectUriUseCase::new(
                service_bundle.redirect_uri_service.clone(),
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            create_role_use_case: CreateRoleUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
            ),
            delete_client_use_case: DeleteClientUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            delete_redirect_uri_use_case: DeleteRedirectUriUseCase::new(
                service_bundle.redirect_uri_service.clone(),
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            get_client_roles_use_case: GetClientRolesUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.role_service.clone(),
            ),
            get_client_use_case: GetClientUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            get_clients_use_case: GetClientsUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            get_redirect_uris_use_case: GetRedirectUrisUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.redirect_uri_service.clone(),
            ),
            update_client_use_case: UpdateClientUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
            ),
            update_redirect_uri_use_case: UpdateRedirectUriUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.redirect_uri_service.clone(),
            ),
        }
    }
}
