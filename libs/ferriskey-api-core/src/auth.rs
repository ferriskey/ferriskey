use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts, Request, State},
    http::{StatusCode, header::SET_COOKIE, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};
use base64::{Engine, engine::general_purpose};
use ferriskey_core::domain::authentication::{
    entities::{AuthorizeRequestInput, TokenUsage},
    ports::AuthService,
    services::{LOGIN_ACTION_SESSION_CLAIM, second_factor_verified},
};
use ferriskey_core::domain::jwt::entities::JwtClaim;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use axum::extract::Path;

use crate::api_entities::api_error::{ApiError, ApiErrorBody};
use crate::app_state::AppState;
use crate::login_step::{
    LOGIN_ACTION_COOKIE, LoginActionStep, LoginContinues, login_action_cookie,
};
use crate::url::FullUrl;

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

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (message, reason) = match self {
            AuthError::InvalidToken => ("Invalid token", "invalid_token"),
            AuthError::TokenExpired => ("Token expired", "expired_token"),
            AuthError::TokenNotFound => ("Token not found", "token_not_found"),
            AuthError::InvalidSignature => ("Invalid signature", "invalid_signature"),
        };

        ApiError::Unauthorized(ApiErrorBody::new(message, reason)).into_response()
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

async fn authorize(
    state: AppState,
    jwt: Jwt,
    mut req: Request,
    next: Next,
    usage: TokenUsage,
) -> Result<Response, StatusCode> {
    let claims = jwt.claims;

    let output = state
        .service
        .authorize_request(AuthorizeRequestInput {
            claims,
            token: jwt.token,
            realm_name: None,
            usage,
        })
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    req.extensions_mut().insert(output.identity);

    Ok(next.run(req).await)
}

#[tracing::instrument(skip(state, jwt, req, next), fields(claims.sub = %jwt.claims.sub))]
pub async fn auth(
    State(state): State<AppState>,
    jwt: Jwt,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    authorize(state, jwt, req, next, TokenUsage::ManagementApi).await
}

#[tracing::instrument(skip(state, jwt, req, next), fields(claims.sub = %jwt.claims.sub))]
pub async fn auth_any_client(
    State(state): State<AppState>,
    jwt: Jwt,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    authorize(state, jwt, req, next, TokenUsage::AnyClient).await
}

const AUTH_SESSION_COOKIE: &str = "FERRISKEY_SESSION";

const STEP_COMPLETING_ACTIONS: [&str; 5] = [
    "/login-actions/verify-otp",
    "/login-actions/challenge-otp",
    "/login-actions/update-password",
    "/login-actions/webauthn-public-key-create",
    "/login-actions/burn-recovery-code",
];

/// The only steps open to a user who has an OTP and has not answered it yet.
const SECOND_FACTOR_ACTIONS: [&str; 2] = [
    "/login-actions/challenge-otp",
    "/login-actions/burn-recovery-code",
];

/// Steps whose success proves the second factor for the rest of the login.
const SECOND_FACTOR_PROOFS: [&str; 3] = [
    "/login-actions/challenge-otp",
    "/login-actions/burn-recovery-code",
    "/login-actions/verify-otp",
];

fn path_is_one_of(path: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| path.ends_with(suffix))
}

pub async fn auth_login_actions(
    State(state): State<AppState>,
    Path(realm_name): Path<String>,
    FullUrl(_, base_url): FullUrl,
    jwt: LoginActionJwt,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let claims = jwt.claims;
    let jti = claims.jti;
    let previous_claims = claims.clone();
    let token_session = claims
        .additional_claims
        .get(LOGIN_ACTION_SESSION_CLAIM)
        .and_then(|value| value.as_str())
        .and_then(|raw| Uuid::parse_str(raw).ok());

    let cookie_session = CookieJar::from_headers(req.headers())
        .get(AUTH_SESSION_COOKIE)
        .map(|cookie| Uuid::parse_str(cookie.value()).ok());

    if cookie_session.is_some_and(|session| session != token_session) {
        tracing::warn!(
            "Refusing a login action: the session cookie belongs to another login than the step token"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    let grant = state
        .service
        .authorize_login_action_request(AuthorizeRequestInput {
            claims,
            token: jwt.token,
            realm_name: Some(realm_name.clone()),
            usage: TokenUsage::AnyClient,
        })
        .await
        .map_err(|error| {
            tracing::warn!(%error, "Refusing a login action: the step token was not accepted");
            StatusCode::UNAUTHORIZED
        })?;

    let path = req.uri().path().to_string();

    if grant.second_factor_pending && !path_is_one_of(&path, &SECOND_FACTOR_ACTIONS) {
        tracing::warn!(
            %path,
            "Refusing a login action: the second factor of this login is still due"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    req.extensions_mut().insert(grant.identity);
    req.extensions_mut().insert(LoginActionStep {
        auth_session_id: grant.auth_session_id,
        second_factor_verified: !grant.second_factor_pending,
    });

    let mut response = next.run(req).await;

    if !path_is_one_of(&path, &STEP_COMPLETING_ACTIONS) || !response.status().is_success() {
        return Ok(response);
    }

    // Mint the next token before burning this one: if minting fails, the browser
    // keeps a token it can still use instead of none at all.
    let follow_up = match response.extensions().get::<LoginContinues>() {
        Some(_) => {
            let second_factor = second_factor_verified(&previous_claims)
                || path_is_one_of(&path, &SECOND_FACTOR_PROOFS);

            let token = state
                .service
                .issue_follow_up_login_action_token(previous_claims, second_factor)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "Failed to issue the step token for the next login step");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            Some(
                login_action_cookie(
                    token,
                    &state.args.server.root_path,
                    &realm_name,
                    base_url.starts_with("https"),
                )
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            )
        }
        None => None,
    };

    if !state.service.consume_login_action_token(jti).await {
        tracing::warn!(
            %path,
            "Refusing a login action: its step token was spent by a concurrent request"
        );
        return Err(StatusCode::CONFLICT);
    }

    if let Some(cookie) = follow_up {
        response.headers_mut().append(SET_COOKIE, cookie);
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_entities::api_error::serialized_error;
    use serde_json::json;

    #[tokio::test]
    async fn the_middleware_emits_the_same_shape_as_the_handlers() {
        let from_middleware = serialized_error(AuthError::InvalidToken.into_response()).await;
        let from_handler = serialized_error(
            ApiError::Unauthorized(ApiErrorBody::new("Invalid token", "invalid_token"))
                .into_response(),
        )
        .await;

        assert_eq!(from_middleware, from_handler);
        assert_eq!(
            from_middleware,
            json!({
                "code": "E_UNAUTHORIZED",
                "status": 401,
                "reason": "invalid_token",
                "message": "Invalid token",
            })
        );
        assert!(from_middleware["status"].is_u64());
    }

    #[tokio::test]
    async fn every_middleware_rejection_carries_its_own_reason() {
        let cases = [
            (AuthError::InvalidToken, "invalid_token"),
            (AuthError::TokenExpired, "expired_token"),
            (AuthError::TokenNotFound, "token_not_found"),
            (AuthError::InvalidSignature, "invalid_signature"),
        ];

        for (error, expected) in cases {
            let body = serialized_error(error.into_response()).await;

            assert_eq!(body["reason"], json!(expected));
            assert_eq!(body["code"], json!("E_UNAUTHORIZED"));
            assert_eq!(body["status"], json!(401));
            assert!(body["message"].as_str().is_some_and(|m| !m.is_empty()));
        }
    }
}
