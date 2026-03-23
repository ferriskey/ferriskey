use axum::extract::{Path, State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{
    PasskeyAuthenticateInput, PublicKeyCredential, TridentService,
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

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct PasskeyAuthenticateRequest(PublicKeyCredential);

impl Validate for PasskeyAuthenticateRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Ok(())
    }
}

impl ToSchema for PasskeyAuthenticateRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PasskeyPublicKeyCredential")
    }
}

impl PartialSchema for PasskeyAuthenticateRequest {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some("PublicKeyCredential assertion response for passkey authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson"))
                .build(),
        ))
    }
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct PasskeyAuthenticateResponse {
    login_url: String,
    status: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/passkey-authenticate",
    tag = "auth",
    summary = "Authenticate using a passkey",
    description = "Complete passkey authentication by submitting the browser's assertion response. On success, returns a login URL with an authorization code.",
    request_body = PasskeyAuthenticateRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "Passkey authentication successful", body = PasskeyAuthenticateResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn passkey_authenticate(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<PasskeyAuthenticateRequest>,
) -> Result<Response<PasskeyAuthenticateResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))?
        .value()
        .to_string();

    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let output = state
        .service
        .passkey_authenticate(PasskeyAuthenticateInput {
            realm_name,
            session_code,
            rp_info,
            credential: payload.0,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(PasskeyAuthenticateResponse {
        login_url: output.login_url,
        status: "Success".to_string(),
    }))
}
