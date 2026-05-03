use uuid::Uuid;

use crate::realm::RealmId;

#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub realm_id: RealmId,
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateUserRequest {
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
    pub required_actions: Option<Vec<String>>,
}
