use axum::extract::{Path, State};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::url::{FullUrl, public_base_url, root_scoped_base_url};
use ferriskey_core::domain::realm::ports::RealmService;

use crate::validators::AuthzenConfiguration;

#[utoipa::path(
    get,
    path = "",
    tag = "authzen",
    summary = "Get AuthZEN PDP metadata",
    description = "AuthZEN 1.0 policy decision point metadata for a realm.",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
    ),
    responses(
        (status = 200, description = "PDP metadata", body = AuthzenConfiguration),
        (status = 404, description = "Realm not found", body = ApiErrorResponse),
    ),
)]
pub async fn get_authzen_configuration(
    Path(realm_name): Path<String>,
    State(state): State<AppState>,
    FullUrl(_, base_url): FullUrl,
) -> Result<Response<AuthzenConfiguration>, ApiError> {
    state
        .service
        .get_authorization_server_settings(realm_name.clone())
        .await?;

    let base = public_base_url(state.args.server.public_url.as_deref(), &base_url);
    let root = root_scoped_base_url(&base, &state.args.server.root_path);
    let policy_decision_point = format!("{root}/realms/{realm_name}/authzen");

    Ok(Response::OK(AuthzenConfiguration {
        access_evaluation_endpoint: format!("{policy_decision_point}/access/v1/evaluation"),
        policy_decision_point,
    }))
}
