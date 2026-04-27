use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse},
            response::Response,
        },
        app_state::AppState,
    },
    trident::validators::webauthn_rp_info_from_webapp_url,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::trident::ports::{TridentService, WebAuthnPublicKeyCreateOptionsInput};
use ferriskey_core::domain::{
    authentication::value_objects::Identity, trident::ports::CreationChallengeResponse,
};
use serde::Serialize;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialrpentity
#[derive(Debug, Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct CreatePublicKeyResponse(CreationChallengeResponse);

impl ToSchema for CreatePublicKeyResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PublicKeyCredentialCreationOptionsJSON")
    }
}

impl PartialSchema for CreatePublicKeyResponse {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
            .description(Some("Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptionsjson"))
            .build()
        ))
    }
}

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-create-options",
    tag = "auth",
    summary = "Create a webauthn public key",
    description = "Provides a full PublicKeyCredentialCreationOption payload for WebAuthn credential creation/authentication. The payload contains the challenge to resolve in B64Url form as described in the specs. The content is described here: https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "WebAuthn public key creation options generated successfully", body = CreatePublicKeyResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn webauthn_public_key_create_options(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
) -> Result<Response<CreatePublicKeyResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".into()))? // Ou un type d'erreur 401/403
        .value()
        .to_string();

    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let output = state
        .service
        .webauthn_public_key_create_options(
            identity,
            WebAuthnPublicKeyCreateOptionsInput {
                session_code,
                rp_info,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let response = CreatePublicKeyResponse(output.0);
    Ok(Response::OK(response))
}
