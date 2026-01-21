use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header::LOCATION},
    response::IntoResponse,
};

use ferriskey_core::domain::identity_provider::broker::{BrokerCallbackInput, BrokerService};

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};
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
        (status = 400, description = "Bad request - invalid state or expired session"),
        (status = 401, description = "Authentication failed at identity provider"),
        (status = 502, description = "Error communicating with identity provider"),
    )
)]
pub async fn broker_callback(
    Path((realm_name, alias)): Path<(String, String)>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(params): Query<BrokerCallbackQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .service
        .broker_service
        .handle_callback(BrokerCallbackInput {
            realm_name,
            alias,
            code: params.code,
            state: params.state,
            error: params.error,
            error_description: params.error_description,
            base_url,
        })
        .await?;

    Ok((StatusCode::FOUND, [(LOCATION, result.redirect_url)]))
}
