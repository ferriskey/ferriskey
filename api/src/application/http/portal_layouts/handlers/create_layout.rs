use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    portal_layouts::{
        entities::PortalLayout,
        ports::{CreateLayoutInput, PortalLayoutsService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::http::{
    portal_layouts::validators::CreatePortalLayoutValidator,
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreatePortalLayoutResponse {
    pub data: PortalLayout,
}

#[utoipa::path(
    post,
    path = "",
    tag = "portal-layouts",
    summary = "Create a portal layout",
    description = "Creates a new portal layout. The first layout in a realm is automatically marked as default.",
    params(
        ("realm_name" = String, Path, description = "Name of the realm"),
    ),
    request_body = CreatePortalLayoutValidator,
    responses(
        (status = 201, description = "Layout created successfully", body = CreatePortalLayoutResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn create_layout(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreatePortalLayoutValidator>,
) -> Result<Response<CreatePortalLayoutResponse>, ApiError> {
    let layout = state
        .service
        .create_layout(
            identity,
            CreateLayoutInput {
                realm_name,
                name: payload.name,
                tree: payload.tree,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(CreatePortalLayoutResponse {
        data: layout,
    }))
}
