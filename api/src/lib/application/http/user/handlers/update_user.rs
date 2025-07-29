use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

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
            user::validators::UpdateUserValidator,
        },
    },
    domain::user::{
        entities::model::User, use_cases::update_user_use_case::UpdateUserUseCaseParams,
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users/{user_id}")]
pub struct UpdateUserRoute {
    pub realm_name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct UpdateUserResponse {
    pub data: User,
}

#[utoipa::path(
    post,
    path = "/{user_id}",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("user_id" = String, Path, description = "User ID"),
    ),
    request_body(
        content = UpdateUserValidator,
        description = "User to update",
        content_type = "application/json",
    ),
)]
pub async fn update_user(
    UpdateUserRoute {
        realm_name,
        user_id,
    }: UpdateUserRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<UpdateUserValidator>,
) -> Result<Response<UpdateUserResponse>, ApiError> {
    let user = state
        .user_orchestrator
        .update_user(
            identity,
            UpdateUserUseCaseParams {
                user_id,
                realm_name,
                firstname: payload.firstname,
                lastname: payload.lastname,
                email: payload.email,
                email_verified: payload.email_verified,
                enabled: payload.enabled.unwrap_or(true),
                required_actions: payload.required_actions,
            },
        )
        .await?;

    Ok(Response::OK(UpdateUserResponse { data: user }))
}
