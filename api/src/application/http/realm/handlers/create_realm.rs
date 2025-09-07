use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use axum::{Extension, extract::State};
use ferriskey_core::domain::realm::ports::{CreateRealmInput, RealmService};
use ferriskey_core::domain::{authentication::value_objects::Identity, realm::entities::Realm};

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
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRealmValidator>,
) -> Result<Response<Realm>, ApiError> {
    let realm = state
        .service
        .create_realm(
            identity,
            CreateRealmInput {
                realm_name: payload.name,
            },
        )
        .await?;

    Ok(Response::Created(realm))
}
