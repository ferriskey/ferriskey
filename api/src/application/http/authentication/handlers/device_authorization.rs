use super::auth::root_scoped_base_url;
use crate::application::http::authentication::basic_auth::try_parse_basic_client_credentials;
use crate::application::http::server::api_entities::api_error::{ApiError, ApiErrorResponse};
use crate::application::http::server::app_state::AppState;
use crate::application::url::FullUrl;
use axum::{
    Form, Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use ferriskey_core::domain::authentication::device_flow::value_objects::{
    InitiateDeviceFlowInput, InitiateDeviceFlowOutput,
};
use serde::Deserialize;
use tracing::{instrument, warn};
use utoipa::ToSchema;

/// Form body for the device authorization request (RFC 8628 §3.1).
#[derive(Debug, Deserialize, ToSchema)]
pub struct DeviceAuthorizationRequest {
    /// OAuth 2.0 client identifier. Optional in the body when the client
    /// authenticates with HTTP Basic (confidential clients).
    pub client_id: Option<String>,
    /// Space-delimited list of requested scopes.
    pub scope: Option<String>,
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/auth/device",
    tag = "auth",
    summary = "Device Authorization Request",
    description = "Initiates the OAuth 2.0 Device Authorization Grant (RFC 8628 §3.1). Public clients pass `client_id` in the form body; confidential clients authenticate with HTTP Basic. Returns a device code, an end-user code, and the verification URIs the device should display.",
    request_body(
        content = DeviceAuthorizationRequest,
        content_type = "application/x-www-form-urlencoded"
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, body = InitiateDeviceFlowOutput),
        (status = 400, description = "Missing client_id", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
#[instrument(skip(state, payload, headers), fields(realm_name = %realm_name))]
pub async fn device_authorization(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    headers: HeaderMap,
    Form(payload): Form<DeviceAuthorizationRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Confidential clients authenticate via HTTP Basic (username = client_id);
    // public clients send `client_id` in the form body.
    let client_id = match try_parse_basic_client_credentials(&headers) {
        Some((id, _secret)) => id,
        None => payload.client_id.clone().unwrap_or_default(),
    };

    if client_id.is_empty() {
        return Err(ApiError::BadRequest("client_id is required".into()));
    }

    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    let output: InitiateDeviceFlowOutput = state
        .service
        .initiate_device_authorization(
            InitiateDeviceFlowInput {
                realm_name,
                client_id: client_id.clone(),
                scope: payload.scope,
            },
            base_url,
        )
        .await
        .map_err(|error| {
            warn!(client_id = %client_id, error = ?error, "Device authorization request failed");
            ApiError::from(error)
        })?;

    Ok((StatusCode::OK, Json(output)))
}
