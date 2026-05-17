use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::ports::{GetLayoutInput, PortalLayoutsService},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse},
        response::Response,
    },
    app_state::AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeletePortalLayoutResponse {
    message: String,
}

#[utoipa::path(
    delete,
    path = "/{layout_id}",
    tag = "portal-layouts",
    summary = "Delete a portal layout",
    description = "Deletes a portal layout.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("layout_id" = Uuid, Path, description = "Portal layout ID"),
    ),
    responses(
        (status = 200, description = "Layout deleted successfully", body = DeletePortalLayoutResponse),
        (status = 404, description = "Layout not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn delete_layout(
    Path((realm_name, layout_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeletePortalLayoutResponse>, ApiError> {
    state
        .service
        .delete_layout(
            identity,
            GetLayoutInput {
                realm_name,
                layout_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(DeletePortalLayoutResponse {
        message: "Portal layout deleted successfully".to_string(),
    }))
}
