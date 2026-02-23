use uuid::Uuid;

pub struct UpdateUserInput {
    pub realm_name: String,
    pub user_id: Uuid,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: Option<bool>,
    pub enabled: bool,
    pub required_actions: Option<Vec<String>>,
}

pub struct UnassignRoleInput {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

pub struct GetUserPermissionsInput {
    pub realm_name: String,
    pub user_id: Uuid,
}
pub struct ResetPasswordInput {
    pub user_id: Uuid,
    pub password: String,
    pub temporary: bool,
    pub realm_name: String,
}

pub struct AssignRoleInput {
    pub realm_name: String,
    pub user_id: Uuid,
    pub role_id: Uuid,
}

pub struct BulkDeleteUsersInput {
    pub realm_name: String,
    pub ids: Vec<Uuid>,
}

pub struct CreateUserInput {
    pub realm_name: String,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: Option<bool>,
}

pub struct GetUserInput {
    pub realm_name: String,
    pub user_id: Uuid,
}
