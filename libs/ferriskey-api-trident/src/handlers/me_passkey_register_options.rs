use axum::{Extension, extract::State};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{PasskeyRegisterOptionsSelfServiceInput, TridentService},
};

use crate::{
    handlers::webauthn_public_key_create_options::CreatePublicKeyResponse,
    validators::webauthn_rp_info_from_webapp_url,
};

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/passkey/registration-options",
    tag = "auth",
    summary = "Start passkey registration for the signed-in user",
    description = "Returns a PublicKeyCredentialCreationOptionsJSON payload for the authenticated user. Bearer-only self-service endpoint; no session cookie required.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Passkey registration options generated", body = CreatePublicKeyResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn me_passkey_register_options(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CreatePublicKeyResponse>, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let output = state
        .service
        .passkey_register_options_self_service(
            identity,
            PasskeyRegisterOptionsSelfServiceInput { rp_info },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(CreatePublicKeyResponse(output.0)))
}
