use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode, header::LOCATION, header::SET_COOKIE},
    response::IntoResponse,
};
use axum_extra::extract::cookie::{Cookie, SameSite};

use ferriskey_core::domain::abyss::identity_provider::broker::{
    BrokerCallbackInput, BrokerService,
};
use ferriskey_core::domain::authentication::{
    entities::{ExchangeTokenInput, GrantType},
    ports::AuthService,
};

use crate::application::http::server::{
    api_entities::api_error::{ApiError, ApiErrorResponse},
    app_state::AppState,
};
use crate::application::url::FullUrl;

use super::super::validators::BrokerCallbackQuery;

/// Handles the callback from the external identity provider
///
/// This endpoint validates the state parameter, exchanges the authorization code
/// for tokens, finds or creates the user, and redirects back to the client
/// with an authorization code.
#[utoipa::path(
    post,
    path = "/broker/{alias}/endpoint",
    tag = "broker",
    summary = "Handle SSO callback from identity provider",
    description = "Processes the IdP callback, exchanges code for tokens, and redirects to client",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("alias" = String, Path, description = "Identity provider alias"),
        BrokerCallbackQuery
    ),
    responses(
        (status = 302, description = "Redirect to client with authorization code"),
        (status = 400, description = "Bad request - invalid state or expired session", body = ApiErrorResponse),
        (status = 401, description = "Authentication failed at identity provider", body = ApiErrorResponse),
        (status = 502, description = "Error communicating with identity provider", body = ApiErrorResponse),
    )
)]
pub async fn broker_callback(
    Path((realm_name, alias)): Path<(String, String)>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(params): Query<BrokerCallbackQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let root_scoped_base_url = format!("{base_url}{}", state.args.server.root_path);
    let result = state
        .service
        .handle_callback(BrokerCallbackInput {
            realm_name: realm_name.clone(),
            alias,
            code: params.code,
            state: params.state,
            error: params.error,
            error_description: params.error_description,
            base_url: root_scoped_base_url.clone(),
        })
        .await?;

    if let Ok(jwt_token) = state
        .service
        .exchange_token(ExchangeTokenInput {
            realm_name,
            client_id: result.client_id.clone(),
            client_secret: None,
            code: Some(result.authorization_code.clone()),
            username: None,
            password: None,
            refresh_token: None,
            base_url: root_scoped_base_url,
            grant_type: GrantType::Code,
            scope: None,
        })
        .await
    {
        let mut identity_cookie =
            Cookie::build(("FERRISKEY_IDENTITY", jwt_token.access_token().to_string()))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax);

        if base_url.starts_with("https://") {
            identity_cookie = identity_cookie.secure(true);
        }

        let cookie_value = HeaderValue::from_str(&identity_cookie.to_string())
            .map_err(|_| ApiError::InternalServerError("Invalid cookie header".to_string()))?;

        let response = axum::response::Response::builder()
            .status(StatusCode::FOUND)
            .header(LOCATION, result.redirect_url)
            .header(SET_COOKIE, cookie_value)
            .body(axum::body::Body::empty())
            .map_err(|_| ApiError::InternalServerError("Failed to build response".to_string()))?;

        return Ok(response.into_response());
    }

    Ok((StatusCode::FOUND, [(LOCATION, result.redirect_url)]).into_response())
}
