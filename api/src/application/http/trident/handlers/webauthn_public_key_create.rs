use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{
    RegisterPublicKeyCredential, TridentService, WebAuthnValidatePublicKeyInput,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, trident::ports::WebAuthnRpInfo,
};
use serde::{Deserialize, Serialize};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use validator::Validate;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidatePublicKeyRequest(RegisterPublicKeyCredential);

impl Validate for ValidatePublicKeyRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Ok(()) // is that correct ????
    }
}

impl ToSchema for ValidatePublicKeyRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PublicKeyCredential")
    }
}
impl PartialSchema for ValidatePublicKeyRequest {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some("Incomplete schema. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson"))
                .build()
        ))
    }
}

#[derive(Debug, ToSchema, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidatePublicKeyResponse {}

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-create",
    tag = "auth",
    summary = "Validate and save a webauthn public key",
    description = "Saving a webauthn public key to use it for authentication attempts or MFA later.",
    request_body = ValidatePublicKeyRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "WebAuthn public key created successfully", body = ValidatePublicKeyResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn webauthn_public_key_create(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<ValidatePublicKeyRequest>,
) -> Result<Response<ValidatePublicKeyResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))? // Ou un type d'erreur 401/403
        .value()
        .to_string();

    let rp_id = state.args.server.host.clone();
    let allowed_origin = state.args.webapp_url.clone();

    let input = WebAuthnValidatePublicKeyInput {
        rp_info: WebAuthnRpInfo {
            rp_id,
            allowed_origin,
        },
        session_code,
        credential: payload.0,
    };

    let _ = state
        .service
        .webauthn_public_key_create(identity, input)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ValidatePublicKeyResponse {}))
}
