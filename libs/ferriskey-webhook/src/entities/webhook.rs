use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use ferriskey_domain::generate_timestamp;

use crate::entities::webhook_subscriber::WebhookSubscriber;
use crate::signing::generate_secret;

/// A configured webhook subscription.
///
/// `headers` and `secret` are `skip_serializing`: no serialized representation of a `Webhook` —
/// an HTTP read response, or a `webhook.*` notification naming this row as its payload `data` —
/// can disclose this webhook's stored credentials or another webhook's. Both fields stay plain,
/// unrestricted struct fields otherwise: the code that composes an outbound delivery for this
/// webhook reads them directly, which is the only path meant to see them again after creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Webhook {
    pub id: Uuid,
    pub endpoint: String,
    #[serde(skip_serializing, default)]
    pub headers: HashMap<String, String>,
    #[serde(skip_serializing, default)]
    pub secret: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub subscribers: Vec<WebhookSubscriber>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Webhook {
    /// Builds a new webhook and mints its delivery secret via [`generate_secret`]. The secret is
    /// assigned exactly once, here; nothing else in this crate assigns it, so whoever persists
    /// the returned value is persisting the only copy anyone will ever be able to show its owner.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        endpoint: String,
        headers: HashMap<String, String>,
        subscribers: Vec<WebhookSubscriber>,
        name: Option<String>,
        description: Option<String>,
        triggered_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Self {
        let (_, timestamp) = generate_timestamp();

        Self {
            id: Uuid::new_v7(timestamp),
            secret: generate_secret(),
            endpoint,
            headers,
            name,
            description,
            subscribers,
            triggered_at,
            updated_at,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_webhook() -> Webhook {
        let now = Utc::now();
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer top-secret".to_string());

        Webhook::new(
            "https://example.com/hook".to_string(),
            headers,
            Vec::new(),
            Some("name".to_string()),
            None,
            None,
            now,
            now,
        )
    }

    #[test]
    fn new_generates_a_non_empty_secret() {
        let webhook = sample_webhook();

        assert!(!webhook.secret.is_empty());
    }

    #[test]
    fn new_generates_distinct_secrets_across_calls() {
        let first = sample_webhook();
        let second = sample_webhook();

        assert_ne!(first.secret, second.secret);
    }

    /// Asserts on the rendered JSON string rather than on field presence, so a future refactor
    /// that renames `headers`/`secret` or flattens `Webhook` into a wrapper still fails this test
    /// if the credential value itself leaks back out.
    #[test]
    fn serialized_response_never_contains_the_secret_or_header_values() {
        let webhook = sample_webhook();
        let json = serde_json::to_string(&webhook).expect("Webhook always serializes to JSON");

        assert!(!json.contains(&webhook.secret));
        assert!(!json.contains("top-secret"));
        assert!(!json.contains("\"headers\""));
        assert!(!json.contains("\"secret\""));
    }
}
