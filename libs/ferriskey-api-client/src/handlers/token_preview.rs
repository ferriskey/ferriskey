use crate::validators::EvaluateScopesValidator;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_api_core::api_entities::{
    api_error::{ApiError, ApiErrorResponse, ValidateJson},
    response::Response,
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::url::FullUrl;
use ferriskey_api_core::url::root_scoped_base_url;
use ferriskey_core::domain::authentication::value_objects::{
    EvaluateClientScopesRequest, Identity, TokenPreviewResult,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    operation_id = "preview_token",
    summary = "Preview token claims for a client scope set",
    description = "Previews the decoded (unsigned, non-persisted) token claims a client's scope set would produce — access token, ID token and userinfo — plus the active scopes and applied protocol mappers. Requires the `ManageClientScopes` or `ManageRealm` permission. When `user_id` is omitted, user-attribute mappers resolve to placeholder values. Never issues a real token.",
    path = "/{client_id}/token-preview",
    tag = "client",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = EvaluateScopesValidator,
    responses(
        (status = 200, body = TokenPreviewResult, description = "Token preview"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn preview_token(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<EvaluateScopesValidator>,
) -> Result<Response<TokenPreviewResult>, ApiError> {
    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    let result = state
        .service
        .preview_token_claims(
            identity,
            EvaluateClientScopesRequest {
                realm_name,
                client_id,
                base_url,
                user_id: payload.user_id,
                scope: payload.scope,
            },
        )
        .await?;

    Ok(Response::OK(result))
}
