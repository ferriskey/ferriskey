use crate::validators::UpdateRealmSettingValidator;
use axum::Extension;
use ferriskey_core::domain::realm::ports::{RealmService, UpdateRealmSettingInput};

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;

use axum::extract::{Path, State};

use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;
use ferriskey_core::domain::webhook::entities::retry_policy::{RetryPolicy, RetryPolicyOverride};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

fn non_negative(field: &str, value: Option<Option<i32>>) -> Result<Option<u32>, ApiError> {
    match value.flatten() {
        Some(value) => u32::try_from(value)
            .map(Some)
            .map_err(|_| ApiError::BadRequest(format!("{field} must not be negative").into())),
        None => Ok(None),
    }
}

fn ensure_retry_policy_is_valid(payload: &UpdateRealmSettingValidator) -> Result<(), ApiError> {
    let candidate = RetryPolicyOverride {
        max_attempts: non_negative(
            "webhook_retry_max_attempts",
            payload.webhook_retry_max_attempts,
        )?,
        base_delay_ms: non_negative(
            "webhook_retry_base_delay_ms",
            payload.webhook_retry_base_delay_ms,
        )?,
        max_delay_ms: non_negative(
            "webhook_retry_max_delay_ms",
            payload.webhook_retry_max_delay_ms,
        )?,
        max_total_delay_ms: non_negative(
            "webhook_retry_max_total_delay_ms",
            payload.webhook_retry_max_total_delay_ms,
        )?,
    };

    if candidate.is_empty() {
        return Ok(());
    }

    RetryPolicy::try_from(candidate)
        .map(|_| ())
        .map_err(|error| ApiError::BadRequest(error.to_string().into()))
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateRealmSettingResponse {
    pub data: Realm,
}

#[utoipa::path(
    put,
    path = "/{name}/settings",
    tag = "realm",
    summary = "Update settings for a realm by name",
    description = "Updates the settings for a specified realm. This endpoint allows modification of various realm settings.",
    params(
        ("name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Realm settings updated successfully", body = UpdateRealmSettingResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    request_body = UpdateRealmSettingValidator
)]
pub async fn update_realm_setting(
    Path(name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRealmSettingValidator>,
) -> Result<Response<UpdateRealmSettingResponse>, ApiError> {
    ensure_retry_policy_is_valid(&payload)?;

    let realm = state
        .service
        .update_realm_setting(
            identity,
            UpdateRealmSettingInput {
                realm_name: name,
                algorithm: payload.default_signing_algorithm,
                forgot_password_enabled: payload.forgot_password_enabled,
                remember_me_enabled: payload.remember_me_enabled,
                user_registration_enabled: payload.user_registration_enabled,
                magic_link_enabled: payload.magic_link_enabled,
                magic_link_ttl: payload.magic_link_ttl,
                passkey_enabled: payload.passkey_enabled,
                compass_enabled: payload.compass_enabled,
                access_token_lifetime: payload.access_token_lifetime,
                refresh_token_lifetime: payload.refresh_token_lifetime,
                id_token_lifetime: payload.id_token_lifetime,
                temporary_token_lifetime: payload.temporary_token_lifetime,
                reset_password_template_id: payload.reset_password_template_id,
                magic_link_template_id: payload.magic_link_template_id,
                email_verification_template_id: payload.email_verification_template_id,
                email_verification_enabled: payload.email_verification_enabled,
                email_verification_ttl_hours: payload.email_verification_ttl_hours,
                lockout_threshold: payload.lockout_threshold,
                lockout_duration_seconds: payload.lockout_duration_seconds,
                login_aliases: payload.login_aliases,
                seawatch_pii_mode: payload.seawatch_pii_mode,
                seawatch_pseudo_key: payload.seawatch_pseudo_key,
                require_mfa: payload.require_mfa,
                edit_username_enabled: payload.edit_username_enabled,
                webhook_retry_max_attempts: payload.webhook_retry_max_attempts,
                webhook_retry_base_delay_ms: payload.webhook_retry_base_delay_ms,
                webhook_retry_max_delay_ms: payload.webhook_retry_max_delay_ms,
                webhook_retry_max_total_delay_ms: payload.webhook_retry_max_total_delay_ms,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateRealmSettingResponse {
        data: realm,
    }))
}
