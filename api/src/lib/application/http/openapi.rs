use utoipa::OpenApi;

use crate::application::http::handlers::realm::RealmApiDoc;

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
