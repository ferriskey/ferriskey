use axum::{Router, middleware, routing::get};
use utoipa::OpenApi;

use crate::application::{
    auth::auth,
    http::{
        compass::handlers::{
            get_daily_activity_stats::{__path_get_daily_activity_stats, get_daily_activity_stats},
            get_flow::{__path_get_flow, get_flow},
            get_flows::{__path_get_flows, get_flows},
            get_stats::{__path_get_stats, get_stats},
        },
        server::app_state::AppState,
    },
};

#[derive(OpenApi)]
#[openapi(paths(get_flows, get_flow, get_stats, get_daily_activity_stats))]
pub struct CompassApiDoc;

pub fn compass_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            &format!(
                "{}/realms/{{realm_name}}/compass/v1/flows",
                state.args.server.root_path
            ),
            get(get_flows),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/compass/v1/flows/{{flow_id}}",
                state.args.server.root_path
            ),
            get(get_flow),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/compass/v1/stats",
                state.args.server.root_path
            ),
            get(get_stats),
        )
        .route(
            &format!(
                "{}/realms/{{realm_name}}/compass/v1/activity/daily",
                state.args.server.root_path
            ),
            get(get_daily_activity_stats),
        )
        .layer(middleware::from_fn_with_state(state.clone(), auth))
}
