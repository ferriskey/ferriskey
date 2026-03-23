use axum::{Extension, extract::State};
use axum_cookie::CookieManager;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    trident::ports::{
        RequestChallengeResponse, TridentService, WebAuthnPublicKeyRequestOptionsInput,
    },
};
use serde::Serialize;
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};

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
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "WebAuthn public key request options generated successfully", body = RequestOptionsResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn webauthn_public_key_request_options(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    cookie: CookieManager,
) -> Result<Response<RequestOptionsResponse>, ApiError> {
    let session_code = cookie
        .get("FERRISKEY_SESSION")
        .ok_or_else(|| ApiError::Unauthorized("Missing session cookie".to_string()))? // Ou un type d'erreur 401/403
        .value()
        .to_string();

    let rp_info = webauthn_rp_info_from_webapp_url(&state.args.webapp_url);

    let output = state
        .service
        .webauthn_public_key_request_options(
            identity,
            WebAuthnPublicKeyRequestOptionsInput {
                session_code,
                rp_info,
            },
        )
        .await
        .map_err(ApiError::from)?;

    let response = RequestOptionsResponse(output.0);
    Ok(Response::OK(response))
}
