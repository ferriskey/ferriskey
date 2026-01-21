use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header::LOCATION},
    response::IntoResponse,
};

use ferriskey_core::domain::identity_provider::broker::{BrokerLoginInput, BrokerService};

use crate::application::http::server::{api_entities::api_error::ApiError, app_state::AppState};
use crate::application::url::FullUrl;

use super::super::validators::BrokerLoginRequest;

/// Initiates SSO login via an external identity provider
///
/// This endpoint validates the request, looks up the identity provider,
/// creates a broker session, and redirects the user to the IdP's authorization URL.
#[utoipa::path(
    get,
    path = "/broker/{alias}/login",
    tag = "broker",
    summary = "Initiate SSO login via identity provider",
    description = "Redirects the user to the external identity provider for authentication",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("alias" = String, Path, description = "Identity provider alias"),
        BrokerLoginRequest
    ),
    responses(
        (status = 302, description = "Redirect to identity provider authorization URL"),
        (status = 400, description = "Bad request - invalid parameters"),
        (status = 404, description = "Identity provider not found"),
    )
)]
pub async fn broker_login(
    Path((realm_name, alias)): Path<(String, String)>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
    Query(params): Query<BrokerLoginRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let result = state
        .service
        .broker_service
        .initiate_login(BrokerLoginInput {
            realm_name,
            alias,
            client_id: params.client_id,
            redirect_uri: params.redirect_uri,
            response_type: params.response_type,
            scope: params.scope,
            state: params.state,
            nonce: params.nonce,
            auth_session_id: params.session_id,
            base_url,
        })
        .await?;

    Ok((StatusCode::FOUND, [(LOCATION, result.authorization_url)]))
}
