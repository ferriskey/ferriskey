use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::web_origin::WebOrigin;
use ferriskey_core::domain::client::{entities::GetWebOriginsInput, ports::ClientService};
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/{client_id}/web-origins",
    summary = "List the web origins registered for a client",
    description = "Returns the origins this client is allowed to call FerrisKey from in a browser. A `+` entry expands to the origins of the client's literal redirect URIs; anchored regex redirect URIs are skipped, since an origin cannot be derived from a pattern.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client",
    responses(
        (status = 200, body = Vec<WebOrigin>),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_web_origins(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<WebOrigin>>, ApiError> {
    state
        .service
        .get_web_origins(
            identity,
            GetWebOriginsInput {
                client_id,
                realm_name,
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}
