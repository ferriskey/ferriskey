use std::sync::Arc;

use axum::Extension;
use axum_macros::TypedPath;

use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::errors::{ApiError, ValidateJson};
use crate::application::http::server::handlers::Response;
use crate::domain::realm::{entities::realm::Realm, ports::RealmService};

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
pub async fn create_realm<R: RealmService>(
    _: CreateRealmRoute,
    Extension(realm_service): Extension<Arc<R>>,
    ValidateJson(payload): ValidateJson<CreateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    realm_service
        .create_realm(payload.name)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}
