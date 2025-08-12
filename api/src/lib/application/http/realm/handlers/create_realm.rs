use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;
use ferriskey_core::application::realm::use_cases::create_realm_use_case::CreateRealmUseCaseParams;
use ferriskey_core::domain::authentication::value_objects::Identity;
use ferriskey_core::domain::realm::entities::Realm;

#[derive(TypedPath)]
#[typed_path("/realms")]
pub struct CreateRealmRoute;

#[utoipa::path(
    post,
    path = "",
    tag = "realm",
    summary = "Create a new realm",
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
    let realm = state
        .use_case_bundle
        .create_realm_use_case
        .execute(
            identity,
            CreateRealmUseCaseParams {
                realm_name: payload.name,
            },
        )
        .await?;

    Ok(Response::Created(realm))
}
