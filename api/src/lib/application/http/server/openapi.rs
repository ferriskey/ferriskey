use crate::application::http::{client::router::ClientApiDoc, realm::router::RealmApiDoc};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerrisKey API"
    ),
    nest(
        (path = "/realms", api = RealmApiDoc),
        (path = "/realms/{realm_name}/clients", api = ClientApiDoc)
    )

)]
pub struct ApiDoc;
