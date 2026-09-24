use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::account_security::entities::StartOwnPasskeyRegistrationInput;
use ferriskey_core::domain::account_security::ports::AccountSecurityService;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::CreationChallengeResponse;
use serde::Serialize;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};

use crate::session::CallerSession;
use crate::validators::{ElevationOnlyValidator, webauthn_rp_info_from_webapp_url};

#[derive(Debug, Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct PasskeyCreationOptionsResponse(CreationChallengeResponse);

impl ToSchema for PasskeyCreationOptionsResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("AccountPublicKeyCredentialCreationOptionsJSON")
    }
}

impl PartialSchema for PasskeyCreationOptionsResponse {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some(
                    "Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptionsjson",
                ))
                .build(),
        ))
    }
}

#[utoipa::path(
    post,
    path = "/me/passkeys/options",
    tag = "account",
    summary = "Start registering a passkey",
    description = "Returns a WebAuthn creation challenge for the caller's own account, excluding the passkeys they already hold. The challenge is recorded server-side under a short TTL and answered by `POST /passkeys`. Requires an elevation proved with a password: a passkey is a primary sign-in means, so a second factor alone must not be able to add one.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = ElevationOnlyValidator,
        description = "A live elevation",
        content_type = "application/json",
    ),
    responses(
        (status = 200, description = "Challenge issued", body = PasskeyCreationOptionsResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "The account is locked or disabled", body = ApiErrorResponse),
        (status = 403, description = "No live elevation for this caller and session, or the elevation was not proved with a password", body = ApiErrorResponse),
        (status = 404, description = "No such realm", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn start_own_passkey_registration(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    CallerSession(session_id): CallerSession,
    ValidateJson(payload): ValidateJson<ElevationOnlyValidator>,
) -> Result<Response<PasskeyCreationOptionsResponse>, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let challenge = state
        .service
        .start_own_passkey_registration(
            identity,
            StartOwnPasskeyRegistrationInput {
                realm_name,
                session_id,
                elevation_id: payload.elevation_id,
                rp_info,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(PasskeyCreationOptionsResponse(challenge.0)))
}
