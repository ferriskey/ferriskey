use axum::Extension;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::domain::authentication::entities::error::AuthenticationError;
use crate::domain::authentication::entities::jwt_token::JwtToken;
use crate::domain::authentication::ports::auth_session::AuthSessionService;
use crate::domain::authentication::ports::authentication::AuthenticationService;

#[derive(Serialize, Deserialize)]
pub struct AuthenticateQueryParams {
    client_id: String,
    session_code: Uuid,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthenticateRequest {
    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: String,

    #[validate(length(min = 1, message = "password is required"))]
    #[serde(default)]
    pub password: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/login-actions/authenticate")]
pub struct TokenRoute {
    realm_name: String,
}

#[utoipa::path(
    post,
    path = "/login-actions/authenticate",
    tag = "auth",
    request_body = AuthenticateRequest,
    responses(
        (status = 200, body = JwtToken)
    )
)]
pub async fn authenticate<A: AuthenticationService>(
    TokenRoute { realm_name }: TokenRoute,
    Extension(authentication_service): Extension<Arc<A>>,
    Extension(auth_session_service): Extension<Arc<dyn AuthSessionService>>,
    Query(query): Query<AuthenticateQueryParams>,
    ValidateJson(payload): ValidateJson<AuthenticateRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let auth_session = auth_session_service
        .get_by_session_code(query.session_code)
        .await
        .map_err(|_| AuthenticationError::NotFound)?;

    let code = authentication_service
        .using_session_code(
            realm_name,
            query.client_id,
            query.session_code,
            payload.username,
            payload.password,
        )
        .await?;

    let current_state = auth_session
        .state
        .ok_or(AuthenticationError::InvalidState)?;

    let login_url = format!(
        "{}?code={}&state={}",
        auth_session.redirect_uri, code, current_state
    );

    Ok(Redirect::to(&login_url))
}
