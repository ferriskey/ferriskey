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
use ferriskey_core::domain::client::{entities::DeleteWebOriginInput, ports::ClientService};
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/{client_id}/web-origins/{web_origin_id}",
    summary = "Remove a web origin from a client",
    description = "Removes a registered origin. Browsers may keep honouring a cached preflight until its max-age lapses, and other API replicas may keep serving the origin for up to one cache TTL.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("web_origin_id" = Uuid, Path, description = "Web origin ID"),
    ),
    tag = "client",
    responses(
        (status = 200, description = "Web origin removed"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Web origin not found for this client", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_web_origin(
    Path((realm_name, client_id, web_origin_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    state
        .service
        .delete_web_origin(
            identity,
            DeleteWebOriginInput {
                client_id,
                web_origin_id,
                realm_name: realm_name.clone(),
            },
        )
        .await
        .map_err(ApiError::from)?;

    state.web_origin_cache.invalidate(&realm_name);

    Ok(Response::OK(()))
}
