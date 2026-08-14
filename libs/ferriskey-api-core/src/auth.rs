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
use ferriskey_core::domain::authentication::{entities::AuthorizeRequestInput, ports::AuthService};
use ferriskey_core::domain::jwt::entities::JwtClaim;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use axum::extract::Path;

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct Jwt {
    pub(crate) claims: JwtClaim,
    pub(crate) token: String,
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

/// The realm name carried by the authenticated bearer token, extracted from the
/// JWT `iss` claim (`{base_url}/realms/{realm_name}`). Self-service handlers use
/// it to reject requests whose URL realm does not match the token's realm.
#[derive(Clone, Debug)]
pub struct AuthenticatedRealm(pub String);

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
        decode_jwt(token)
    }
}

pub const LOGIN_ACTION_COOKIE: &str = "FERRISKEY_LOGIN_ACTION";

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginActionJwt {
    pub(crate) claims: JwtClaim,
    pub(crate) token: String,
}

impl<S> FromRequestParts<S> for LoginActionJwt
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _: &S,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|raw| {
                raw.split(';').find_map(|part| {
                    let (name, value) = part.trim().split_once('=')?;
                    (name == LOGIN_ACTION_COOKIE).then(|| value.to_string())
                })
            })
            .ok_or(AuthError::TokenNotFound)?;

        let Jwt { claims, token } = decode_jwt(token)?;
        Ok(LoginActionJwt { claims, token })
    }
}

fn decode_jwt(token: String) -> Result<Jwt, AuthError> {
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

pub async fn extract_token_from_bearer(parts: &mut Parts) -> Result<String, AuthError> {
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::TokenNotFound)?;

    Ok(bearer.token().to_string())
}

#[tracing::instrument(skip(state, jwt, req, next), fields(claims.sub = %jwt.claims.sub))]
pub async fn auth(
    State(state): State<AppState>,
    jwt: Jwt,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = jwt.claims;
    let realm_from_iss = realm_name_from_iss(&claims.iss);

    let output = state
        .service
        .authorize_request(AuthorizeRequestInput {
            claims,
            token: jwt.token,
            realm_name: None,
        })
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(output.identity);
    let realm = realm_from_iss.ok_or(StatusCode::UNAUTHORIZED)?;
    req.extensions_mut().insert(AuthenticatedRealm(realm));

    Ok(next.run(req).await)
}

const STEP_COMPLETING_ACTIONS: [&str; 4] = [
    "/login-actions/verify-otp",
    "/login-actions/challenge-otp",
    "/login-actions/update-password",
    "/login-actions/webauthn-public-key-create",
];
/// Extracts the realm name from an issuer claim of the form
/// `{base_url}/realms/{realm_name}`. Returns `None` when the claim does not
/// follow that shape; the authentication middleware rejects such requests
/// with `401 UNAUTHORIZED` rather than proceeding without a realm.
fn realm_name_from_iss(iss: &str) -> Option<String> {
    let marker = "/realms/";
    let idx = iss.find(marker)?;
    let realm = &iss[idx + marker.len()..];
    let realm = realm.split(['/', '?']).next().unwrap_or(realm);
    if realm.is_empty() {
        None
    } else {
        Some(realm.to_string())
    }
}
pub async fn auth_login_actions(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    jwt: LoginActionJwt,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = jwt.claims;
    let jti = claims.jti;

    let output = state
        .service
        .authorize_login_action_request(AuthorizeRequestInput {
            claims,
            token: jwt.token,
            realm_name: Some(realm_name),
        })
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(output.identity);

    let completes_step = STEP_COMPLETING_ACTIONS
        .iter()
        .any(|suffix| req.uri().path().ends_with(suffix));

    let response = next.run(req).await;

    if completes_step && response.status().is_success() {
        state.service.consume_login_action_token(jti).await;
    }

    Ok(response)
}
