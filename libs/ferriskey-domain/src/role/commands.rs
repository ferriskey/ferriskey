use uuid::Uuid;

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
