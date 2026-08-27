use chrono::{DateTime, Utc};
use ferriskey_core::domain::abyss::identity_provider::IdentityProviderLinkView;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IdentityProviderLinkResponse {
    pub id: Uuid,
    pub identity_provider_id: Uuid,
    pub identity_provider_alias: String,
    pub identity_provider_user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<IdentityProviderLinkView> for IdentityProviderLinkResponse {
    fn from(value: IdentityProviderLinkView) -> Self {
        Self {
            id: value.id,
            identity_provider_id: value.identity_provider_id.as_uuid(),
            identity_provider_alias: value.identity_provider_alias,
            identity_provider_user_id: value.identity_provider_user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IdentityProviderLinksResponse {
    pub data: Vec<IdentityProviderLinkResponse>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct DeleteIdentityProviderLinkResponse {
    pub count: u32,
}
