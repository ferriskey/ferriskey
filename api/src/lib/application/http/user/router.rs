use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use crate::domain::credential::ports::credential_service::CredentialService;

use super::handlers::reset_password::{__path_reset_password, reset_password};

#[derive(OpenApi)]
#[openapi(paths(reset_password))]
pub struct UserApiDoc;

pub fn user_routes<C: CredentialService>() -> Router {
    Router::new().typed_put(reset_password::<C>)
}
