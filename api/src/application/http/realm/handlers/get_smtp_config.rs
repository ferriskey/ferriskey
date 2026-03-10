use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::{
        entities::SmtpConfig,
        ports::{GetSmtpConfigInput, MailService},
    },
};

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[utoipa::path(
    get,
    path = "/{realm_name}/smtp-config",
    tag = "realm",
    summary = "Get SMTP configuration for a realm",
    description = "Retrieves the SMTP configuration for the specified realm. The password field is omitted from the response.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "SMTP config retrieved successfully", body = SmtpConfig),
        (status = 404, description = "SMTP config not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_smtp_config(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SmtpConfig>, ApiError> {
    state
        .service
        .get_smtp_config(identity, GetSmtpConfigInput { realm_name })
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}
