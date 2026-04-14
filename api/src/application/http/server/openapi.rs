use crate::application::http::{
    abyss::AbyssApiDoc,
    aegis::router::AegisApiDoc,
    authentication::router::AuthenticationApiDoc,
    broker::BrokerApiDoc,
    client::router::ClientApiDoc,
    compass::router::CompassApiDoc,
    email_template::router::{EmailTemplateApiDoc, EmailTemplateVariablesApiDoc},
    maintenance::router::MaintenanceApiDoc,
    organization::router::OrganizationApiDoc,
    realm::router::RealmApiDoc,
    role::router::RoleApiDoc,
    seawatch::router::SeawatchApiDoc,
    trident::router::TridentApiDoc,
    user::router::UserApiDoc,
    webhook::router::WebhookApiDoc,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerrisKey API",
        license(name = "Apache-2.0", identifier = "Apache-2.0")
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
        (path = "/realms/{realm_name}", api = AegisApiDoc),
        (path = "/realms/{realm_name}", api = CompassApiDoc),
        (path = "/realms/{realm_name}/email-templates", api = EmailTemplateApiDoc),
        (path = "/email-templates/variables", api = EmailTemplateVariablesApiDoc),
        (path = "/realms/{realm_name}/organizations", api = OrganizationApiDoc),
        (path = "/realms/{realm_name}/clients", api = MaintenanceApiDoc)
    )
)]
pub struct ApiDoc;
