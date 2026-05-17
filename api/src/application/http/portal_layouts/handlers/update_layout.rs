use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::{
        entities::PortalLayout,
        ports::{PortalLayoutsService, UpdateLayoutInput},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::application::http::{
    portal_layouts::validators::UpdatePortalLayoutValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdatePortalLayoutResponse {
    pub data: PortalLayout,
}

#[utoipa::path(
    put,
    path = "/{layout_id}",
    tag = "portal-layouts",
    summary = "Update a portal layout",
    description = "Updates the name and tree of an existing portal layout.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
        ("layout_id" = Uuid, Path, description = "Portal layout ID"),
    ),
    request_body = UpdatePortalLayoutValidator,
    responses(
        (status = 200, description = "Layout updated successfully", body = UpdatePortalLayoutResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 404, description = "Layout not found", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn update_layout(
    Path((realm_name, layout_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdatePortalLayoutValidator>,
) -> Result<Response<UpdatePortalLayoutResponse>, ApiError> {
    let layout = state
        .service
        .update_layout(
            identity,
            UpdateLayoutInput {
                realm_name,
                layout_id,
                name: payload.name,
                tree: payload.tree,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Updated(UpdatePortalLayoutResponse {
        data: layout,
    }))
}
