use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, ToSchema)]
pub struct PostConsentRequest {
    #[validate(length(min = 1, message = "consent_token is required"))]
    pub consent_token: String,
    #[serde(default)]
    pub approved_scopes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct PostConsentResponse {
    pub redirect_url: String,
}

#[utoipa::path(
    post,
    summary = "Submit a consent decision",
    path = "/auth/consent",
    tag = "consent",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = PostConsentRequest,
    responses(
        (status = 200, description = "Decision stored, redirect to the client", body = PostConsentResponse),
        (status = 404, description = "Unknown or expired consent token", body = ApiErrorResponse),
    )
)]
pub async fn post_consent(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<PostConsentRequest>,
) -> Result<Response<PostConsentResponse>, ApiError> {
    let redirect_url = state
        .service
        .submit_consent_decision(&realm_name, &payload.consent_token, payload.approved_scopes)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(PostConsentResponse { redirect_url }))
}
