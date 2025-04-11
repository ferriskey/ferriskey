use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_macros::TypedPath;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    application::http::server::{api_entities::api_error::ApiError, app_state::AppState},
    domain::{
        authentication::ports::auth_session::AuthSessionService,
        client::ports::client_service::ClientService, realm::ports::realm_service::RealmService,
    },
};

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct AuthRequest {
    #[validate(length(min = 1, message = "response_type is required"))]
    #[serde(default)]
    pub response_type: String,
    #[validate(length(min = 1, message = "client_id is required"))]
    #[serde(default)]
    pub client_id: String,
    #[validate(length(min = 1, message = "redirect_uri is required"))]
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/protocol/openid-connect/auth")]
pub struct AuthRoute {
    pub realm_name: String,
}

pub async fn auth(
    AuthRoute { realm_name }: AuthRoute,
    State(state): State<AppState>,
    Query(params): Query<AuthRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(|_| ApiError::InternalServerError("".to_string()))?;

    let client = state
        .client_service
        .get_by_client_id(params.client_id, realm.id)
        .await
        .map_err(|_| ApiError::InternalServerError("".to_string()))?;

    // @todo: verify redirect_uri

    let params_state = params.state.clone();
    let redirect_uri = params.redirect_uri.clone();

    let _ = state
        .auth_session_service
        .create_session(
            realm.id,
            client.id,
            redirect_uri,
            params.response_type,
            params.scope.unwrap_or_default(),
            params_state,
            None,
            None,
        )
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let login_url = format!(
        "http://localhost:5173/realms/{}/authentication/login?client_id={}&redirect_uri={}&state={}",
        realm.name,
        client.client_id,
        params.redirect_uri,
        params.state.unwrap_or_default()
    );

    Ok(Redirect::to(&login_url))
}
