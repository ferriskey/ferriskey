use axum::{
    Extension,
    extract::{Path, State},
    response::Response as AxumResponse,
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::ports::{GetLayoutInput, PortalLayoutsService},
};
use uuid::Uuid;

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::export::ExportEnvelope;
use ferriskey_api_core::app_state::AppState;

#[utoipa::path(
    get,
    path = "/{layout_id}/export",
    tag = "portal-layouts",
    summary = "Export a portal layout",
    description = "Returns the layout as a downloadable JSON envelope.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("layout_id" = Uuid, Path, description = "Portal layout ID"),
    ),
    responses(
        (status = 200, description = "Layout exported successfully", content_type = "application/json", body = ExportEnvelope),
        (status = 404, description = "Layout not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn export_layout(
    Path((realm_name, layout_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<AxumResponse, ApiError> {
    let layout = state
        .service
        .get_layout(
            identity,
            GetLayoutInput {
                realm_name,
                layout_id,
            },
        )
        .await
        .map_err(ApiError::from)?;

    ExportEnvelope::portal_layout(layout.name, layout.tree)
        .into_download()
        .map_err(|e| {
            ApiError::InternalServerError(format!("failed to serialize layout: {e}").into())
        })
}
