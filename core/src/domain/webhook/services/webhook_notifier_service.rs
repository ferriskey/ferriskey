use reqwest::Client;
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

use crate::domain::webhook::{
    entities::{errors::WebhookError, webhook_payload::WebhookPayload},
    ports::{WebhookNotifierService, WebhookRepository},
};

#[derive(Clone)]
pub struct WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    webhook_repository: W,
    http_client: Client,
}

impl<W> WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    pub fn new(webhook_repository: W) -> Self {
        WebhookNotifierServiceImpl {
            webhook_repository,
            http_client: Client::new(),
        }
    }
}

impl<W> WebhookNotifierService for WebhookNotifierServiceImpl<W>
where
    W: WebhookRepository,
{
    async fn notify<T: Send + Sync + Serialize + Clone + 'static>(
        &self,
        realm_id: Uuid,
        payload: WebhookPayload<T>,
    ) -> Result<(), WebhookError> {
        let repo = self.webhook_repository.clone();
        let client = self.http_client.clone();

        tokio::spawn(async move {
            let webhooks = repo
                .fetch_webhooks_by_subscriber(realm_id, payload.event.clone())
                .await;

            match webhooks {
                Ok(webhooks) => {
                    for webhook in webhooks {
                        let response = client
                            .clone()
                            .post(webhook.endpoint)
                            .json(&payload.clone())
                            .send()
                            .await;

                        if let Err(err) = response {
                            error!("Webhook POST failed: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    error!("Failed to fetch webhooks: {:?}", err);
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::webhook::entities::webhook::Webhook;
    use crate::domain::webhook::entities::webhook_subscriber::WebhookSubscriber;
    use crate::domain::webhook::entities::webhook_trigger::WebhookTrigger;
    use crate::domain::webhook::ports::test::get_mock_webhook_repository_with_clone_expectations;
    use chrono::Utc;
    use std::future::ready;

    #[tokio::test]
    async fn notify_returns_ok_and_calls_repo_once_with_subscribers() {
        let mut repo = get_mock_webhook_repository_with_clone_expectations();
        let realm_id = Uuid::new_v4();

        let event = WebhookTrigger::UserCreated;
        // build two dummy webhooks with endpoints that will fail fast; errors are only logged


        repo.expect_clone().returning(move || {
            let mut new_repo = get_mock_webhook_repository_with_clone_expectations();
            let expected_event = WebhookTrigger::UserCreated;
            new_repo.expect_fetch_webhooks_by_subscriber()
                .times(1)
                .returning(move |rid: Uuid, subscriber: WebhookTrigger| {
                    let wh1 = Webhook::new(
                        "http://127.0.0.1:1".to_string(),
                        vec![WebhookSubscriber::new(Uuid::new_v4(), expected_event.clone(), Uuid::new_v4())],
                        Some("wh1".into()),
                        None,
                        None,
                        Utc::now(),
                        Utc::now(),
                    );
                    let wh2 = Webhook::new(
                        "http://[::1]:1".to_string(),
                        vec![WebhookSubscriber::new(Uuid::new_v4(), expected_event.clone(), Uuid::new_v4())],
                        Some("wh2".into()),
                        None,
                        None,
                        Utc::now(),
                        Utc::now(),
                    );
                    assert_eq!(subscriber, expected_event.clone());
                    assert_eq!(rid, realm_id.clone());
                    Box::pin(ready(Ok(vec![wh1.clone(), wh2.clone()])))
                });
            new_repo
        });


        let service = WebhookNotifierServiceImpl { webhook_repository: repo, http_client: Client::new() };

        let payload: WebhookPayload<serde_json::Value> = WebhookPayload::new(event.clone(), Uuid::new_v4(), Some(serde_json::json!({"k":"v"})));

        // Act
        let out = service.notify(realm_id, payload).await;

        // Assert immediate Ok
        assert!(out.is_ok());

        // Allow spawned task to run and hit the mock expectation
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn notify_returns_ok_even_if_repo_errors() {
        let mut repo = get_mock_webhook_repository_with_clone_expectations();
        let realm_id = Uuid::new_v4();
        let event = WebhookTrigger::UserUpdated;

        repo.expect_clone().returning(|| {
            let mut new_repo = get_mock_webhook_repository_with_clone_expectations();
            new_repo.expect_fetch_webhooks_by_subscriber()
                .times(1)
                .returning(move |_rid: Uuid, _subscriber: WebhookTrigger| {
                    Box::pin(ready(Err(WebhookError::InternalServerError)))
                });
            new_repo
        });

        let service = WebhookNotifierServiceImpl { webhook_repository: repo, http_client: Client::new() };
        let payload: WebhookPayload<()> = WebhookPayload::new(event, Uuid::new_v4(), None);

        let out = <WebhookNotifierServiceImpl<_> as WebhookNotifierService>::notify(&service, realm_id, payload).await;
        assert!(out.is_ok());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    #[tokio::test]
    async fn notify_returns_ok_when_no_subscribers() {
        let mut repo = get_mock_webhook_repository_with_clone_expectations();
        let realm_id = Uuid::new_v4();
        let event = WebhookTrigger::UserDeleted;

        repo.expect_clone().returning(|| {
            let mut new_repo = get_mock_webhook_repository_with_clone_expectations();
            new_repo.expect_fetch_webhooks_by_subscriber()
                .times(1)
                .returning(move |_rid: Uuid, _subscriber: WebhookTrigger| {
                    Box::pin(ready(Ok(Vec::new())))
                });
            new_repo
        });

        let service = WebhookNotifierServiceImpl { webhook_repository: repo, http_client: Client::new() };
        let payload: WebhookPayload<()> = WebhookPayload::new(event, Uuid::new_v4(), None);

        let out = <WebhookNotifierServiceImpl<_> as WebhookNotifierService>::notify(&service, realm_id, payload).await;
        assert!(out.is_ok());
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
