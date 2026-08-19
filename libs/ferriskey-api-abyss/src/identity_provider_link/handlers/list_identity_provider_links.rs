use crate::identity_provider_link::dto::{
    IdentityProviderLinkResponse, IdentityProviderLinksResponse,
};
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
    entities::ListIdentityProviderLinksInput, ports::IdentityProviderService,
};
use ferriskey_core::domain::authentication::value_objects::Identity;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/users/{user_id}/identity-provider-links",
    summary = "List the identity provider links of a user",
    description = "Returns every external identity provider account currently linked to the given user in the realm. Provider secrets and stored tokens are never returned. Requires the same permission as updating a user.",
    responses(
        (status = 200, body = IdentityProviderLinksResponse, description = "Identity provider links of the user"),
        (status = 401, description = "Unauthorized", body = ApiErrorResponse),
        (status = 403, description = "Forbidden", body = ApiErrorResponse),
        (status = 404, description = "Realm or user not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    ),
    params(
        ("realm_name" = String, Path, description = "The name of the realm"),
        ("user_id" = Uuid, Path, description = "The user whose links are listed"),
    ),
    tag = "identity_provider",
)]
pub async fn list_identity_provider_links(
    Path((realm_name, user_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<IdentityProviderLinksResponse>, ApiError> {
    let links = state
        .service
        .list_identity_provider_links(
            identity,
            ListIdentityProviderLinksInput {
                realm_name,
                user_id,
            },
        )
        .await?
        .into_iter()
        .map(IdentityProviderLinkResponse::from)
        .collect::<Vec<IdentityProviderLinkResponse>>();

    Ok(Response::OK(IdentityProviderLinksResponse { data: links }))
}
