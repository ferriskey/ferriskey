use crate::validators::OtpVerifyRequest;
use axum::{Extension, extract::State, response::IntoResponse};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
    device_cookie::LoginDevice,
    login_step::{LoginActionStep, LoginStepReply},
    url::{FullUrl, WebappUrl},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, VerifyOtpInput},
    user::entities::RequiredAction,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VerifyOtpResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required_actions: Vec<RequiredAction>,
}

#[utoipa::path(
    post,
    path = "/login-actions/verify-otp",
    tag = "auth",
    summary = "Verify OTP for user authentication",
    description = "Verifies the One-Time Password (OTP) provided by the user. This is typically used in multi-factor authentication scenarios.",
    request_body = OtpVerifyRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "OTP verified successfully", body = VerifyOtpResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "No pending enrollment, or the code does not match it", body = ApiErrorResponse),
        (status = 403, description = "OTP is already configured and no re-configuration was requested", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn verify_otp(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
    WebappUrl(webapp_url): WebappUrl,
    Extension(step): Extension<LoginActionStep>,
    device: LoginDevice,
    ValidateJson(payload): ValidateJson<OtpVerifyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let LoginDevice {
        realm_name,
        token: device,
    } = device;
    let result = state
        .service
        .verify_otp(
            identity,
            VerifyOtpInput {
                realm_name: realm_name.clone(),
                session_code: step.auth_session_id.to_string(),
                second_factor_verified: step.second_factor_verified,
                webapp_base_url: webapp_url,
                code: payload.code,
                label: Some(payload.label),
            },
        )
        .await
        .map_err(ApiError::from)?;

    let mut reply = LoginStepReply::new(result.progress, base_url.starts_with("https"))?;
    reply
        .attach_device(&state, &realm_name, device, base_url.starts_with("https"))
        .await;

    Ok((
        reply.headers,
        reply.continues,
        Response::OK(VerifyOtpResponse {
            message: result.message,
            url: reply.url,
            required_actions: reply.required_actions,
        }),
    ))
}
