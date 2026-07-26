use std::borrow::Cow;

use axum::{
    Json,
    extract::{Form, FromRequest, Request, rejection::FormRejection},
    http::StatusCode,
    response::IntoResponse,
};
use ferriskey_core::domain::jwt::JwtError;
use ferriskey_core::domain::{
    authentication::entities::AuthenticationError, webhook::entities::errors::WebhookError,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct ValidationError {
    pub message: Cow<'static, str>,
    pub field: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum ApiError {
    InternalServerError(Cow<'static, str>),
    UnProcessableEntity(Vec<ValidationError>),
    NotFound(Cow<'static, str>),
    Unauthorized(Cow<'static, str>),
    Forbidden(Cow<'static, str>),
    BadRequest(Cow<'static, str>),
    ServiceUnavailable(Cow<'static, str>),
    /// RFC 6749 §5.2 OAuth2 error response
    OAuthError {
        error: Cow<'static, str>,
        error_description: Cow<'static, str>,
    },
}

impl ApiError {
    pub fn validation_error(
        message: impl Into<Cow<'static, str>>,
        field: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::UnProcessableEntity(vec![ValidationError {
            message: message.into(),
            field: field.into(),
        }])
    }

    pub fn validation_errors(errors: Vec<ValidationError>) -> Self {
        Self::UnProcessableEntity(errors)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidateJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidateJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, Rejection = FormRejection>,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| ApiError::BadRequest(format!("Unexpected payload: {err}").into()))?;

        value.validate()?;

        Ok(ValidateJson(value))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        match e {
            e if e.to_string().contains("validation error") => {
                Self::UnProcessableEntity(vec![ValidationError {
                    message: e.to_string().into(),
                    field: "unknown".into(),
                }])
            }
            _ => Self::InternalServerError(e.to_string().into()),
        }
    }
}

// Implémentation de From<validator::ValidationErrors> pour ApiError
impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut validation_errors = Vec::new();

        for (field, error_msgs) in errors.field_errors() {
            for error in error_msgs {
                let message = error
                    .message
                    .clone()
                    .unwrap_or_else(|| Cow::Owned(format!("Validation failed on {field}")));

                validation_errors.push(ValidationError {
                    message,
                    field: field.clone(),
                });
            }
        }

        Self::UnProcessableEntity(validation_errors)
    }
}

impl From<AuthenticationError> for ApiError {
    fn from(error: AuthenticationError) -> Self {
        match error {
            AuthenticationError::NotFound => Self::NotFound("Token not found".into()),
            AuthenticationError::Invalid => Self::Unauthorized("Invalid client".into()),
            AuthenticationError::InternalServerError => {
                Self::InternalServerError("Internal server error".into())
            }
            AuthenticationError::InvalidClient => Self::NotFound("Client not found".into()),
            AuthenticationError::InvalidPassword => Self::Unauthorized("Invalid password".into()),
            AuthenticationError::InvalidRealm => Self::Unauthorized("Realm not found".into()),
            AuthenticationError::InvalidState => Self::Unauthorized("Invalid state".into()),
            AuthenticationError::InvalidUser => Self::Unauthorized("User not found".into()),
            AuthenticationError::ServiceAccountNotFound => {
                Self::NotFound("Service account not found".into())
            }
            AuthenticationError::InvalidRefreshToken => {
                Self::Unauthorized("Invalid refresh token".into())
            }
            AuthenticationError::InvalidClientSecret => {
                Self::Unauthorized("Invalid client secret".into())
            }
            AuthenticationError::InvalidRequest => {
                Self::Unauthorized("Invalid authorization request".into())
            }
        }
    }
}

impl From<JwtError> for ApiError {
    fn from(error: JwtError) -> Self {
        match error {
            JwtError::InvalidToken => Self::Unauthorized("Invalid token".into()),
            JwtError::ValidationError(e) => Self::InternalServerError(e.into()),
            JwtError::ExpirationError(e) => Self::InternalServerError(e.into()),
            JwtError::GenerationError(e) => Self::InternalServerError(e.into()),
            JwtError::HashingError(e) => Self::InternalServerError(e.into()),
            JwtError::ExpiredToken => Self::InternalServerError("Token expired".into()),
            JwtError::InvalidKey(e) => Self::InternalServerError(e.into()),
            JwtError::ParsingError(e) => Self::InternalServerError(e.into()),
            JwtError::RealmKeyNotFound => Self::InternalServerError("Realm key not found".into()),
        }
    }
}

impl From<WebhookError> for ApiError {
    fn from(error: WebhookError) -> Self {
        match error {
            WebhookError::Forbidden => Self::Unauthorized("Invalid webhook".into()),
            WebhookError::NotFound => Self::NotFound("Webhook not found".into()),
            WebhookError::InternalServerError => {
                Self::InternalServerError("Internal server error".into())
            }
            WebhookError::RealmNotFound => Self::InternalServerError("Realm not found".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: String,
    pub status: u16,
    pub message: String,
}

/// RFC 6749 §5.2 OAuth2 error response body
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct OAuth2ErrorResponse {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidationErrorResponse {
    pub errors: Vec<ValidationError>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::InternalServerError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiErrorResponse {
                    code: "E_INTERNAL_SERVER_ERROR".to_string(),
                    status: 500,
                    message: format!("Internal Server Error: {e}"),
                }),
            )
                .into_response(),
            ApiError::UnProcessableEntity(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ValidationErrorResponse { errors }),
            )
                .into_response(),
            ApiError::NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ApiErrorResponse {
                    code: "E_NOT_FOUND".to_string(),
                    status: 404,
                    message: message.into(),
                }),
            )
                .into_response(),
            ApiError::Unauthorized(message) => (
                StatusCode::UNAUTHORIZED,
                Json(ApiErrorResponse {
                    code: "E_UNAUTHORIZED".to_string(),
                    status: 401,
                    message: message.into(),
                }),
            )
                .into_response(),
            ApiError::Forbidden(message) => (
                StatusCode::FORBIDDEN,
                Json(ApiErrorResponse {
                    code: "E_FORBIDDEN".to_string(),
                    status: 403,
                    message: message.into(),
                }),
            )
                .into_response(),
            ApiError::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse {
                    code: "E_BAD_REQUEST".to_string(),
                    status: 400,
                    message: message.into(),
                }),
            )
                .into_response(),
            ApiError::ServiceUnavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ApiErrorResponse {
                    code: "E_SERVICE_UNAVAILABLE".to_string(),
                    status: 503,
                    message: message.into(),
                }),
            )
                .into_response(),
            ApiError::OAuthError {
                error,
                error_description,
            } => (
                StatusCode::BAD_REQUEST,
                Json(OAuth2ErrorResponse {
                    error: error.into(),
                    error_description: error_description.into(),
                }),
            )
                .into_response(),
        }
    }
}
