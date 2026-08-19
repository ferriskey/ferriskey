use crate::identity_provider_link::dto::DeleteIdentityProviderLinkResponse;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_core::domain::abyss::identity_provider::{
    entities::DeleteIdentityProviderLinkInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    delete,
    path = "/users/{user_id}/identity-provider-links/{link_id}",
    summary = "Sever an identity provider link of a user",
    description = "Removes the link between the given user and an external identity provider account. The user can no longer authenticate through that provider until the account is linked again. Requires the same permission as updating a user.",
    responses(
        (status = 200, body = DeleteIdentityProviderLinkResponse, description = "Identity provider link severed successfully"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Realm, user or link not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ("user_id" = Uuid, Path, description = "The user holding the link"),
        ("link_id" = Uuid, Path, description = "The identity provider link to sever"),
    ),
    tag = "identity_provider",
)]
pub async fn delete_identity_provider_link(
    Path((realm_name, user_id, link_id)): Path<(String, Uuid, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DeleteIdentityProviderLinkResponse>, ApiError> {
    state
        .service
        .delete_identity_provider_link(
            identity,
            DeleteIdentityProviderLinkInput {
                realm_name,
                user_id,
                link_id,
            },
        )
        .await?;

    Ok(Response::OK(DeleteIdentityProviderLinkResponse {
        count: 1,
    }))
}
