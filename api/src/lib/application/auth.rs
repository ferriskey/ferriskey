use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts, Request, State},
    http::{StatusCode, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::client::ports::ClientService;
use ferriskey_core::domain::jwt::entities::{ClaimsTyp, JwtClaim};
use ferriskey_core::domain::jwt::ports::JwtService;
use ferriskey_core::domain::user::ports::UserService;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use super::http::server::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    claims: JwtClaim,
    token: String,
}

#[derive(Debug, Error, Deserialize, Serialize, PartialEq, Eq)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token not found")]
    TokenNotFound,
    #[error("Invalid signature")]
    InvalidSignature,
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
    status: i64,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            AuthError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "E_UNAUTHORIZED", "Invalid token")
            }
            AuthError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "E_UNAUTHORIZED", "Token expired")
            }
            AuthError::TokenNotFound => (
                StatusCode::UNAUTHORIZED,
                "E_UNAUTHORIZED",
                "Token not found",
            ),
            AuthError::InvalidSignature => (
                StatusCode::UNAUTHORIZED,
                "E_UNAUTHORIZED",
                "Invalid signature",
            ),
        };

        let error_response = ErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
            status: status.as_u16() as i64,
        };

        let body = serde_json::to_string(&error_response).unwrap_or_else(|_| {
            r#"{"code":"INTERNAL_SERVER_ERROR","message":"Failed to serialize error response"}"#
                .to_string()
        });

        axum::response::Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(body.clone().into())
            .unwrap_or_else(|_| axum::response::Response::new(body.clone().into()))
    }
}

impl<S> FromRequestParts<S> for Jwt
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_token_from_bearer(parts).await?;

        let t: Vec<&str> = token.split('.').collect();
        if t.len() != 3 {
            return Err(AuthError::InvalidToken);
        }

        let payload = t[1];

        let decoded = general_purpose::URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|e| {
                tracing::error!("Failed to decode JWT payload: {:?}", e);
                AuthError::InvalidToken
            })?;

        let payload_str = String::from_utf8(decoded).map_err(|e| {
            tracing::error!("Failed to decode JWT payload: {:?}", e);
            AuthError::InvalidToken
        })?;
        let claims: JwtClaim = serde_json::from_str(&payload_str).map_err(|e| {
            tracing::error!("Failed to deserialize JWT claims: {:?}", e);
            AuthError::InvalidToken
        })?;

        Ok(Jwt {
            claims,
            token: token.clone(),
        })
    }
}

pub async fn extract_token_from_bearer(parts: &mut Parts) -> Result<String, AuthError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::TokenNotFound)?;

    Ok(bearer.token().to_string())
}

pub async fn auth(
    State(state): State<AppState>,
    jwt: Jwt,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = jwt.claims;

    if claims.typ != ClaimsTyp::Bearer {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = state
        .service_bundle
        .user_service
        .get_by_id(claims.sub)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let _ = state
        .service_bundle
        .jwt_service
        .verify_token(jwt.token, user.realm_id)
        .await
        .map_err(|e| {
            tracing::error!("JWT verification failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    let identity: Identity = match claims.is_service_account() {
        true => {
            let client_id = match claims.client_id {
                Some(client_id) => client_id,
                None => return Err(StatusCode::UNAUTHORIZED),
            };

            let client_id = Uuid::parse_str(&client_id).map_err(|_| StatusCode::UNAUTHORIZED)?;
            let client = state
                .service_bundle
                .client_service
                .get_by_id(client_id)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

            Identity::Client(client)
        }
        false => Identity::User(user),
    };

    req.extensions_mut().insert(identity);

    Ok(next.run(req).await)
}
