use crate::application::http::{
    aegis::validators::UpdateClientScopeValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::aegis::entities::ClientScope;
use ferriskey_core::domain::aegis::ports::ClientScopeService;
use ferriskey_core::domain::aegis::value_objects::{
    UpdateClientScopeInput, UpdateClientScopeRequest,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    patch,
    path = "/{scope_id}",
    summary = "Update a client scope",
    description = "Updates an existing client scope in the specified realm.",
    responses(
        (status = 200, description = "Client scope updated successfully", body = ClientScope),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("scope_id" = Uuid, Path, description = "Client scope ID"),
    ),
    tag = "client-scope",
    request_body = UpdateClientScopeValidator,
)]
pub async fn update_client_scope(
    Path((realm_name, scope_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateClientScopeValidator>,
) -> Result<Response<ClientScope>, ApiError> {
    state
        .service
        .update_client_scope(
            identity,
            UpdateClientScopeInput {
                realm_name,
                scope_id,
                payload: UpdateClientScopeRequest {
                    name: payload.name,
                    description: payload.description,
                    protocol: payload.protocol,
                    is_default: payload.is_default,
                },
            },
        )
        .await
        .map_err(ApiError::from)
        .map(Response::OK)
}
