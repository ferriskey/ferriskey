use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        webhook::{
            entities::webhook::Webhook,
            ports::{
                CreateWebhookInput, DeleteWebhookInput, GetWebhookInput,
                GetWebhookSubscribersInput, GetWebhooksInput, UpdateWebhookInput, WebhookService,
            },
        },
    },
};

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

    async fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        input: GetWebhooksInput,
    ) -> Result<Vec<Webhook>, CoreError> {
        self.webhook_service
            .get_webhooks_by_realm(identity, input)
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
}
