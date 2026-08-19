use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::entities::webhook_trigger::WebhookTrigger;

#[derive(Debug, Clone, Serialize)]
pub struct WebhookPayload<T>
where
    T: Serialize + Send + Sync + Clone + 'static,
{
    pub event: WebhookTrigger,
    pub timestamp: String,
    pub resource_id: Uuid,
    pub data: Option<T>,
}

impl<T> WebhookPayload<T>
where
    T: Serialize + Send + Sync + Clone + 'static,
{
    pub fn new(event: WebhookTrigger, resource_id: Uuid, data: Option<T>) -> Self {
        WebhookPayload {
            event,
            timestamp: Utc::now().to_rfc3339(),
            resource_id,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::entities::webhook::Webhook;

    fn webhook_with_credential() -> Webhook {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer top-secret".to_string());

        Webhook::new(
            "https://example.com/hook".to_string(),
            headers,
            Vec::new(),
            None,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    /// This is the path the finding named directly: `create_webhook`/`update_webhook`/
    /// `delete_webhook` notify subscribers with the complete `Webhook` as `data`, so webhook A's
    /// `Authorization` header would otherwise be POSTed in clear to every endpoint subscribed to
    /// `webhook.created`. Asserted on the rendered string so a rename or a flatten of `Webhook`
    /// still fails this test if the value starts leaking back out.
    #[test]
    fn notification_payload_never_contains_the_wrapped_webhooks_secret_or_headers() {
        let webhook = webhook_with_credential();
        let payload = WebhookPayload::new(
            WebhookTrigger::WebhookCreated,
            webhook.id,
            Some(webhook.clone()),
        );

        let json =
            serde_json::to_string(&payload).expect("WebhookPayload always serializes to JSON");

        assert!(!json.contains(&webhook.secret));
        assert!(!json.contains("top-secret"));
        assert!(!json.contains("\"headers\""));
        assert!(!json.contains("\"secret\""));
    }
}
