use super::handlers::evaluate::{__path_evaluate, evaluate};
use super::handlers::metadata::{__path_get_authzen_configuration, get_authzen_configuration};
use axum::{
    Router,
    extract::{DefaultBodyLimit, Request},
    http::HeaderName,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use ferriskey_api_core::app_state::AppState;
use ferriskey_api_core::auth::auth;
use utoipa::OpenApi;

const BODY_LIMIT: usize = 64 * 1024;
const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(OpenApi)]
#[openapi(paths(evaluate))]
pub struct AuthzApiDoc;

#[derive(OpenApi)]
#[openapi(paths(get_authzen_configuration))]
pub struct AuthzMetadataApiDoc;

async fn echo_request_id(request: Request, next: Next) -> Response {
    let request_id = request.headers().get(&REQUEST_ID).cloned();
    let mut response = next.run(request).await;
    if let Some(value) = request_id {
        response.headers_mut().insert(REQUEST_ID, value);
    }
    response
}

pub fn authz_routes(state: AppState) -> Router<AppState> {
    let root_path = &state.args.server.root_path;

    let evaluation = Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/authzen/access/v1/evaluation"),
            post(evaluate),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(middleware::from_fn(echo_request_id));

    let metadata = Router::new().route(
        &format!("/.well-known/authzen-configuration{root_path}/realms/{{realm_name}}/authzen"),
        get(get_authzen_configuration),
    );

    evaluation.merge(metadata)
}
