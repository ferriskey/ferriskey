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
    pub field: Cow<'static, str>,
    pub code: Cow<'static, str>,
    pub message: Cow<'static, str>,
}

impl ValidationError {
    pub fn new(
        field: impl Into<Cow<'static, str>>,
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiErrorBody {
    pub message: Cow<'static, str>,
    pub reason: Option<&'static str>,
}

impl ApiErrorBody {
    pub fn new(message: impl Into<Cow<'static, str>>, reason: &'static str) -> Self {
        Self {
            message: message.into(),
            reason: Some(reason),
        }
    }

    pub fn with_default_reason(self, reason: &'static str) -> Self {
        match self.reason {
            Some(_) => self,
            None => Self {
                reason: Some(reason),
                ..self
            },
        }
    }
}

impl From<Cow<'static, str>> for ApiErrorBody {
    fn from(message: Cow<'static, str>) -> Self {
        Self {
            message,
            reason: None,
        }
    }
}

impl From<&'static str> for ApiErrorBody {
    fn from(message: &'static str) -> Self {
        Self {
            message: Cow::Borrowed(message),
            reason: None,
        }
    }
}

impl From<String> for ApiErrorBody {
    fn from(message: String) -> Self {
        Self {
            message: Cow::Owned(message),
            reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ToSchema)]
pub enum ApiError {
    #[schema(value_type = String)]
    InternalServerError(ApiErrorBody),
    UnProcessableEntity(Vec<ValidationError>),
    #[schema(value_type = String)]
    NotFound(ApiErrorBody),
    #[schema(value_type = String)]
    Unauthorized(ApiErrorBody),
    #[schema(value_type = String)]
    Forbidden(ApiErrorBody),
    #[schema(value_type = String)]
    BadRequest(ApiErrorBody),
    #[schema(value_type = String)]
    Conflict(ApiErrorBody),
    #[schema(value_type = String)]
    ServiceUnavailable(ApiErrorBody),
    /// RFC 6749 §5.2 OAuth2 error response
    OAuthError {
        error: Cow<'static, str>,
        error_description: Cow<'static, str>,
    },
}

impl ApiError {
    pub fn validation_error(
        field: impl Into<Cow<'static, str>>,
        code: impl Into<Cow<'static, str>>,
        message: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self::UnProcessableEntity(vec![ValidationError::new(field, code, message)])
    }

    pub fn validation_errors(errors: Vec<ValidationError>) -> Self {
        Self::UnProcessableEntity(errors)
    }

    pub fn with_default_reason(self, reason: &'static str) -> Self {
        match self {
            Self::InternalServerError(body) => {
                Self::InternalServerError(body.with_default_reason(reason))
            }
            Self::NotFound(body) => Self::NotFound(body.with_default_reason(reason)),
            Self::Unauthorized(body) => Self::Unauthorized(body.with_default_reason(reason)),
            Self::Forbidden(body) => Self::Forbidden(body.with_default_reason(reason)),
            Self::BadRequest(body) => Self::BadRequest(body.with_default_reason(reason)),
            Self::Conflict(body) => Self::Conflict(body.with_default_reason(reason)),
            Self::ServiceUnavailable(body) => {
                Self::ServiceUnavailable(body.with_default_reason(reason))
            }
            Self::UnProcessableEntity(errors) => Self::UnProcessableEntity(errors),
            Self::OAuthError {
                error,
                error_description,
            } => Self::OAuthError {
                error,
                error_description,
            },
        }
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
        let Json(value) = Json::<T>::from_request(req, state).await.map_err(|err| {
            ApiError::BadRequest(ApiErrorBody::new(
                format!("Unexpected payload: {err}"),
                "unexpected_payload",
            ))
        })?;

        value.validate()?;

        Ok(ValidateJson(value))
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        match e {
            e if e.to_string().contains("validation error") => {
                Self::validation_error("unknown", "validation_error", e.to_string())
            }
            _ => {
                Self::InternalServerError(ApiErrorBody::new(e.to_string(), "internal_server_error"))
            }
        }
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut validation_errors = Vec::new();

        for (field, error_msgs) in errors.field_errors() {
            for error in error_msgs {
                let message = error
                    .message
                    .clone()
                    .unwrap_or_else(|| Cow::Owned(format!("Validation failed on {field}")));

                validation_errors.push(ValidationError::new(
                    field.clone(),
                    error.code.clone(),
                    message,
                ));
            }
        }

        Self::UnProcessableEntity(validation_errors)
    }
}

impl From<AuthenticationError> for ApiError {
    fn from(error: AuthenticationError) -> Self {
        match error {
            AuthenticationError::NotFound => {
                Self::NotFound(ApiErrorBody::new("Token not found", "token_not_found"))
            }
            AuthenticationError::Invalid => {
                Self::Unauthorized(ApiErrorBody::new("Invalid client", "invalid_client"))
            }
            AuthenticationError::InternalServerError => Self::InternalServerError(
                ApiErrorBody::new("Internal server error", "internal_server_error"),
            ),
            AuthenticationError::InvalidClient => {
                Self::NotFound(ApiErrorBody::new("Client not found", "client_not_found"))
            }
            AuthenticationError::InvalidPassword => {
                Self::Unauthorized(ApiErrorBody::new("Invalid password", "invalid_password"))
            }
            AuthenticationError::InvalidRealm => {
                Self::Unauthorized(ApiErrorBody::new("Realm not found", "invalid_realm"))
            }
            AuthenticationError::InvalidState => {
                Self::Unauthorized(ApiErrorBody::new("Invalid state", "invalid_state"))
            }
            AuthenticationError::InvalidUser => {
                Self::Unauthorized(ApiErrorBody::new("User not found", "invalid_user"))
            }
            AuthenticationError::ServiceAccountNotFound => Self::NotFound(ApiErrorBody::new(
                "Service account not found",
                "service_account_not_found",
            )),
            AuthenticationError::InvalidRefreshToken => Self::Unauthorized(ApiErrorBody::new(
                "Invalid refresh token",
                "invalid_refresh_token",
            )),
            AuthenticationError::InvalidClientSecret => Self::Unauthorized(ApiErrorBody::new(
                "Invalid client secret",
                "invalid_client_secret",
            )),
            AuthenticationError::InvalidRequest => Self::Unauthorized(ApiErrorBody::new(
                "Invalid authorization request",
                "invalid_request",
            )),
        }
    }
}

impl From<JwtError> for ApiError {
    fn from(error: JwtError) -> Self {
        match error {
            JwtError::InvalidToken => {
                Self::Unauthorized(ApiErrorBody::new("Invalid token", "invalid_token"))
            }
            JwtError::ValidationError(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "token_validation_error"))
            }
            JwtError::ExpirationError(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "token_expiration_error"))
            }
            JwtError::GenerationError(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "token_generation_error"))
            }
            JwtError::HashingError(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "hash_password_error"))
            }
            JwtError::ExpiredToken => {
                Self::InternalServerError(ApiErrorBody::new("Token expired", "expired_token"))
            }
            JwtError::InvalidKey(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "invalid_key"))
            }
            JwtError::ParsingError(e) => {
                Self::InternalServerError(ApiErrorBody::new(e, "token_parsing_error"))
            }
            JwtError::RealmKeyNotFound => Self::InternalServerError(ApiErrorBody::new(
                "Realm key not found",
                "realm_key_not_found",
            )),
            JwtError::UnsupportedHash(e) => {
                Self::validation_error("secret_data", "invalid_password_hash", e)
            }
        }
    }
}

impl From<WebhookError> for ApiError {
    fn from(error: WebhookError) -> Self {
        match error {
            WebhookError::Forbidden => {
                Self::Unauthorized(ApiErrorBody::new("Invalid webhook", "webhook_forbidden"))
            }
            WebhookError::NotFound => {
                Self::NotFound(ApiErrorBody::new("Webhook not found", "webhook_not_found"))
            }
            WebhookError::InternalServerError => Self::InternalServerError(ApiErrorBody::new(
                "Internal server error",
                "internal_server_error",
            )),
            WebhookError::RealmNotFound => Self::InternalServerError(ApiErrorBody::new(
                "Realm not found",
                "webhook_realm_not_found",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: String,
    pub status: u16,
    pub reason: String,
    pub message: String,
}

/// RFC 6749 §5.2 OAuth2 error response body
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct OAuth2ErrorResponse {
    pub error: String,
    pub error_description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    pub errors: Vec<ValidationError>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, code, default_reason, body) = match self {
            ApiError::UnProcessableEntity(errors) => {
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(ValidationErrorResponse { errors }),
                )
                    .into_response();
            }
            ApiError::OAuthError {
                error,
                error_description,
            } => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(OAuth2ErrorResponse {
                        error: error.into(),
                        error_description: error_description.into(),
                    }),
                )
                    .into_response();
            }
            ApiError::InternalServerError(body) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "E_INTERNAL_SERVER_ERROR",
                "internal_server_error",
                ApiErrorBody {
                    message: format!("Internal Server Error: {}", body.message).into(),
                    reason: body.reason,
                },
            ),
            ApiError::NotFound(body) => (StatusCode::NOT_FOUND, "E_NOT_FOUND", "not_found", body),
            ApiError::Unauthorized(body) => (
                StatusCode::UNAUTHORIZED,
                "E_UNAUTHORIZED",
                "unauthorized",
                body,
            ),
            ApiError::Forbidden(body) => (StatusCode::FORBIDDEN, "E_FORBIDDEN", "forbidden", body),
            ApiError::BadRequest(body) => (
                StatusCode::BAD_REQUEST,
                "E_BAD_REQUEST",
                "bad_request",
                body,
            ),
            ApiError::Conflict(body) => (StatusCode::CONFLICT, "E_CONFLICT", "conflict", body),
            ApiError::ServiceUnavailable(body) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "E_SERVICE_UNAVAILABLE",
                "service_unavailable",
                body,
            ),
        };

        (
            status,
            Json(ApiErrorResponse {
                code: code.to_string(),
                status: status.as_u16(),
                reason: body.reason.unwrap_or(default_reason).to_string(),
                message: body.message.into_owned(),
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
pub(crate) async fn serialized_error(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("error response body is fully buffered in memory");

    serde_json::from_slice(&body).expect("error responses are JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    async fn body_of(error: ApiError) -> Value {
        serialized_error(error.into_response()).await
    }

    #[tokio::test]
    async fn bad_request_carries_a_reason_next_to_its_http_code() {
        let body = body_of(ApiError::BadRequest(ApiErrorBody::new(
            "Email already exists in this realm",
            "email_already_exists",
        )))
        .await;

        assert_eq!(
            body,
            json!({
                "code": "E_BAD_REQUEST",
                "status": 400,
                "reason": "email_already_exists",
                "message": "Email already exists in this realm",
            })
        );
    }

    #[tokio::test]
    async fn an_untagged_error_falls_back_to_a_status_derived_reason() {
        let body = body_of(ApiError::NotFound("Resource not found".into())).await;

        assert_eq!(body["reason"], json!("not_found"));
        assert_eq!(body["code"], json!("E_NOT_FOUND"));
    }

    #[tokio::test]
    async fn every_shape_reports_its_status_as_a_number() {
        let cases = [
            (ApiError::NotFound("nope".into()), 404),
            (ApiError::Unauthorized("nope".into()), 401),
            (ApiError::Forbidden("nope".into()), 403),
            (ApiError::BadRequest("nope".into()), 400),
            (ApiError::Conflict("nope".into()), 409),
            (ApiError::ServiceUnavailable("nope".into()), 503),
            (ApiError::InternalServerError("nope".into()), 500),
        ];

        for (error, expected) in cases {
            let body = body_of(error).await;
            assert_eq!(body["status"], json!(expected));
            assert!(body["reason"].as_str().is_some_and(|r| !r.is_empty()));
        }
    }

    #[tokio::test]
    async fn oauth_errors_keep_the_rfc_6749_shape() {
        let body = body_of(ApiError::OAuthError {
            error: "invalid_grant".into(),
            error_description: "code_verifier does not match code_challenge".into(),
        })
        .await;

        assert_eq!(
            body,
            json!({
                "error": "invalid_grant",
                "error_description": "code_verifier does not match code_challenge",
            })
        );
    }

    #[tokio::test]
    async fn validation_responses_expose_one_coded_entry_per_error() {
        let body = body_of(ApiError::validation_errors(vec![
            ValidationError::new("password", "too_short", "Password is too short"),
            ValidationError::new("password", "missing_number", "Password needs a number"),
        ]))
        .await;

        assert_eq!(
            body,
            json!({
                "errors": [
                    {
                        "field": "password",
                        "code": "too_short",
                        "message": "Password is too short",
                    },
                    {
                        "field": "password",
                        "code": "missing_number",
                        "message": "Password needs a number",
                    },
                ]
            })
        );
    }

    #[test]
    fn a_default_reason_never_overwrites_an_explicit_one() {
        let error = ApiError::BadRequest(ApiErrorBody::new("boom", "invalid_redirect_uri"))
            .with_default_reason("invalid");

        assert_eq!(
            error,
            ApiError::BadRequest(ApiErrorBody::new("boom", "invalid_redirect_uri"))
        );
    }

    #[test]
    fn a_default_reason_leaves_the_oauth_shape_untouched() {
        let error = ApiError::OAuthError {
            error: "invalid_grant".into(),
            error_description: "nope".into(),
        }
        .with_default_reason("invalid_authorization_code");

        assert_eq!(
            error,
            ApiError::OAuthError {
                error: "invalid_grant".into(),
                error_description: "nope".into(),
            }
        );
    }

    #[test]
    fn validator_rule_names_become_validation_codes() {
        let mut errors = validator::ValidationErrors::new();
        errors.add("password", validator::ValidationError::new("length"));

        let error = ApiError::from(errors);

        assert_eq!(
            error,
            ApiError::validation_errors(vec![ValidationError::new(
                "password",
                "length",
                "Validation failed on password",
            )])
        );
    }
}
