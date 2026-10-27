use crate::validators::webauthn_rp_info_from_webapp_url;
use axum::{Extension, extract::State, response::IntoResponse};
use ferriskey_api_core::login_step::{LoginActionStep, LoginStepReply};
use ferriskey_api_core::url::{FullUrl, WebappUrl};
use ferriskey_api_core::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::trident::ports::{
    RegisterPublicKeyCredential, TridentService, WebAuthnValidatePublicKeyInput,
};
use ferriskey_core::domain::user::entities::RequiredAction;
use serde::{Deserialize, Serialize};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{ObjectBuilder, RefOr, Schema},
};
use validator::Validate;

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

#[derive(Debug, ToSchema, Serialize, PartialEq, Eq)]
pub struct ValidatePublicKeyResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub required_actions: Vec<RequiredAction>,
}

#[utoipa::path(
    post,
    path = "/login-actions/webauthn-public-key-create",
    tag = "auth",
    summary = "Validate and save a webauthn public key",
    description = "Saving a webauthn public key to use it for authentication attempts or MFA later.",
    request_body = ValidatePublicKeyRequest,
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    responses(
        (status = 200, description = "WebAuthn public key created successfully", body = ValidatePublicKeyResponse),
        (status = 400, description = "Invalid request payload", body = ApiErrorResponse),
        (status = 401, description = "Missing or invalid session cookie", body = ApiErrorResponse),
        (status = 403, description = "Identity not authorized", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn webauthn_public_key_create(
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    WebappUrl(webapp_url): WebappUrl,
    Extension(identity): Extension<Identity>,
    Extension(step): Extension<LoginActionStep>,
    ValidateJson(payload): ValidateJson<ValidatePublicKeyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let rp_info = webauthn_rp_info_from_webapp_url(&webapp_url);

    let input = WebAuthnValidatePublicKeyInput {
        rp_info,
        session_code: step.auth_session_id.to_string(),
        webapp_base_url: webapp_url,
        second_factor_verified: step.second_factor_verified,
        credential: payload.0,
    };

    let result = state
        .service
        .webauthn_public_key_create(identity, input)
        .await
        .map_err(ApiError::from)?;

    let reply = LoginStepReply::new(result.progress, base_url.starts_with("https"))?;

    Ok((
        reply.headers,
        reply.continues,
        Response::OK(ValidatePublicKeyResponse {
            url: reply.url,
            required_actions: reply.required_actions,
        }),
    ))
}
