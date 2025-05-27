use axum::{Extension, extract::State};
use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    application::{
        auth::Identity,
        http::{
            role::{policies::RolePolicy, validators::CreateRoleValidator},
            server::{
                api_entities::{
                    api_error::{ApiError, ValidateJson},
                    response::Response,
                },
                app_state::AppState,
            },
        },
    },
    domain::{
        realm::ports::realm_service::RealmService,
        role::{entities::models::Role, ports::RoleService},
    },
};

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/roles")]
pub struct CreateRoleRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[utoipa::path(
    post,
    summary = "Create a new role",
    path = "",
    tag = "role",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = CreateRoleValidator,
    responses(
        (status = 201, body = Role)
    )
)]
pub async fn create_role(
    CreateRoleRoute {
        realm_name,
        client_id,
    }: CreateRoleRoute,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    ValidateJson(payload): ValidateJson<CreateRoleValidator>,
) -> Result<Response<Role>, ApiError> {
    let realm = state
        .realm_service
        .get_by_name(realm_name)
        .await
        .map_err(ApiError::from)?;

    let payload = payload.to_dto(realm.id, Some(client_id));

    let has_permission = RolePolicy::create(identity, state.clone(), realm).await?;

    if !has_permission {
        return Err(ApiError::Forbidden(
            "User not allowed to create role".to_string(),
        ));
    }

    let role = state
        .role_service
        .create(payload)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(Response::Created(role))
}
