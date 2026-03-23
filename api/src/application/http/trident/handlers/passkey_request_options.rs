use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{
    PasskeyRequestOptionsInput, RequestChallengeResponse, TridentService,
};
use serde::{Deserialize, Serialize};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use validator::Validate;

use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    trident::validators::webauthn_rp_info_from_webapp_url,
};

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct PasskeyRequestOptionsRequest {
    pub username: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct PasskeyRequestOptionsResponse(RequestChallengeResponse);

impl ToSchema for PasskeyRequestOptionsResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PasskeyPublicKeyCredentialRequestOptionsJSON")
    }
}

impl PartialSchema for PasskeyRequestOptionsResponse {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some("PublicKeyCredentialRequestOptionsJSON for passkey authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson"))
                .build(),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/login-actions/passkey-request-options",
    tag = "auth",
    summary = "Request passkey authentication challenge",
    description = "Initiates a passkey authentication flow. If username is provided, returns a challenge scoped to that user's passkeys. If omitted, returns a discoverable challenge allowing the browser to propose available passkeys.",
    request_body = PasskeyRequestOptionsRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Passkey request options generated successfully", body = PasskeyRequestOptionsResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn passkey_request_options(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<PasskeyRequestOptionsRequest>,
) -> Result<Response<PasskeyRequestOptionsResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))?
        .value()
        .to_string();

    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let output = state
        .service
        .passkey_request_options(PasskeyRequestOptionsInput {
            realm_name,
            session_code,
            username: payload.username,
            rp_info,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(PasskeyRequestOptionsResponse(output.0)))
}
