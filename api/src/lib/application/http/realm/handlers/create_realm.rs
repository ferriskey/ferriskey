use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use tracing::info;

use crate::application::auth::Identity;
use crate::application::http::realm::policies::RealmPolicy;
use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};

#[derive(TypedPath)]
#[typed_path("/realms")]
pub struct CreateRealmRoute;

#[utoipa::path(
    post,
    path = "",
    tag = "realm",
    responses(
        (status = 201, body = Realm)
    ),
    request_body = CreateRealmValidator
)]
pub async fn create_realm(
    _: CreateRealmRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    let c = RealmPolicy::create(identity, state.clone()).await?;

    if !c {
        return Err(ApiError::Forbidden(
            "You do not have permission to create a realm".into(),
        ));
    }
    state
        .realm_service
        .create_realm(payload.name)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}
