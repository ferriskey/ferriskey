use axum::extract::{Path, Query, State};
use ferriskey_core::domain::consent::ScopeDescriptor;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;

#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct GetConsentQuery {
    pub consent_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ScopeView {
    pub name: String,
    pub description: Option<String>,
}

impl From<ScopeDescriptor> for ScopeView {
    fn from(value: ScopeDescriptor) -> Self {
        Self {
            name: value.name,
            description: value.description,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct GetConsentResponse {
    pub client_name: String,
    pub granted_by_default: Vec<ScopeView>,
    pub awaiting_decision: Vec<ScopeView>,
}

#[utoipa::path(
    get,
    summary = "Get a pending consent request",
    path = "/auth/consent",
    tag = "consent",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        GetConsentQuery,
    ),
    responses(
        (status = 200, description = "Pending consent request", body = GetConsentResponse),
        (status = 404, description = "Unknown or expired consent token", body = ApiErrorResponse),
    )
)]
pub async fn get_consent(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<GetConsentQuery>,
) -> Result<Response<GetConsentResponse>, ApiError> {
    let view = state
        .service
        .get_consent_request(&realm_name, &query.consent_token)
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(GetConsentResponse {
        client_name: view.client_name,
        granted_by_default: view.default_scopes.into_iter().map(Into::into).collect(),
        awaiting_decision: view.optional_scopes.into_iter().map(Into::into).collect(),
    }))
}
