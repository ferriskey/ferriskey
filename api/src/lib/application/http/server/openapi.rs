use crate::application::http::realm::router::RealmApiDoc;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerrisKey API"
    ),
    nest(
        (path = "/realms", api = RealmApiDoc)
    )

)]
pub struct ApiDoc;
