use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    application::http::{
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
        user::validators::UpdateUserValidator,
    },
    domain::{
        realm::ports::realm_service::RealmService,
        user::{
            dtos::user_dto::UpdateUserDto, entities::model::User, ports::user_service::UserService,
        },
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
    ValidateJson(payload): ValidateJson<UpdateUserValidator>,
) -> Result<Response<UpdateUserResponse>, ApiError> {
    let _realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let user = state
        .user_service
        .update_user(
            user_id,
            UpdateUserDto {
                firstname: payload.firstname,
                lastname: payload.lastname,
                email: payload.email,
                email_verified: payload.email_verified.unwrap_or(false),
                enabled: payload.enabled.unwrap_or(true),
                required_actions: payload.required_actions,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(UpdateUserResponse { data: user }))
}
