use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header::LOCATION},
    response::IntoResponse,
};

use ferriskey_core::domain::abyss::identity_provider::broker::{
    BrokerCallbackInput, BrokerService,
};
use ferriskey_core::domain::common::entities::app_errors::CoreError;

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
    let result = match state
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
        .await
    {
        Ok(result) => result,
        Err(CoreError::UserDisabled) => {
            let frontend_origin = state
                .args
                .server
                .allowed_origins
                .first()
                .map(|s| s.trim_end_matches('/').to_string())
                .unwrap_or_else(|| base_url.clone());

            let login_url = format!(
                "{frontend_origin}/realms/{realm_name}/authentication/login?login_error=User+account+is+disabled"
            );
            return Ok((StatusCode::FOUND, [(LOCATION, login_url)]).into_response());
        }
        Err(e) => return Err(e.into()),
    };

    Ok((StatusCode::FOUND, [(LOCATION, result.redirect_url)]).into_response())
}
