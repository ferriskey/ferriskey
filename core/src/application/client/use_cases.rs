use crate::application::client::use_cases::{
    delete_redirect_uri_use_case::DeleteRedirectUriUseCase, get_client_use_case::GetClientUseCase,
    get_clients_use_case::GetClientsUseCase, get_redirect_uris_use_case::GetRedirectUrisUseCase,
    update_client_use_case::UpdateClientUseCase,
    update_redirect_uri_use_case::UpdateRedirectUriUseCase,
};
use crate::application::common::services::ServiceBundle;

pub mod delete_redirect_uri_use_case;
pub mod get_client_use_case;
pub mod get_clients_use_case;
pub mod get_redirect_uris_use_case;
pub mod update_client_use_case;
pub mod update_redirect_uri_use_case;

pub struct ClientUseCase {
    pub delete_redirect_uri_use_case: DeleteRedirectUriUseCase,
    pub get_client_use_case: GetClientUseCase,
    pub get_clients_use_case: GetClientsUseCase,
    pub get_redirect_uris_use_case: GetRedirectUrisUseCase,
    pub update_client_use_case: UpdateClientUseCase,
    pub update_redirect_uri_use_case: UpdateRedirectUriUseCase,
}

impl ClientUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            delete_redirect_uri_use_case: DeleteRedirectUriUseCase::new(
                service_bundle.redirect_uri_service.clone(),
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.webhook_notifier_service.clone(),
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
                service_bundle.webhook_notifier_service.clone(),
            ),
            update_redirect_uri_use_case: UpdateRedirectUriUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.redirect_uri_service.clone(),
                service_bundle.webhook_notifier_service.clone(),
            ),
        }
    }
}
