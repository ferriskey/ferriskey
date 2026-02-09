use crate::application::http::{
    abyss::AbyssApiDoc, aegis::router::AegisApiDoc, authentication::router::AuthenticationApiDoc,
    broker::BrokerApiDoc, client::router::ClientApiDoc, realm::router::RealmApiDoc,
    role::router::RoleApiDoc, seawatch::router::SeawatchApiDoc, trident::router::TridentApiDoc,
    user::router::UserApiDoc, webhook::router::WebhookApiDoc,
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
        (path = "/realms/{realm_name}/roles", api = RoleApiDoc),
        (path = "/realms/{realm_name}/webhooks", api = WebhookApiDoc),
        (path = "/realms/{realm_name}", api = TridentApiDoc),
        (path = "/realms/{realm_name}", api = SeawatchApiDoc),
        (path = "/realms/{realm_name}", api = AbyssApiDoc),
        (path = "/realms/{realm_name}", api = BrokerApiDoc),
        (path = "/realms/{realm_name}/client-scopes", api = AegisApiDoc)
    )
)]
pub struct ApiDoc;
