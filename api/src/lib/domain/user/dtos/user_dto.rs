use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateUserDto {
    pub realm_id: Uuid,
    pub client_id: Option<Uuid>,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateUserDto {
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
    pub required_actions: Option<Vec<String>>,
}
