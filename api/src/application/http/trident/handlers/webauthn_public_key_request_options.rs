use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{TridentService, WebAuthnPublicKeyRequestOptionsInput},
};
use serde::Serialize;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use webauthn_rs::prelude::RequestChallengeResponse;

use crate::application::http::server::{
    api_entities::{api_error::ApiError, response::Response},
    app_state::AppState,
};

#[derive(Debug, ToSchema, PartialEq, Eq)]
pub struct RequestOptionsRequest {}

#[derive(Debug, Serialize)]
#[serde(transparent, rename_all = "camelCase")]
pub struct RequestOptionsResponse(RequestChallengeResponse);

impl ToSchema for RequestOptionsResponse {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("PublicKeyCredentialRequestOptionsJSON")
    }
}

impl PartialSchema for RequestOptionsResponse {
    fn schema() -> RefOr<Schema> {
        RefOr::T(Schema::Object(
            ObjectBuilder::new()
            .description(Some("Incomplete schema. see https://w3c.github.io/webauthn/#dictdef-publickeycredentialrequestoptionsjson"))
            .build()
        ))
    }
}

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
