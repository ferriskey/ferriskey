use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};
use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::{TridentService, WebAuthnPublicKeyCreateOptionsInput};
use serde::{Deserialize, Serialize};
use utoipa::{openapi::{ObjectBuilder, RefOr, Schema}, PartialSchema, ToSchema};
use validator::Validate;
use webauthn_rs::prelude::CreationChallengeResponse;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePublicKeyRequest {}

/// https://w3c.github.io/webauthn/#dictdef-publickeycredentialrpentity
/// A tad bit repetetitive but its explicit
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
    path = "/protocol/webauthn/create-public-key",
    tag = "auth",
    summary = "Create a webauthn public key",
    description = "Provides a full PublicKeyCredentialCreationOption payload for WebAuthn credential creation/authentication. The payload contains the challenge to resolve in B64Url form as described in the specs. The content is described here: https://w3c.github.io/webauthn/#dictdef-publickeycredentialcreationoptions.",
    request_body = CreatePublicKeyRequest,
    responses(
        (status = 200, body = CreatePublicKeyResponse)
    )
)]
pub async fn webauthn_create_public_key(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
) -> Result<Response<CreatePublicKeyResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let server_host = state.args.server.host.clone();

    let output = state
        .service
        .webauthn_public_key_create_options(
            identity,
            WebAuthnPublicKeyCreateOptionsInput {
                session_code,
                server_host,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let response = CreatePublicKeyResponse(output.0);
    Ok(Response::OK(response))
}
