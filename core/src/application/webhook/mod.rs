use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::{
            entities::app_errors::CoreError,
            pagination::{Page, PageRequest},
        },
        webhook::{
            entities::webhook::{Webhook, WebhookFilter, WebhookSortField},
            entities::webhook_delivery::{
                WebhookDelivery, WebhookDeliveryFilter, WebhookDeliverySortField,
            },
            ports::{
                CreateWebhookInput, DeleteWebhookInput, GetWebhookDeliveryInput, GetWebhookInput,
                GetWebhookSubscribersInput, RetryWebhookDeliveryInput, RotateWebhookSecretInput,
                UpdateWebhookInput, WebhookService,
            },
        },
    },
};
use uuid::Uuid;

impl WebhookService for ApplicationService {
    async fn create_webhook(
        &self,
        identity: Identity,
        input: CreateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        self.webhook_service.create_webhook(identity, input).await
    }

    async fn delete_webhook(
        &self,
        identity: Identity,
        input: DeleteWebhookInput,
    ) -> Result<(), CoreError> {
        self.webhook_service.delete_webhook(identity, input).await
    }

    async fn get_webhook(
        &self,
        identity: Identity,
        input: GetWebhookInput,
    ) -> Result<Option<Webhook>, CoreError> {
        self.webhook_service.get_webhook(identity, input).await
    }

    async fn list_webhooks(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<WebhookFilter, WebhookSortField>,
    ) -> Result<Page<Webhook>, CoreError> {
        self.webhook_service
            .list_webhooks(identity, realm_name, request)
            .await
    }

    async fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        input: GetWebhookSubscribersInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        self.webhook_service
            .get_webhooks_by_subscribers(identity, input)
            .await
    }

    async fn update_webhook(
        &self,
        identity: Identity,
        input: UpdateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        self.webhook_service.update_webhook(identity, input).await
    }

    async fn list_webhook_deliveries(
        &self,
        identity: Identity,
        realm_name: String,
        webhook_id: Uuid,
        request: PageRequest<WebhookDeliveryFilter, WebhookDeliverySortField>,
    ) -> Result<Page<WebhookDelivery>, CoreError> {
        self.webhook_service
            .list_webhook_deliveries(identity, realm_name, webhook_id, request)
            .await
    }

    async fn get_webhook_delivery(
        &self,
        identity: Identity,
        input: GetWebhookDeliveryInput,
    ) -> Result<WebhookDelivery, CoreError> {
        self.webhook_service
            .get_webhook_delivery(identity, input)
            .await
    }

    async fn retry_webhook_delivery(
        &self,
        identity: Identity,
        input: RetryWebhookDeliveryInput,
    ) -> Result<(), CoreError> {
        self.webhook_service
            .retry_webhook_delivery(identity, input)
            .await
    }

    async fn rotate_webhook_secret(
        &self,
        identity: Identity,
        input: RotateWebhookSecretInput,
    ) -> Result<String, CoreError> {
        self.webhook_service
            .rotate_webhook_secret(identity, input)
            .await
    }
}
