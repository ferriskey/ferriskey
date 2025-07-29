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
            user::validators::CreateUserValidator,
        },
    },
    domain::user::{
        entities::model::User, use_cases::create_user_use_case::CreateUserUseCaseParams,
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
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateUserValidator>,
) -> Result<Response<CreateUserResponse>, ApiError> {
    let user = state
        .user_orchestrator
        .create_user(
            identity,
            CreateUserUseCaseParams {
                realm_name,
                username: payload.username,
                firstname: payload.firstname,
                lastname: payload.lastname,
                email: payload.email,
                email_verified: payload.email_verified,
            },
        )
        .await?;

    Ok(Response::OK(CreateUserResponse { data: user }))
}
