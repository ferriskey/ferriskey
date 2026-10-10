use crate::validators::ReplaceResourceOwnersValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::{
    authentication::value_objects::Identity,
    realm::{
        entities::ResourceOwner,
        ports::{ListResourceOwnersInput, ReplaceResourceOwnersInput, ResourceOwnerService},
    },
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ResourceOwnersResponse {
    pub data: Vec<ResourceOwner>,
}

#[utoipa::path(
    get,
    path = "/{realm_name}/resource-owners",
    tag = "realm",
    summary = "List the clients owning a protected resource",
    description = "Lists which confidential client owns each allowed resource of the realm. An owner may exchange tokens whose audience is the resource.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "Resource owners", body = ResourceOwnersResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn get_resource_owners(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ResourceOwnersResponse>, ApiError> {
    state
        .service
        .list_resource_owners(identity, ListResourceOwnersInput { realm_name })
        .await
        .map(|data| Response::OK(ResourceOwnersResponse { data }))
        .map_err(ApiError::from)
}

#[utoipa::path(
    put,
    path = "/{realm_name}/resource-owners",
    tag = "realm",
    summary = "Replace the clients owning a protected resource",
    description = "Replaces the whole set of resource owners of the realm. Every uri must be in the realm's allowed resources and appear once; every client must be a confidential, admin-registered client of the realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body = ReplaceResourceOwnersValidator,
    responses(
        (status = 200, description = "Resource owners replaced", body = ResourceOwnersResponse),
        (status = 400, description = "Invalid owners", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn replace_resource_owners(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<ReplaceResourceOwnersValidator>,
) -> Result<Response<ResourceOwnersResponse>, ApiError> {
    state
        .service
        .replace_resource_owners(
            identity,
            ReplaceResourceOwnersInput {
                realm_name,
                owners: payload.owners,
            },
        )
        .await
        .map(|data| Response::OK(ResourceOwnersResponse { data }))
        .map_err(ApiError::from)
}
