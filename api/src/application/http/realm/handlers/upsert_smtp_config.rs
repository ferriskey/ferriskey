use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::{
        entities::SmtpConfig,
        ports::{MailService, UpsertSmtpConfigInput},
    },
};

use crate::application::http::{
    realm::validators::UpsertSmtpConfigValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[utoipa::path(
    put,
    path = "/{realm_name}/smtp-config",
    tag = "realm",
    summary = "Create or update SMTP configuration for a realm",
    description = "Creates or updates the SMTP configuration for the specified realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = UpsertSmtpConfigValidator,
    responses(
        (status = 200, description = "SMTP config saved successfully", body = SmtpConfig),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn upsert_smtp_config(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpsertSmtpConfigValidator>,
) -> Result<Response<SmtpConfig>, ApiError> {
    state
        .service
        .upsert_smtp_config(
            identity,
            UpsertSmtpConfigInput {
                realm_name,
                host: payload.host,
                port: payload.port,
                username: payload.username,
                password: payload.password,
                from_email: payload.from_email,
                from_name: payload.from_name,
                encryption: payload.encryption,
            },
        )
        .await
        .map(Response::OK)
        .map_err(ApiError::from)
}
