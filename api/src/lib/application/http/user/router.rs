use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::application::http::server::app_state::AppState;

use super::handlers::{
    get_users::{self, get_users},
    reset_password::{__path_reset_password, reset_password},
};

#[derive(OpenApi)]
#[openapi(paths(reset_password))]
pub struct UserApiDoc;

pub fn user_routes() -> Router<AppState> {
    Router::new().typed_put(reset_password).typed_get(get_users)
}
