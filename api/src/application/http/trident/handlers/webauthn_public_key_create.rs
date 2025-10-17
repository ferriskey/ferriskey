use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use axum::{Extension, extract::State};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::{TridentService, WebAuthnValidatePublicKeyInput};
use serde::{Deserialize, Serialize};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use validator::Validate;
use webauthn_rs::prelude::RegisterPublicKeyCredential;

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
    responses(
        (status = 200, body = ValidatePublicKeyResponse),
    )
)]
pub async fn webauthn_public_key_create(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ValidatePublicKeyRequest>,
) -> Result<Response<ValidatePublicKeyResponse>, ApiError> {
    let input = WebAuthnValidatePublicKeyInput(payload.0);

    let _ = state
        .service
        .webauthn_validate_public_key(identity, input)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(ValidatePublicKeyResponse {}))
}
