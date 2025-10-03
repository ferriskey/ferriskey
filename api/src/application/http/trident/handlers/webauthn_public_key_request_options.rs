use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::{
        entities::WebAuthnPublicKeyCredentialRequestOptions,
        ports::{TridentService, WebAuthnPublicKeyRequestOptionsInput},
    },
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct RequestOptionsRequest {}

#[derive(Debug, Serialize, ToSchema, PartialEq, Eq)]
pub struct RequestOptionsResponse(WebAuthnPublicKeyCredentialRequestOptions);

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-request-options",
    tag = "auth",
    summary = "Request webauthn challenge",
    description = "Provides a full PublicKeyCredentialRequestOption payload for webauthn authentication. See https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptions and https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson",
    request_body = RequestOptionsRequest,
    responses(
        (status = 200, body = RequestOptionsResponse),
    )
)]
pub async fn webauthn_public_key_request_options(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
) -> Result<Response<RequestOptionsResponse>, ApiError> {
    let session_code = cookie.get("FERRISKEY_SESSION").unwrap();
    let session_code = session_code.value().to_string();

    let server_host = state.args.server.host.clone();

    let output = state
        .service
        .webauthn_public_key_request_options(
            identity,
            WebAuthnPublicKeyRequestOptionsInput {
                session_code,
                server_host,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let response = RequestOptionsResponse(output.0);
    Ok(Response::OK(response))
}
