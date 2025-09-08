use crate::application::http::{
    server::{
        api_entities::{
            api_error::{ApiError, ValidateJson},
            response::Response,
        },
        app_state::AppState,
    },
    user::validators::CreateUserValidator,
};
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::user::entities::User;
use ferriskey_core::domain::user::{entities::CreateUserInput, ports::UserService};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct CreateUserResponse {
    pub data: User,
}

#[utoipa::path(
    post,
    path = "",
    tag = "user",
    summary = "Create a new user in a realm",
    responses(
        (status = 201, body = CreateUserResponse, description = "User created successfully"),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
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
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateUserValidator>,
) -> Result<Response<CreateUserResponse>, ApiError> {
    let user = state
        .service
        .create_user(
            identity,
            CreateUserInput {
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
