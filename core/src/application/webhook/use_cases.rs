use crate::application::{
    common::services::ServiceBundle,
    webhook::use_cases::{
        create_webhook_use_case::CreateWebhookUseCase,
        delete_webhook_use_case::DeleteWebhookUseCase,
        fetch_realm_webhooks_use_case::FetchRealmWebhooksUseCase,
        get_webhook_use_case::GetWebhookUseCase, update_webhook_use_case::UpdateWebhookUseCase,
    },
};

pub mod create_webhook_use_case;
pub mod delete_webhook_use_case;
pub mod fetch_realm_webhooks_use_case;
pub mod get_webhook_use_case;
pub mod update_webhook_use_case;

pub struct WebhookUseCase {
    pub fetch_realm_webhooks_use_case: FetchRealmWebhooksUseCase,
    pub create_webhook_use_case: CreateWebhookUseCase,
    pub get_webhook_use_case: GetWebhookUseCase,
    pub update_webhook_use_case: UpdateWebhookUseCase,
    pub delete_webhook_use_case: DeleteWebhookUseCase,
}

impl WebhookUseCase {
    pub fn new(service_bundle: &ServiceBundle) -> Self {
        Self {
            fetch_realm_webhooks_use_case: FetchRealmWebhooksUseCase::new(
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.realm_service.clone(),
                service_bundle.webhook_service.clone(),
            ),
            create_webhook_use_case: CreateWebhookUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.webhook_service.clone(),
            ),
            get_webhook_use_case: GetWebhookUseCase::new(
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.realm_service.clone(),
                service_bundle.webhook_service.clone(),
            ),
            update_webhook_use_case: UpdateWebhookUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.webhook_service.clone(),
            ),
            delete_webhook_use_case: DeleteWebhookUseCase::new(
                service_bundle.realm_service.clone(),
                service_bundle.user_service.clone(),
                service_bundle.client_service.clone(),
                service_bundle.webhook_service.clone(),
            ),
        }
    }
}
