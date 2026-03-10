use uuid::Uuid;

pub struct CreateRoleInput {
    pub realm_name: String,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
}

pub struct UpdateRoleInput {
    pub realm_name: String,
    pub role_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

pub struct GetUserRolesInput {
    pub realm_name: String,
    pub user_id: Uuid,
}
