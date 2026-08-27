use crate::validators::CreateWebOriginValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::web_origin::WebOrigin;
use ferriskey_core::domain::client::value_objects::CreateWebOriginRequest;
use ferriskey_core::domain::client::{entities::CreateWebOriginInput, ports::ClientService};
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/{client_id}/web-origins",
    summary = "Register a web origin for a client",
    description = "Registers an origin the client is allowed to call FerrisKey from in a browser. The value must be a serialized origin (scheme://host[:port], no path), or the `+` sentinel meaning \"derive the origins from this client's literal redirect URIs\". Enforcement is per realm: the allowlist a browser is answered with is the union of the origins of every enabled client of the realm.",
    responses(
        (status = 201, body = WebOrigin, description = "Web origin registered"),
        (status = 400, description = "The value is not a valid origin, or is already registered", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    request_body = CreateWebOriginValidator,
)]
pub async fn create_web_origin(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateWebOriginValidator>,
) -> Result<Response<WebOrigin>, ApiError> {
    let web_origin = state
        .service
        .create_web_origin(
            identity,
            CreateWebOriginInput {
                client_id,
                payload: CreateWebOriginRequest {
                    value: payload.value,
                },
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    state.web_origin_cache.invalidate(&realm_name);

    Ok(Response::Created(web_origin))
}
