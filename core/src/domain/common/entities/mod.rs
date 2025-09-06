use uuid::Uuid;

pub mod app_errors;

#[derive(Debug, Clone)]
pub struct StartupConfig {
    pub master_realm_name: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: String,
    pub default_client_id: String,
}

#[derive(Debug, Clone)]
pub struct InitializationResult {
    pub master_realm_id: Uuid,
    pub admin_user_id: Uuid,
    pub admin_role_id: Uuid,
    pub default_client_id: Uuid,
}
