use crate::{
    application::common::{FerriskeyService, permissions::FerriskeyPolicy},
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        realm::{entities::Realm, ports::RealmRepository},
        role::{
            entities::{Role, RoleError},
            ports::{RolePolicy, RoleService},
            value_objects::CreateRoleRequest,
        },
    },
};

pub mod policies;
pub mod use_cases;

#[inline]
pub(in crate::application::role) fn ensure_permissions(
    result_has_permission: Result<bool, RoleError>,
    error_message: &str,
) -> Result<(), RoleError> {
    result_has_permission
        .map_err(|_| RoleError::Forbidden(error_message.to_string()))?
        .then_some(())
        .ok_or_else(|| RoleError::Forbidden(error_message.to_string()))
}

impl RolePolicy for FerriskeyPolicy {
    async fn can_create_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        todo!()
    }

    async fn can_delete_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        todo!()
    }

    async fn can_update_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        todo!()
    }

    async fn can_view_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        todo!()
    }
}

impl RoleService for FerriskeyService {
    async fn create(&self, _payload: CreateRoleRequest) -> Result<Role, RoleError> {
        unimplemented!()
    }

    async fn delete_by_id(&self, id: uuid::Uuid) -> Result<(), RoleError> {
        unimplemented!()
    }

    async fn delete_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        todo!()
    }

    async fn find_by_name(&self, name: String, realm_id: uuid::Uuid) -> Result<Role, RoleError> {
        unimplemented!()
    }

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        unimplemented!()
    }

    async fn get_by_client_id_text(
        &self,
        client_id: String,
        realm_id: uuid::Uuid,
    ) -> Result<Vec<Role>, RoleError> {
        unimplemented!()
    }

    async fn get_by_id(&self, id: uuid::Uuid) -> Result<Role, RoleError> {
        unimplemented!()
    }

    async fn get_by_realm_id(&self, realm_id: uuid::Uuid) -> Result<Vec<Role>, RoleError> {
        unimplemented!()
    }

    async fn update_by_id(
        &self,
        id: uuid::Uuid,
        payload: crate::domain::role::value_objects::UpdateRoleRequest,
    ) -> Result<Role, RoleError> {
        unimplemented!()
    }

    async fn update_permissions_by_id(
        &self,
        id: uuid::Uuid,
        payload: crate::domain::role::value_objects::UpdateRolePermissionsRequest,
    ) -> Result<Role, RoleError> {
        unimplemented!()
    }
}
