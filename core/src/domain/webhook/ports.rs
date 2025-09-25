use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    webhook::entities::{
        errors::WebhookError, webhook::Webhook, webhook_payload::WebhookPayload,
        webhook_trigger::WebhookTrigger,
    },
};

pub trait WebhookService: Clone + Send + Sync {
    fn get_webhooks_by_realm(
        &self,
        identity: Identity,
        input: GetWebhooksInput,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhooks_by_subscribers(
        &self,
        identity: Identity,
        input: GetWebhookSubscribersInput,
    ) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;

    fn get_webhook(
        &self,
        identity: Identity,
        input: GetWebhookInput,
    ) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;

    fn create_webhook(
        &self,
        identity: Identity,
        input: CreateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn update_webhook(
        &self,
        identity: Identity,
        input: UpdateWebhookInput,
    ) -> impl Future<Output = Result<Webhook, CoreError>> + Send;

    fn delete_webhook(
        &self,
        identity: Identity,
        input: DeleteWebhookInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait WebhookRepository: Clone + Send + Sync + 'static {
    fn fetch_webhooks_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn fetch_webhooks_by_subscriber(
        &self,
        realm_id: Uuid,
        subscriber: WebhookTrigger,
    ) -> impl Future<Output = Result<Vec<Webhook>, WebhookError>> + Send;

    fn get_webhook_by_id(
        &self,
        webhook_id: Uuid,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<Webhook>, WebhookError>> + Send;

    fn create_webhook(
        &self,
        realm_id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn update_webhook(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        endpoint: String,
        subscribers: Vec<WebhookTrigger>,
    ) -> impl Future<Output = Result<Webhook, WebhookError>> + Send;

    fn delete_webhook(&self, id: Uuid) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookNotifierRepository: Clone + Send + Sync + 'static {
    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        webhooks: Vec<Webhook>,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait WebhookNotifierService: Clone + Send + Sync {
    fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> impl Future<Output = Result<(), WebhookError>> + Send;
}

pub trait WebhookPolicy: Clone + Send + Sync + 'static {
    fn can_create_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_update_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_delete_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_view_webhook(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetWebhooksInput {
    pub realm_name: String,
}

pub struct GetWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

pub struct GetWebhookSubscribersInput {
    pub realm_name: String,
    pub subscriber: WebhookTrigger,
}

pub struct CreateWebhookInput {
    pub realm_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct UpdateWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub endpoint: String,
    pub subscribers: Vec<WebhookTrigger>,
}

pub struct DeleteWebhookInput {
    pub realm_name: String,
    pub webhook_id: Uuid,
}

#[cfg(test)]
pub mod test {
    use super::*;
    use mockall::mock;

    mock! {
        pub WebhookService {}
        impl Clone for WebhookService { fn clone(&self) -> Self; }
        impl WebhookService for WebhookService {
            fn get_webhooks_by_realm(&self, identity: Identity, input: GetWebhooksInput) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;
            fn get_webhooks_by_subscribers(&self, identity: Identity, input: GetWebhookSubscribersInput) -> impl Future<Output = Result<Vec<Webhook>, CoreError>> + Send;
            fn get_webhook(&self, identity: Identity, input: GetWebhookInput) -> impl Future<Output = Result<Option<Webhook>, CoreError>> + Send;
            fn create_webhook(&self, identity: Identity, input: CreateWebhookInput) -> impl Future<Output = Result<Webhook, CoreError>> + Send;
            fn update_webhook(&self, identity: Identity, input: UpdateWebhookInput) -> impl Future<Output = Result<Webhook, CoreError>> + Send;
            fn delete_webhook(&self, identity: Identity, input: DeleteWebhookInput) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }
    pub fn get_mock_webhook_service_with_clone_expectations() -> MockWebhookService {
        let mut mock = MockWebhookService::new();
        mock.expect_clone()
            .returning(|| get_mock_webhook_service_with_clone_expectations());
        mock
    }

    mock! {
        pub WebhookRepository {}
        impl Clone for WebhookRepository { fn clone(&self) -> Self; }
        impl WebhookRepository for WebhookRepository {
            fn fetch_webhooks_by_realm(&self, realm_id: Uuid) -> impl Future<Output = Result<Vec<Webhook>, crate::domain::webhook::entities::errors::WebhookError>> + Send;
            fn fetch_webhooks_by_subscriber(&self, realm_id: Uuid, subscriber: crate::domain::webhook::entities::webhook_trigger::WebhookTrigger) -> impl Future<Output = Result<Vec<Webhook>, crate::domain::webhook::entities::errors::WebhookError>> + Send;
            fn get_webhook_by_id(&self, webhook_id: Uuid, realm_id: Uuid) -> impl Future<Output = Result<Option<Webhook>, crate::domain::webhook::entities::errors::WebhookError>> + Send;
            fn create_webhook(&self, realm_id: Uuid, name: Option<String>, description: Option<String>, endpoint: String, subscribers: Vec<crate::domain::webhook::entities::webhook_trigger::WebhookTrigger>) -> impl Future<Output = Result<Webhook, crate::domain::webhook::entities::errors::WebhookError>> + Send;
            fn update_webhook(&self, id: Uuid, name: Option<String>, description: Option<String>, endpoint: String, subscribers: Vec<crate::domain::webhook::entities::webhook_trigger::WebhookTrigger>) -> impl Future<Output = Result<Webhook, crate::domain::webhook::entities::errors::WebhookError>> + Send;
            fn delete_webhook(&self, id: Uuid) -> impl Future<Output = Result<(), crate::domain::webhook::entities::errors::WebhookError>> + Send;
        }
    }
    pub fn get_mock_webhook_repository_with_clone_expectations() -> MockWebhookRepository {
        let mut mock = MockWebhookRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_webhook_repository_with_clone_expectations());
        mock
    }

    mock! {
        pub WebhookNotifierRepository {}
        impl Clone for WebhookNotifierRepository { fn clone(&self) -> Self; }
        impl WebhookNotifierRepository for WebhookNotifierRepository {
            fn notify<T: Send + Sync + serde::Serialize + Clone + 'static>(&self, webhooks: Vec<Webhook>, payload: crate::domain::webhook::entities::webhook_payload::WebhookPayload<T>) -> impl Future<Output = Result<(), CoreError>> + Send;
        }
    }
    pub fn get_mock_webhook_notifier_repository_with_clone_expectations() -> MockWebhookNotifierRepository {
        let mut mock = MockWebhookNotifierRepository::new();
        mock.expect_clone()
            .returning(|| get_mock_webhook_notifier_repository_with_clone_expectations());
        mock
    }

    mock! {
        pub WebhookNotifierService {}
        impl Clone for WebhookNotifierService { fn clone(&self) -> Self; }
        impl WebhookNotifierService for WebhookNotifierService {
            fn notify<T: Send + Sync + serde::Serialize + Clone + 'static>(&self, realm_id: Uuid, payload: crate::domain::webhook::entities::webhook_payload::WebhookPayload<T>) -> impl Future<Output = Result<(), crate::domain::webhook::entities::errors::WebhookError>> + Send;
        }
    }
    pub fn get_mock_webhook_notifier_service_with_clone_expectations() -> MockWebhookNotifierService {
        let mut mock = MockWebhookNotifierService::new();
        mock.expect_clone()
            .returning(|| get_mock_webhook_notifier_service_with_clone_expectations());
        mock
    }

    mock! {
        pub WebhookPolicy {}
        impl Clone for WebhookPolicy { fn clone(&self) -> Self; }
        impl WebhookPolicy for WebhookPolicy {
            fn can_create_webhook(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_update_webhook(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_delete_webhook(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
            fn can_view_webhook(&self, identity: Identity, target_realm: Realm) -> impl Future<Output = Result<bool, CoreError>> + Send;
        }
    }
    pub fn get_mock_webhook_policy_with_clone_expectations() -> MockWebhookPolicy {
        let mut mock = MockWebhookPolicy::new();
        mock.expect_clone()
            .returning(|| get_mock_webhook_policy_with_clone_expectations());
        mock
    }
}
