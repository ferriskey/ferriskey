use axum_macros::TypedPath;
use serde::Deserialize;
use uuid::Uuid;

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects")]
pub struct GetRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects")]
pub struct CreateRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}")]
pub struct UpdateRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/realms/{realm_name}/clients/{client_id}/redirects/{uri_id}")]
pub struct DeleteRedirectUriRoute {
    pub realm_name: String,
    pub client_id: Uuid,
    pub uri_id: Uuid,
}
