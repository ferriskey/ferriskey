use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::ports::{DeleteSmtpConfigInput, MailService},
};

use crate::application::http::server::{
    api_entities::api_error::{ApiError, ApiErrorResponse},
    app_state::AppState,
};

#[utoipa::path(
    delete,
    path = "/{realm_name}/smtp-config",
    tag = "realm",
    summary = "Delete SMTP configuration for a realm",
    description = "Deletes the SMTP configuration for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 204, description = "SMTP config deleted successfully"),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_smtp_config(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<axum::http::StatusCode, ApiError> {
    state
        .service
        .delete_smtp_config(identity, DeleteSmtpConfigInput { realm_name })
        .await
        .map_err(ApiError::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
