use uuid::Uuid;

use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        role::{
            entities::{GetUserRolesInput, Role, UpdateRoleInput},
            ports::RoleService,
        },
    },
};

impl RoleService for ApplicationService {
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

    async fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Role>, CoreError> {
        self.role_service.get_roles(identity, realm_name).await
    }

    async fn get_user_roles(
        &self,
        identity: Identity,
        input: GetUserRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        self.role_service.get_user_roles(identity, input).await
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
