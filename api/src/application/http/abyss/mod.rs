use utoipa::OpenApi;

pub mod federation;
pub mod identity_provider;
pub mod routes;

#[derive(OpenApi)]
#[openapi(
    paths(
        federation::handlers::create_provider::create_provider,
        federation::handlers::list_providers::list_providers,
        federation::handlers::get_provider::get_provider,
        federation::handlers::update_provider::update_provider,
        federation::handlers::delete_provider::delete_provider,
        federation::handlers::test_connection::test_connection,
        federation::handlers::sync_users::sync_users,

        identity_provider::handlers::create_identity_provider::create_identity_provider,
        identity_provider::handlers::list_identity_providers::list_identity_providers,
        identity_provider::handlers::get_identity_provider::get_identity_provider,
        identity_provider::handlers::update_identity_provider::update_identity_provider,
        identity_provider::handlers::delete_identity_provider::delete_identity_provider,
    ),
    components(
        schemas(
            federation::dto::CreateProviderRequest,
            federation::dto::UpdateProviderRequest,
            federation::dto::ProviderResponse,
        )
    ),
    tags(
        (name = "federation", description = "User Federation management (LDAP, Kerberos, etc.)")
    )
)]
pub struct AbyssApiDoc;
