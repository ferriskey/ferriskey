use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ScopeMappingService;
use ferriskey_core::domain::aegis::value_objects::GetClientClientScopesInput;
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/clients/{client_id}/client-scopes",
    summary = "Get client scopes assigned to a client",
    description = "Returns all client scopes (default and optional) assigned to the specified client.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    tag = "client-scope",
    responses(
        (status = 200, body = Vec<ClientScope>, description = "Client scopes retrieved successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_client_client_scopes(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ClientScope>>, ApiError> {
    let scopes = state
        .service
        .get_client_scopes(
            identity,
            GetClientClientScopesInput {
                realm_name,
                client_id,
            },
        )
        .await?;

    Ok(Response::OK(scopes))
}
