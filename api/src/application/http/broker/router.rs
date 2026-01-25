use axum::{Router, routing::get};
use utoipa::OpenApi;

use crate::application::http::server::app_state::AppState;

use super::handlers::callback::__path_broker_callback;
use super::handlers::login::__path_broker_login;
use super::handlers::{broker_callback, broker_login};

#[derive(OpenApi)]
#[openapi(paths(broker_login, broker_callback))]
pub struct BrokerApiDoc;

pub fn broker_routes(state: AppState, root_path: &str) -> Router<AppState> {
    Router::new()
        .route(
            &format!("{root_path}/realms/{{realm_name}}/broker/{{alias}}/login"),
            get(broker_login),
        )
        .route(
            &format!("{root_path}/realms/{{realm_name}}/broker/{{alias}}/endpoint"),
            get(broker_callback),
        )
        .with_state(state)
}
