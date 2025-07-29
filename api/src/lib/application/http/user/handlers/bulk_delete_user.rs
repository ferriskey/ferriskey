use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::{
        auth::Identity,
        http::{
            server::{
                api_entities::{
                    api_error::{ApiError, ValidateJson},
                    response::Response,
                },
                app_state::AppState,
            },
            user::validators::BulkDeleteUserValidator,
        },
    },
    domain::user::use_cases::bulk_delete_user::BulkDeleteUserUseCaseParams,
};

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct BulkDeleteUserResponse {
    pub count: u32,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/bulk")]
pub struct BulkDeleteUserRoute {
    pub realm_name: String,
}

#[utoipa::path(
    delete,
    path = "/bulk",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("ids" = Vec<Uuid>, Path, description = "User IDs"),
    ),
)]
pub async fn bulk_delete_user(
    BulkDeleteUserRoute { realm_name }: BulkDeleteUserRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<BulkDeleteUserValidator>,
) -> Result<Response<BulkDeleteUserResponse>, ApiError> {
    let count = state
        .user_orchestrator
        .bulk_delete_user(
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
