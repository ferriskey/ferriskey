use crate::application::http::{
    authentication::router::AuthenticationApiDoc, client::router::ClientApiDoc,
    realm::router::RealmApiDoc, role::router::RoleApiDoc, user::router::UserApiDoc,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerrisKey API"
    ),
    nest(
        (path = "/realms", api = RealmApiDoc),
        (path = "/realms/{realm_name}/clients", api = ClientApiDoc),
        (path = "/realms/{realm_name}/users", api = UserApiDoc),
        (path = "/realms/{realm_name}", api = AuthenticationApiDoc),
        (path = "realms/{realm_name}/roles", api = RoleApiDoc),
    )
)]
pub struct ApiDoc;
