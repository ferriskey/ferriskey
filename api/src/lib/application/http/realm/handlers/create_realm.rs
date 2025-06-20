use axum::Extension;
use axum::extract::State;
use axum_macros::TypedPath;

use crate::application::auth::Identity;
use crate::application::http::realm::policies::RealmPolicy;
use crate::application::http::realm::validators::CreateRealmValidator;
use crate::application::http::server::api_entities::api_error::{ApiError, ValidateJson};
use crate::application::http::server::api_entities::response::Response;
use crate::application::http::server::app_state::AppState;
use crate::domain::realm::{entities::realm::Realm, ports::realm_service::RealmService};
use crate::domain::user::ports::user_service::UserService;

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
    let c = RealmPolicy::create(identity.clone(), state.clone()).await?;

    if !c {
        return Err(ApiError::Forbidden(
            "You do not have permission to create a realm".into(),
        ));
    }

    let user = match identity {
        Identity::User(user) => user,
        Identity::Client(client) => state
            .user_service
            .get_by_client_id(client.id)
            .await
            .map_err(|_| ApiError::Forbidden("Service account not found".to_string()))?,
    };

    state
        .realm_service
        .create_realm_with_user(payload.name, &user)
        .await
        .map_err(ApiError::from)
        .map(Response::Created)
}
