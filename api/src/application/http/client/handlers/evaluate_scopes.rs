use crate::application::http::authentication::handlers::auth::root_scoped_base_url;
use crate::application::http::client::validators::EvaluateScopesValidator;
use crate::application::http::server::{
    api_entities::{
        api_error::{ApiError, ApiErrorResponse, ValidateJson},
        response::Response,
    },
    app_state::AppState,
};
use crate::application::url::FullUrl;
use axum::{
    Extension,
    extract::{Path, State},
};
use ferriskey_core::domain::authentication::value_objects::{
    EvaluateClientScopesRequest, EvaluateClientScopesResult, Identity,
};
use uuid::Uuid;

#[utoipa::path(
    post,
    operation_id = "evaluate_client_scopes",
    summary = "Evaluate client scopes for a user",
    description = "Previews the effective protocol mappers, roles and token claims (access token, ID token, userinfo) a user would receive from this client for a given scope set — without issuing (signing or persisting) a real token.",
    path = "/{client_id}/evaluate-scopes",
    tag = "client",
    params(
        ("realm_name" = String, Path, description = "Realm name"),
        ("client_id" = Uuid, Path, description = "Client ID"),
    ),
    request_body = EvaluateScopesValidator,
    responses(
        (status = 200, body = EvaluateClientScopesResult, description = "Evaluation preview"),
        (status = 401, description = "Realm not found", body = ApiErrorResponse),
        (status = 403, description = "Insufficient permissions", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse),
    )
)]
pub async fn evaluate_scopes(
    Path((realm_name, client_id)): Path<(String, Uuid)>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    FullUrl(_, base_url): FullUrl,
    ValidateJson(payload): ValidateJson<EvaluateScopesValidator>,
) -> Result<Response<EvaluateClientScopesResult>, ApiError> {
    let base_url = root_scoped_base_url(&base_url, &state.args.server.root_path);

    let result = state
        .service
        .evaluate_client_scopes(
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
