use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::{
        entities::PortalLayout,
        ports::{ImportLayoutInput, PortalLayoutsService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::validators::ImportPortalLayoutValidator;
use ferriskey_api_core::api_entities::export::{ExportEnvelope, ensure_importable};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportPortalLayoutResponse {
    pub data: PortalLayout,
}

#[utoipa::path(
    post,
    path = "/import",
    tag = "portal-layouts",
    summary = "Import a portal layout",
    description = "Creates a portal layout from an exported layout tree. The tree is validated before \
                   it is stored, unlike the plain create endpoint which is fed by the builder itself.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = ImportPortalLayoutValidator,
    responses(
        (status = 201, description = "Layout imported successfully", body = ImportPortalLayoutResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 422, description = "Invalid layout tree", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn import_layout(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ImportPortalLayoutValidator>,
) -> Result<Response<ImportPortalLayoutResponse>, ApiError> {
    ensure_importable(
        payload.ferriskey.as_deref(),
        payload.version,
        ExportEnvelope::PORTAL_LAYOUT,
    )?;

    let tree = payload
        .tree
        .ok_or_else(|| ApiError::BadRequest("tree is required".into()))?;

    let layout = state
        .service
        .import_layout(
            identity,
            ImportLayoutInput {
                realm_name,
                name: payload.name,
                tree,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(ImportPortalLayoutResponse {
        data: layout,
    }))
}
