use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::domain::credential::{entities::CredentialOverview, ports::CredentialService};
use serde::{Deserialize, Serialize};
use tracing::info;
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}/credentials")]
pub struct GetUserCredentialsRoute {
    pub realm_name: String,
    pub user_id: Uuid,
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
        .service_bundle
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
