use axum::{Extension, extract::State};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{PasskeyRegisterSelfServiceInput, TridentService},
};

use crate::{
    handlers::webauthn_public_key_create::{ValidatePublicKeyRequest, ValidatePublicKeyResponse},
    validators::webauthn_rp_info_from_webapp_url,
};

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/passkey/registration",
    tag = "auth",
    summary = "Complete passkey registration for the signed-in user",
    description = "Validates the WebAuthn credential produced from /me/passkey/registration-options and stores it. Bearer-only self-service endpoint; no session cookie required.",
    request_body = ValidatePublicKeyRequest,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Passkey saved", body = ValidatePublicKeyResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_passkey_register(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ValidatePublicKeyRequest>,
) -> Result<Response<ValidatePublicKeyResponse>, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let input = PasskeyRegisterSelfServiceInput { rp_info, credential: payload.0 };

    state.service.passkey_register_self_service(identity, input).await.map_err(ApiError::from)?;

    Ok(Response::OK(ValidatePublicKeyResponse {}))
}
