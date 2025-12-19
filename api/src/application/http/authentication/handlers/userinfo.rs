use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::{
    ports::AuthService,
    value_objects::{GetUserInfoInput, Identity, UserInfoResponse},
};

use crate::application::{
    decoded_token::OptionalToken,
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[utoipa::path(
    get,
    path = "/protocol/openid-connect/userinfo",
    tag = "auth",
    summary = "Get user info",
    description = "Retrieves the user information for the authenticated user in a specific realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, body = UserInfoResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn get_userinfo(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    OptionalToken(optional_token): OptionalToken,
) -> Result<Response<UserInfoResponse>, ApiError> {
    let result_token = optional_token
        .ok_or_else(|| ApiError::Unauthorized("Missing or invalid access token".to_string()))?;

    let user_info = state
        .service
        .get_userinfo(GetUserInfoInput {
            identity,
            realm_name,
            token: result_token.decoded_token,
        })
        .await?;

    Ok(Response::OK(user_info))
}
