use crate::application::http::{
    client::validators::UpdateRedirectUriValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::entities::redirect_uri::RedirectUri;
use ferriskey_core::domain::client::{entities::UpdateRedirectUriInput, ports::ClientService};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct UpdateRedirectUriResponse {
    pub data: RedirectUri,
}

#[utoipa::path(
    put,
    path = "/{client_id}/redirects/{uri_id}",
    summary = "Update a redirect URI for a client",
    description = "Updates an existing redirect URI for a client in a specific realm. This endpoint allows you to modify the enabled status of a redirect URI.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
        ("uri_id" = Uuid, Path, description = "Redirect URI ID"),
    ),
    tag = "client",
    request_body = UpdateRedirectUriValidator,
    responses(
        (status = 200, body = UpdateRedirectUriResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 404, description = "Redirect not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_redirect_uri(
    Path((realm_name, client_id, uri_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateRedirectUriValidator>,
) -> Result<Response<UpdateRedirectUriResponse>, ApiError> {
    info!(
        "Updating redirect URI: realm_name={}, client_id={}, uri_id={}",
        realm_name, client_id, uri_id
    );
    let redirect_uri = state
        .service
        .update_redirect_uri(
            identity,
            UpdateRedirectUriInput {
                redirect_uri_id: uri_id,
                realm_name,
                client_id,
                enabled: payload.enabled,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdateRedirectUriResponse {
        data: redirect_uri,
    }))
}
