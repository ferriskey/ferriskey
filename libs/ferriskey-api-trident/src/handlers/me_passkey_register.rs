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
    trident::ports::{
        PasskeyRegisterSelfServiceInput, RegisterPublicKeyCredential, TridentService,
    },
};
use serde::Deserialize;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use validator::Validate;

use crate::{
    handlers::webauthn_public_key_create::ValidatePublicKeyResponse,
    validators::webauthn_rp_info_from_webapp_url,
};

/// Request for the self-service `/me/passkey/registration` flow. Wraps the
/// WebAuthn credential plus the step-up token minted by `/me/reauthenticate`.
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct MePasskeyRegisterRequest {
    #[serde(flatten)]
    pub credential: RegisterPublicKeyCredential,
    /// Step-up token minted by `/me/reauthenticate`.
    #[validate(length(min = 1))]
    pub step_up_token: String,
}

impl ToSchema for MePasskeyRegisterRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("MePasskeyRegisterRequest")
    }
}

impl PartialSchema for MePasskeyRegisterRequest {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some(
                    "WebAuthn credential plus step-up token. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson",
                ))
                .build(),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/realms/{realm_name}/me/passkey/registration",
    tag = "auth",
    summary = "Complete passkey registration for the signed-in user",
    description = "Validates the WebAuthn credential produced from /me/passkey/registration-options and stores it. Bearer-only self-service endpoint; no session cookie required.",
    request_body = MePasskeyRegisterRequest,
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
    ValidateJson(payload): ValidateJson<MePasskeyRegisterRequest>,
) -> Result<Response<ValidatePublicKeyResponse>, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let input = PasskeyRegisterSelfServiceInput {
        rp_info,
        credential: payload.credential,
        step_up_token: payload.step_up_token,
    };

    state
        .service
        .passkey_register_self_service(identity, input)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ValidatePublicKeyResponse {}))
}
