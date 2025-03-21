use axum::Router;
use axum_extra::routing::RouterExt;
use utoipa::OpenApi;

use super::handlers::create_client::{__path_create_client, create_client};
use crate::domain::client::ports::ClientService;

#[derive(OpenApi)]
#[openapi(
    paths(create_client),
    tags(
        (name = "client", description = "Client management")
    )
)]
pub struct ClientApiDoc;

pub fn client_routes<C: ClientService>() -> Router {
    Router::new().typed_post(create_client::<C>)
}
