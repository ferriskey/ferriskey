use crate::application::common::services::DefaultRoleService;
use crate::domain::role::entities::Role;
use crate::domain::role::ports::RoleService;
use uuid::Uuid;

#[derive(Clone)]
pub struct GetRoleUseCase {
    pub role_service: DefaultRoleService,
}

pub struct GetRoleUseCaseParams {
    pub role_id: Uuid,
}

impl GetRoleUseCase {
    pub fn new(role_service: DefaultRoleService) -> Self {
        Self { role_service }
    }

    pub async fn execute(&self, params: GetRoleUseCaseParams) -> Result<Role, String> {
        self.role_service
            .get_by_id(params.role_id)
            .await
            .map_err(|e| e.to_string())
    }
}
