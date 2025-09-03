use uuid::Uuid;

use crate::{
    application::common::FerriskeyService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        webhook::{
            entities::{errors::WebhookError, webhook::Webhook, webhook_trigger::WebhookTrigger},
            ports::{CreateWebhookInput, UpdateWebhookInput, WebhookService},
        },
    },
};

impl WebhookService for FerriskeyService {
    async fn get_webhooks_by_realm(
        &self,
        _identity: Identity,
        _realm_name: String,
    ) -> Result<Vec<Webhook>, CoreError> {
        todo!()
    }

    async fn get_webhooks_by_subscribers(
        &self,
        _identity: Identity,
        _realm_name: String,
        _subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, CoreError> {
        todo!()
    }

    async fn get_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
    ) -> Result<Option<Webhook>, CoreError> {
        todo!()
    }

    async fn create_webhook(
        &self,
        _identity: Identity,
        _realm_name: String,
        _input: CreateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        todo!()
    }

    async fn update_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
        _input: UpdateWebhookInput,
    ) -> Result<Webhook, CoreError> {
        todo!()
    }

    async fn delete_webhook(
        &self,
        _identity: Identity,
        _webhook_id: Uuid,
    ) -> Result<(), CoreError> {
        todo!()
    }

    async fn fetch_by_realm(&self, _realm_id: Uuid) -> Result<Vec<Webhook>, WebhookError> {
        unimplemented!()
    }

    async fn get_by_id(
        &self,
        _webhook_id: Uuid,
        _realm_id: Uuid,
    ) -> Result<Option<Webhook>, WebhookError> {
        unimplemented!()
    }

    async fn fetch_by_subscriber(
        &self,
        _realm_id: Uuid,
        _subscriber: WebhookTrigger,
    ) -> Result<Vec<Webhook>, WebhookError> {
        unimplemented!()
    }

    async fn create(
        &self,
        _realm_id: Uuid,
        _name: Option<String>,
        _description: Option<String>,
        _endpoint: String,
        _subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        unimplemented!()
    }

    async fn update(
        &self,
        _id: Uuid,
        _name: Option<String>,
        _description: Option<String>,
        _endpoint: String,
        _subscribers: Vec<WebhookTrigger>,
    ) -> Result<Webhook, WebhookError> {
        unimplemented!()
    }

    async fn delete(&self, _id: Uuid) -> Result<(), WebhookError> {
        unimplemented!()
    }
}
