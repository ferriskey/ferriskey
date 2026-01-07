use utoipa::OpenApi;

pub mod federation;
pub mod routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        federation::handlers::create_provider::create_provider,
        federation::handlers::list_providers::list_providers,
        federation::handlers::get_provider::get_provider,
        federation::handlers::update_provider::update_provider,
        federation::handlers::delete_provider::delete_provider,
    ),
    components(
        schemas(
            federation::dto::CreateProviderRequest,
            federation::dto::UpdateProviderRequest,
            federation::dto::ProviderResponse,
        )
    ),
    tags(
        (name = "Federation", description = "User Federation management (LDAP, Kerberos, etc.)")
    )
)]
pub struct AbyssApiDoc;
