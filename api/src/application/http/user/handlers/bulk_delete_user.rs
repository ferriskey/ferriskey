use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ApiErrorResponse, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    user::validators::BulkDeleteUserValidator,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::{entities::BulkDeleteUsersInput, ports::UserService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BulkDeleteUserResponse {
    pub count: u32,
    pub realm_name: String,
}

#[utoipa::path(
    delete,
    path = "/bulk",
    tag = "user",
    summary = "Bulk delete users in a realm",
    description = "Deletes multiple users in a realm by their IDs. This action is irreversible and will remove all associated data.",
    request_body = BulkDeleteUserValidator,
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        //("ids" = Vec<Uuid>, Path, description = "User IDs"),
    ),
    responses(
        (status = 200, description = "Users deleted successfully", body = BulkDeleteUserResponse),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
)]
pub async fn bulk_delete_user(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<BulkDeleteUserValidator>,
) -> Result<Response<BulkDeleteUserResponse>, ApiError> {
    let count = state
        .service
        .bulk_delete_users(
            identity,
            BulkDeleteUsersInput {
                realm_name: realm_name.clone(),
                ids: payload.ids,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(BulkDeleteUserResponse {
        count: count as u32,
        realm_name,
    }))
}
