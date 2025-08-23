use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
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
use ferriskey_core::application::user::use_cases::bulk_delete_user::BulkDeleteUserUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct BulkDeleteUserResponse {
    pub count: u32,
}

#[utoipa::path(
    delete,
    path = "/bulk",
    tag = "user",
    summary = "Bulk delete users in a realm",
    description = "Deletes multiple users in a realm by their IDs. This action is irreversible and will remove all associated data.",
    responses(
        (status = 200, body = BulkDeleteUserResponse, description = "Users deleted successfully"),
    ),
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("ids" = Vec<Uuid>, Path, description = "User IDs"),
    ),
)]
pub async fn bulk_delete_user(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<BulkDeleteUserValidator>,
) -> Result<Response<BulkDeleteUserResponse>, ApiError> {
    let count = state
        .use_case_bundle
        .bulk_delete_user_use_case
        .execute(
            identity,
            BulkDeleteUserUseCaseParams {
                realm_name,
                ids: payload.ids,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(BulkDeleteUserResponse {
        count: count as u32,
    }))
}
