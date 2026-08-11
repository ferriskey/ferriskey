use crate::validators::CreateRealmValidator;
use axum::{Extension, extract::State};
use ferriskey_api_core::api_entities::api_error::{ApiError, ApiErrorResponse, ValidateJson};
use ferriskey_api_core::api_entities::response::Response;
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::realm::ports::{CreateRealmInput, RealmService};
use ferriskey_core::domain::{authentication::value_objects::Identity, realm::entities::Realm};

#[utoipa::path(
    post,
    path = "",
    tag = "realm",
    summary = "Create a new realm",
    request_body = CreateRealmValidator,
    responses(
        (status = 201, description = "Realm created successfully", body = Realm),
        (status = 400, description = "Invalid request data", body = ApiErrorResponse),
        (status = 401, description = "Realm Master not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
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
                display_name: payload.display_name,
            },
        )
        .await?;

    Ok(Response::Created(realm))
}
