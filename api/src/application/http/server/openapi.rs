use ferriskey_api_abyss::AbyssApiDoc;
use ferriskey_api_aegis::router::AegisApiDoc;
use ferriskey_api_authentication::router::AuthenticationApiDoc;
use ferriskey_api_broker::BrokerApiDoc;
use ferriskey_api_client::router::ClientApiDoc;
use ferriskey_api_compass::router::CompassApiDoc;
use ferriskey_api_email_template::router::{EmailTemplateApiDoc, EmailTemplateVariablesApiDoc};
use ferriskey_api_maintenance::router::MaintenanceApiDoc;
use ferriskey_api_organization::router::OrganizationApiDoc;
use ferriskey_api_portal_layouts::router::{PortalLayoutsApiDoc, PortalLayoutsPublicApiDoc};
use ferriskey_api_portal_theme::router::{PortalThemeApiDoc, PortalThemePublicApiDoc};
use ferriskey_api_realm::router::RealmApiDoc;
use ferriskey_api_role::router::RoleApiDoc;
use ferriskey_api_saml::router::SamlApiDoc;
use ferriskey_api_seawatch::router::SeawatchApiDoc;
use ferriskey_api_trident::router::TridentApiDoc;
use ferriskey_api_user::router::UserApiDoc;
use ferriskey_api_webhook::router::WebhookApiDoc;
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
        (path = "/realms/{realm_name}", api = SamlApiDoc),
        (path = "/realms/{realm_name}/roles", api = RoleApiDoc),
        (path = "/realms/{realm_name}/webhooks", api = WebhookApiDoc),
        (path = "/realms/{realm_name}", api = TridentApiDoc),
        (path = "/realms/{realm_name}", api = SeawatchApiDoc),
        (path = "/realms/{realm_name}", api = AbyssApiDoc),
        (path = "/realms/{realm_name}", api = BrokerApiDoc),
        (path = "/realms/{realm_name}", api = AegisApiDoc),
        (path = "/realms/{realm_name}", api = CompassApiDoc),
        (path = "/realms/{realm_name}/email-templates", api = EmailTemplateApiDoc),
        (path = "/realms/{realm_name}", api = PortalThemeApiDoc),
        (path = "/realms/{realm_name}/portal", api = PortalThemePublicApiDoc),
        (path = "/realms/{realm_name}/portal-layouts", api = PortalLayoutsApiDoc),
        (path = "/realms/{realm_name}/portal-layouts/public", api = PortalLayoutsPublicApiDoc),
        (path = "/email-templates/variables", api = EmailTemplateVariablesApiDoc),
        (path = "/realms/{realm_name}/organizations", api = OrganizationApiDoc),
        (path = "/realms/{realm_name}/clients", api = MaintenanceApiDoc)
    )
)]
pub struct ApiDoc;
