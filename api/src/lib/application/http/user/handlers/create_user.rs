use axum::extract::State;
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;
use utoipa::ToSchema;

use crate::{
    application::http::{
        server::{
            api_entities::{
                api_error::{ApiError, ValidateJson},
                response::Response,
            },
            app_state::AppState,
        },
        user::validators::CreateUserValidator,
    },
    domain::{
        realm::ports::realm_service::RealmService,
        user::{
            dtos::user_dto::CreateUserDto, entities::model::User, ports::user_service::UserService,
        },
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/users")]
pub struct CreateUserRoute {
    pub realm_name: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
#[typeshare]
pub struct CreateUserResponse {
    pub data: User,
}

#[utoipa::path(
    post,
    path = "",
    tag = "user",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    request_body(
        content = CreateUserValidator,
        description = "User to create",
        content_type = "application/json",
    ),
)]
pub async fn create_user(
    CreateUserRoute { realm_name }: CreateUserRoute,
    State(state): State<AppState>,
    ValidateJson(payload): ValidateJson<CreateUserValidator>,
) -> Result<Response<CreateUserResponse>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let user = state
        .user_service
        .create_user(CreateUserDto {
            client_id: None,
            realm_id: realm.id,
            username: payload.username,
            firstname: payload.firstname,
            lastname: payload.lastname,
            email: payload.email,
            email_verified: payload.email_verified.unwrap_or(false),
            enabled: true,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(CreateUserResponse { data: user }))
}
