use super::auth::root_scoped_base_url;
use crate::basic_auth::try_parse_basic_client_credentials;
use crate::validators::TokenRequestValidator;
use axum::{
    Form,
    extract::{Path, State, rejection::FormRejection},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use ferriskey_api_core::api_entities::api_error::ApiError;
use ferriskey_api_core::api_entities::api_error::ApiErrorResponse;
use ferriskey_api_core::api_entities::api_error::OAuth2ErrorResponse;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::request_context::RequestContext;
use ferriskey_api_core::url::FullUrl;
use ferriskey_core::domain::authentication::entities::{GrantType, JwtToken};
use ferriskey_core::domain::authentication::token_exchange::{
    TokenExchangeError, TokenExchangeInput, TokenExchangeOutput,
};
use ferriskey_core::domain::authentication::{entities::ExchangeTokenInput, ports::AuthService};
use ferriskey_core::domain::common::entities::app_errors::CoreError;
use serde::Serialize;
use tracing::{instrument, warn};
use utoipa::ToSchema;

/// The two success bodies of the token endpoint, for the OpenAPI document
/// only: every grant answers with a `JwtToken`, except token exchange, which
/// answers with the RFC 8693 §2.2.1 body.
#[derive(Serialize, ToSchema)]
#[serde(untagged)]
#[allow(dead_code)]
enum TokenResponse {
    Jwt(JwtToken),
    Exchanged(TokenExchangeOutput),
}

/// The two 401 bodies of the token endpoint, for the OpenAPI document only:
/// token exchange answers every 401 with the RFC 6749 §5.2 body
/// (`invalid_client`), the other grants with the FerrisKey error body.
#[derive(Serialize, ToSchema)]
#[serde(untagged)]
#[allow(dead_code)]
enum TokenUnauthorizedResponse {
    Api(ApiErrorResponse),
    OAuth(OAuth2ErrorResponse),
}

#[utoipa::path(
    post,
    path = "/protocol/openid-connect/token",
    tag = "auth",
    summary = "Exchange token",
    description = "Exchanges a token for a JWT token. This endpoint allows clients to exchange various types of tokens (like authorization codes, refresh tokens, etc.) for a JWT token. With `grant_type=urn:ietf:params:oauth:grant-type:token-exchange` (RFC 8693), a confidential client swaps a `subject_token` for a narrower access token and gets the RFC 8693 response instead.",
    request_body = TokenRequestValidator,
    params(
      ("realm_name" = String, Path, description = "Realm name")
    ),
    responses(
        (status = 200, body = TokenResponse),
        (status = 400, description = "RFC 6749 error (`invalid_request`, `invalid_scope`, `invalid_target`, `unauthorized_client`, ...)", body = OAuth2ErrorResponse),
        (status = 401, description = "Realm not found or client authentication failed. Token exchange always answers with the RFC 6749 body (`invalid_client`).", body = TokenUnauthorizedResponse),
        (status = 404, description = "Client not found", body = ApiErrorResponse),
        (status = 500, description = "Internal Server Error", body = ApiErrorResponse),
    )
)]
pub async fn exchange_token(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    context: RequestContext,
    headers: HeaderMap,
    payload: Result<Form<TokenRequestValidator>, FormRejection>,
) -> Response {
    let mut response = match payload {
        Ok(Form(payload)) => token_response(realm_name, state, base_url, context, headers, payload)
            .await
            .unwrap_or_else(IntoResponse::into_response),
        Err(rejection) => malformed_token_request(&rejection.body_text()).into_response(),
    };

    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    response
}

fn malformed_token_request(reason: &str) -> ApiError {
    if reason.contains("grant_type") && reason.contains("unknown variant") {
        return ApiError::OAuthError {
            error: "unsupported_grant_type".into(),
            error_description: "The grant type is not supported.".into(),
        };
    }

    ApiError::OAuthError {
        error: "invalid_request".into(),
        error_description: "The token request is malformed.".into(),
    }
}

fn missing_parameter(payload: &TokenRequestValidator) -> Option<&'static str> {
    match payload.grant_type {
        GrantType::Code if payload.code.is_none() => Some("code"),
        GrantType::Password if payload.username.is_none() => Some("username"),
        GrantType::Password if payload.password.is_none() => Some("password"),
        GrantType::RefreshToken if payload.refresh_token.is_none() => Some("refresh_token"),
        GrantType::DeviceCode if payload.device_code.is_none() => Some("device_code"),
        _ => None,
    }
}

fn token_endpoint_error(error: CoreError) -> ApiError {
    let invalid_grant = |description: String| ApiError::OAuthError {
        error: "invalid_grant".into(),
        error_description: description.into(),
    };

    match error {
        CoreError::ClientAuthenticationFailed
        | CoreError::InvalidClient
        | CoreError::InvalidClientSecret
        | CoreError::ClientNotFound => ApiError::OAuthUnauthorized {
            error: "invalid_client".into(),
            error_description: "Client authentication failed.".into(),
        },
        CoreError::ServiceAccountNotFound => ApiError::OAuthError {
            error: "unauthorized_client".into(),
            error_description: "The client has no service account.".into(),
        },
        CoreError::Invalid
        | CoreError::InvalidPassword
        | CoreError::InvalidCredentials
        | CoreError::InvalidUser
        | CoreError::UserNotFound => invalid_grant("Invalid user credentials.".to_string()),
        CoreError::Forbidden(description) => invalid_grant(description),
        CoreError::UserDisabled
        | CoreError::AccountLocked
        | CoreError::InvalidRefreshToken
        | CoreError::InvalidToken
        | CoreError::ExpiredToken
        | CoreError::SessionRevoked
        | CoreError::SessionExpired
        | CoreError::SessionNotFound
        | CoreError::InvalidSession
        | CoreError::MissingAuthorizationCode => invalid_grant(error.to_string()),
        CoreError::TokenValidationError(_) => {
            invalid_grant("The token is invalid or expired.".to_string())
        }
        CoreError::InvalidRequest => ApiError::OAuthError {
            error: "invalid_request".into(),
            error_description: "The token request is invalid.".into(),
        },
        other => other.into(),
    }
}

#[instrument(
    skip(state, payload, headers, context),
    fields(
        realm_name = %realm_name,
        grant_type = ?payload.grant_type,
        has_username = payload.username.is_some(),
        has_password = payload.password.is_some(),
        has_code = payload.code.is_some(),
        has_refresh_token = payload.refresh_token.is_some()
    )
)]
async fn token_response(
    realm_name: String,
    state: AppState,
    base_url: String,
    context: RequestContext,
    headers: HeaderMap,
    payload: TokenRequestValidator,
) -> Result<Response, ApiError> {
    let (client_id, client_secret) = match try_parse_basic_client_credentials(&headers) {
        Some((id, sec)) => (id, Some(sec)),
        None => (
            payload.client_id.clone().unwrap_or_default(),
            payload.client_secret.clone(),
        ),
    };

    // RFC 8693 has its own service and error codes; it never goes through
    // `exchange_token`, which would also open a second Compass flow.
    if payload.grant_type == GrantType::TokenExchange {
        return exchange_subject_token(
            &state,
            realm_name,
            client_id,
            client_secret,
            payload,
            context.ip_address,
            context.user_agent,
        )
        .await
        .map(IntoResponse::into_response);
    }

    if let Some(parameter) = missing_parameter(&payload) {
        return Err(ApiError::OAuthError {
            error: "invalid_request".into(),
            error_description: format!(
                "The {parameter} parameter is required for this grant type."
            )
            .into(),
        });
    }

    let grant_type = payload.grant_type.clone();
    let has_client_secret = client_secret.is_some();
    let has_username = payload.username.is_some();
    let has_password = payload.password.is_some();
    let has_code = payload.code.is_some();
    let has_refresh_token = payload.refresh_token.is_some();

    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    let exchange_input = ExchangeTokenInput {
        realm_name,
        client_id: client_id.clone(),
        client_secret,
        code: payload.code,
        username: payload.username,
        password: payload.password,
        refresh_token: payload.refresh_token,
        base_url,
        grant_type: payload.grant_type.clone(),
        scope: payload.scope,
        device_code: payload.device_code,
        code_verifier: payload.code_verifier,
        redirect_uri: payload.redirect_uri,
        ip_address: context.ip_address,
        user_agent: context.user_agent,
    };

    // The device_code grant is served by the device flow polling path so its
    // RFC 8628 §3.5 error codes survive as an RFC 6749 §5.2 error response.
    let token = if payload.grant_type == GrantType::DeviceCode {
        match state.service.poll_device_token(exchange_input).await {
            Ok(token) => token,
            Err(error) => {
                warn!(client_id = %client_id, error = ?error, "Device token poll failed");
                return Err(error.into());
            }
        }
    } else {
        match state.service.exchange_token(exchange_input).await {
            Ok(token) => token,
            Err(error) => {
                warn!(
                    client_id = %client_id,
                    grant_type = ?grant_type,
                    has_client_secret,
                    has_username,
                    has_password,
                    has_code,
                    has_refresh_token,
                    error = ?error,
                    "Token exchange failed"
                );
                return Err(token_endpoint_error(error));
            }
        }
    };

    // No cookie: a bearer token in a response header lands in every proxy log
    // that records headers, and the browser's SSO rides on `FERRISKEY_SSO`.
    Ok((StatusCode::OK, axum::Json(token)).into_response())
}

/// The token-exchange grant (RFC 8693 §2). Answers with the §2.2.1 body, no
/// cookie, and `Cache-Control: no-store` as RFC 6749 §5.1 requires.
async fn exchange_subject_token(
    state: &AppState,
    realm_name: String,
    client_id: String,
    client_secret: Option<String>,
    payload: TokenRequestValidator,
    ip_address: Option<String>,
    user_agent: Option<String>,
) -> Result<impl IntoResponse, ApiError> {
    let (Some(subject_token), Some(subject_token_type)) =
        (payload.subject_token, payload.subject_token_type)
    else {
        return Err(TokenExchangeError::InvalidRequest.into());
    };

    let input = TokenExchangeInput {
        subject_token,
        subject_token_type,
        requested_token_type: payload.requested_token_type,
        audience: payload.audience,
        resource: payload.resource,
        scope: payload.scope,
        actor_token: payload.actor_token,
        actor_token_type: payload.actor_token_type,
        ip_address,
        user_agent,
    };

    let output = state
        .service
        .exchange_subject_token(realm_name, client_id.clone(), client_secret, input)
        .await
        .inspect_err(|error| {
            warn!(client_id = %client_id, error = ?error, "Subject token exchange failed");
        })?;

    Ok((
        StatusCode::OK,
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::PRAGMA, "no-cache"),
        ],
        axum::Json(output),
    ))
}

#[cfg(test)]
mod tests {
    use super::{malformed_token_request, missing_parameter, token_endpoint_error};
    use crate::validators::TokenRequestValidator;
    use ferriskey_api_core::api_entities::api_error::ApiError;
    use ferriskey_core::domain::common::entities::app_errors::CoreError;

    fn oauth_code(error: ApiError) -> (u16, String) {
        match error {
            ApiError::OAuthError { error, .. } => (400, error.into_owned()),
            ApiError::OAuthUnauthorized { error, .. } => (401, error.into_owned()),
            other => panic!("not an OAuth error: {other:?}"),
        }
    }

    fn request(grant_type: &str) -> TokenRequestValidator {
        serde_urlencoded::from_str(&format!("grant_type={grant_type}")).expect("form")
    }

    #[test]
    fn client_failures_are_invalid_client() {
        for error in [
            CoreError::ClientAuthenticationFailed,
            CoreError::InvalidClient,
            CoreError::InvalidClientSecret,
            CoreError::ClientNotFound,
        ] {
            assert_eq!(
                oauth_code(token_endpoint_error(error)),
                (401, "invalid_client".to_string())
            );
        }
    }

    #[test]
    fn grant_failures_are_invalid_grant() {
        for error in [
            CoreError::Invalid,
            CoreError::InvalidPassword,
            CoreError::UserDisabled,
            CoreError::AccountLocked,
            CoreError::InvalidRefreshToken,
            CoreError::InvalidToken,
            CoreError::ExpiredToken,
            CoreError::SessionRevoked,
            CoreError::Forbidden("a required action is pending".to_string()),
            CoreError::InvalidAuthorizationCode,
        ] {
            assert_eq!(
                oauth_code(token_endpoint_error(error)),
                (400, "invalid_grant".to_string())
            );
        }
    }

    #[test]
    fn a_token_validation_failure_is_an_invalid_grant_without_its_detail() {
        let error = token_endpoint_error(CoreError::TokenValidationError(
            "ExpiredSignature: token expired".to_string(),
        ));
        let ApiError::OAuthError {
            ref error_description,
            ..
        } = error
        else {
            panic!("expected an OAuth error");
        };
        assert!(!error_description.contains("ExpiredSignature"));
        assert_eq!(oauth_code(error), (400, "invalid_grant".to_string()));
    }

    #[test]
    fn credential_failures_do_not_say_which_part_was_wrong() {
        let ApiError::OAuthError {
            error_description, ..
        } = token_endpoint_error(CoreError::UserNotFound)
        else {
            panic!("expected an OAuth error");
        };
        assert_eq!(error_description, "Invalid user credentials.");
    }

    #[tokio::test]
    async fn an_unknown_grant_type_is_unsupported() {
        use axum::{Form, extract::FromRequest, http::Request};

        let request = Request::post("/token")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(axum::body::Body::from("grant_type=made-up"))
            .expect("request");
        let rejection = Form::<TokenRequestValidator>::from_request(request, &())
            .await
            .expect_err("unknown grant type");

        assert_eq!(
            oauth_code(malformed_token_request(&rejection.body_text())),
            (400, "unsupported_grant_type".to_string())
        );
    }

    #[test]
    fn any_other_malformed_body_is_an_invalid_request() {
        assert_eq!(
            oauth_code(malformed_token_request("missing field `foo`")),
            (400, "invalid_request".to_string())
        );
    }

    #[test]
    fn each_grant_names_its_missing_parameter() {
        assert_eq!(
            missing_parameter(&request("authorization_code")),
            Some("code")
        );
        assert_eq!(missing_parameter(&request("password")), Some("username"));
        assert_eq!(
            missing_parameter(&request("refresh_token")),
            Some("refresh_token")
        );
        assert_eq!(
            missing_parameter(&request("urn:ietf:params:oauth:grant-type:device_code")),
            Some("device_code")
        );
        assert_eq!(missing_parameter(&request("client_credentials")), None);
    }
}
