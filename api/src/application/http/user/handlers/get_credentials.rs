use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::extract::State;
use axum::{Extension, extract::Path};
use ferriskey_core::application::user::use_cases::get_credentials_use_case::GetCredentialsUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::credential::entities::CredentialOverview;
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct GetUserCredentialsResponse {
    pub data: Vec<CredentialOverview>,
}

#[utoipa::path(
    get,
    path = "/{user_id}/credentials",
    tag = "user",
    summary = "Get user credentials in a realm",
    description = "Retrieves all credentials associated with a user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = Uuid, Path, description = "User ID"),
    ),
    responses(
        (status = 200, body = GetUserCredentialsResponse, description = "User credentials retrieved successfully"),
    )
)]
pub async fn get_user_credentials(
    Path(realm_name): Path<String>,
    Path(user_id): Path<Uuid>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<GetUserCredentialsResponse>, ApiError> {
    info!(
        "Fetching credentials for user {} in realm {}",
        user_id, realm_name
    );
    let credentials: Vec<CredentialOverview> = state
        .use_case_bundle
        .get_credentials_use_case
        .execute(
            identity,
            GetCredentialsUseCaseParams {
                user_id,
                realm_name,
            },
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Response::OK(GetUserCredentialsResponse {
        data: credentials,
    }))
}
