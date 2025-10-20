use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, WebAuthnPublicKeyAuthenticateInput},
};
use serde::{Deserialize, Serialize};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use validator::Validate;
use webauthn_rs::prelude::PublicKeyCredential;

#[derive(Debug, Deserialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct AuthenticationAttemptRequest(PublicKeyCredential);

impl Validate for AuthenticationAttemptRequest {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        Ok(())
    }
}

impl ToSchema for AuthenticationAttemptRequest {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PublicKeyCredential")
    }
}
impl PartialSchema for AuthenticationAttemptRequest {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .description(Some("Incomplete schema. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialjson"))
                .build()
        ))
    }
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct AuthenticationAttemptResponse {
    login_url: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-authenticate",
    tag = "auth",
    summary = "Authenticate using webauthn",
    description = "Attempt authentication using a WebAuthnAssertionResponse payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-authenticationresponsejson and https://w3c.github.io/webauthn/#authenticatorassertionresponse",
    request_body = AuthenticationAttemptRequest,
    responses(
        (status = 200, body = AuthenticationAttemptResponse),
    )
)]
pub async fn webauthn_public_key_authenticate(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
    ValidateJson(payload): ValidateJson<AuthenticationAttemptRequest>,
) -> Result<Response<AuthenticationAttemptResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let output = state
        .service
        .webauthn_public_key_authenticate(
            identity,
            WebAuthnPublicKeyAuthenticateInput {
                session_code,
                server_host: state.args.server.host.clone(),
                credential: payload.0,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(AuthenticationAttemptResponse {
        login_url: output.login_url,
    }))
}
