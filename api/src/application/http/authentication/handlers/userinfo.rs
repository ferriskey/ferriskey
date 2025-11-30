use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::application::{
    decoded_token::OptionalToken,
    http::server::{
        api_entities::{api_error::ApiError, response::Response},
        app_state::AppState,
    },
};

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq)]
pub struct GetUserInfoResponse {
    sub: String,
    email_verified: bool,
    name: String,
    preferred_username: String,
    given_name: String,
    family_name: String,
    email: String,
}

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
        (status = 200, body = GetUserInfoResponse),
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
) -> Result<Response<GetUserInfoResponse>, ApiError> {
    let token = optional_token.unwrap();

    println!("{:?}", token);

    Ok(Response::OK(GetUserInfoResponse {
        email: "".to_string(),
        email_verified: true,
        family_name: "".to_string(),
        given_name: "".to_string(),
        name: "".to_string(),
        preferred_username: "".to_string(),
        sub: "".to_string(),
    }))
}
