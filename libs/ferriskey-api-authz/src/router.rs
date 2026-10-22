use super::handlers::evaluate::{__path_evaluate, evaluate};
use super::handlers::evaluations::{__path_evaluations, evaluations};
use super::handlers::metadata::{__path_get_authzen_configuration, get_authzen_configuration};
use super::handlers::policies::{
    __path_create_policy, __path_delete_policy, __path_get_policy, __path_list_policies,
    __path_update_policy, __path_validate_policies, create_policy, delete_policy, get_policy,
    list_policies, update_policy, validate_policies,
};
use super::handlers::schema::{
    __path_get_builtin_schema, __path_get_schema, __path_put_schema, get_builtin_schema,
    get_schema, put_schema,
};
use super::handlers::simulate::{__path_simulate, simulate};
use super::handlers::state::{__path_get_state, __path_set_state, get_state, set_state};
use super::handlers::templates::{
    __path_create_template, __path_delete_template, __path_get_template, __path_list_templates,
    __path_update_template, create_template, delete_template, get_template, list_templates,
    update_template,
};
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
const MANAGEMENT_BODY_LIMIT: usize = 1024 * 1024;
const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(OpenApi)]
#[openapi(paths(evaluate, evaluations))]
pub struct AuthzApiDoc;

#[derive(OpenApi)]
#[openapi(paths(
    get_state,
    set_state,
    get_schema,
    put_schema,
    get_builtin_schema,
    list_policies,
    create_policy,
    get_policy,
    update_policy,
    delete_policy,
    validate_policies,
    simulate,
    list_templates,
    create_template,
    get_template,
    update_template,
    delete_template
))]
pub struct AuthzManagementApiDoc;

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

    let batch = Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/authzen/access/v1/evaluations"),
            post(evaluations),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(DefaultBodyLimit::max(MANAGEMENT_BODY_LIMIT))
        .layer(middleware::from_fn(echo_request_id));

    let metadata = Router::new().route(
        &format!("/.well-known/authzen-configuration{root_path}/realms/{{realm_name}}/authzen"),
        get(get_authzen_configuration),
    );

    let base = format!("{root_path}/realms/{{realm_name}}/authz");
    let management = Router::new()
        .route(&format!("{base}/state"), get(get_state).put(set_state))
        .route(&format!("{base}/schema"), get(get_schema).put(put_schema))
        .route(&format!("{base}/schema/builtin"), get(get_builtin_schema))
        .route(
            &format!("{base}/policies"),
            get(list_policies).post(create_policy),
        )
        .route(
            &format!("{base}/policies/validate"),
            post(validate_policies),
        )
        .route(
            &format!("{base}/policies/{{policy_id}}"),
            get(get_policy).put(update_policy).delete(delete_policy),
        )
        .route(&format!("{base}/simulate"), post(simulate))
        .route(
            &format!("{base}/templates"),
            get(list_templates).post(create_template),
        )
        .route(
            &format!("{base}/templates/{{template_id}}"),
            get(get_template)
                .put(update_template)
                .delete(delete_template),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
        .layer(DefaultBodyLimit::max(MANAGEMENT_BODY_LIMIT));

    evaluation.merge(batch).merge(management).merge(metadata)
}
