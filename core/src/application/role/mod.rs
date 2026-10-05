use uuid::Uuid;

use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::{
            entities::app_errors::CoreError,
            pagination::{Page, PageRequest},
        },
        role::{
            entities::{CreateRoleInput, Role, RoleFilter, RoleSortField, UpdateRoleInput},
            ports::RoleService,
        },
    },
};

impl RoleService for ApplicationService {
    async fn create_role(
        &self,
        identity: Identity,
        input: CreateRoleInput,
    ) -> Result<Role, CoreError> {
        self.role_service.create_role(identity, input).await
    }

    async fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> Result<(), CoreError> {
        self.role_service
            .delete_role(identity, realm_name, role_id)
            .await
    }

    async fn get_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> Result<Role, CoreError> {
        self.role_service
            .get_role(identity, realm_name, role_id)
            .await
    }

    async fn list_roles(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<RoleFilter, RoleSortField>,
    ) -> Result<Page<Role>, CoreError> {
        self.role_service
            .list_roles(identity, realm_name, request)
            .await
    }

    async fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> Result<Role, CoreError> {
        self.role_service.update_role(identity, input).await
    }

    async fn update_role_permissions(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
        permissions: Vec<String>,
    ) -> Result<Role, CoreError> {
        self.role_service
            .update_role_permissions(identity, realm_name, role_id, permissions)
            .await
    }
}
