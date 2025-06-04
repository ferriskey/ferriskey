use axum::extract::State;
use axum_macros::TypedPath;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
    domain::credential::{
        entities::model::{Credential, CredentialData},
        ports::credential_service::CredentialService,
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/credentials")]
pub struct GetUserCredentialsRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct CredentialOverview {
    #[typeshare(serialized_as = "string")]
    pub id: Uuid,
    #[typeshare(serialized_as = "string")]
    pub user_id: Uuid,
    pub credential_type: String,
    pub user_label: Option<String>,
    pub credential_data: CredentialData,
    #[typeshare(serialized_as = "Date")]
    pub created_at: DateTime<Utc>,
    #[typeshare(serialized_as = "Date")]
    pub updated_at: DateTime<Utc>,
}

impl From<Credential> for CredentialOverview {
    fn from(credential: Credential) -> Self {
        Self {
            id: credential.id,
            user_id: credential.user_id,
            credential_type: credential.credential_type,
            user_label: credential.user_label,
            credential_data: credential.credential_data,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct GetUserCredentialsResponse {
    pub data: Vec<CredentialOverview>,
}

#[utoipa::path(
  get,
  path = "/{user_id}/credentials",
  tag = "user",
  params(
    ("realm_name" = String, Path, description = "Realm name"),
    ("user_id" = Uuid, Path, description = "User ID"),
  ),
  responses(
    (status = 200, body = GetUserCredentialsResponse, description = "User credentials retrieved successfully"),
  )
)]
pub async fn get_user_credentials(
    GetUserCredentialsRoute {
        user_id,
        realm_name,
    }: GetUserCredentialsRoute,
    State(state): State<AppState>,
) -> Result<Response<GetUserCredentialsResponse>, ApiError> {
    info!(
        "Fetching credentials for user {} in realm {}",
        user_id, realm_name
    );
    let credentials: Vec<CredentialOverview> = state
        .credential_service
        .get_credentials_by_user_id(user_id)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .into_iter()
        .map(CredentialOverview::from)
        .collect();

    Ok(Response::OK(GetUserCredentialsResponse {
        data: credentials,
    }))
}
