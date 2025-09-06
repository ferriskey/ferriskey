use crate::{
    application::common::{
        FerriskeyService, permissions::FerriskeyPolicy, policies::ensure_policy,
    },
    domain::{
        authentication::value_objects::Identity,
        common::{entities::app_errors::CoreError, policies::Policy},
        realm::{entities::Realm, ports::RealmRepository},
        role::{
            entities::{Role, RoleError, UpdateRoleInput, permission::Permissions},
            ports::{RolePolicy, RoleRepository, RoleService},
            value_objects::{CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest},
        },
    },
};

pub mod policies;

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
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_delete_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_update_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[Permissions::ManageRealm, Permissions::ManageUsers],
        );

        Ok(has_permission)
    }

    async fn can_view_role(
        &self,
        identity: Identity,
        target_realm: Realm,
    ) -> Result<bool, CoreError> {
        let user = self.get_user_from_identity(&identity).await?;

        let permissions = self
            .get_permission_for_target_realm(&user, &target_realm)
            .await?;

        let has_permission = Permissions::has_one_of_permissions(
            &permissions.iter().cloned().collect::<Vec<Permissions>>(),
            &[
                Permissions::ManageRealm,
                Permissions::ManageUsers,
                Permissions::ViewRoles,
            ],
        );

        Ok(has_permission)
    }
}

impl RoleService for FerriskeyService {
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
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_delete_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .delete_by_id(role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn get_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .get_by_id(role_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .ok_or(CoreError::NotFound)
    }

    async fn get_roles(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.role_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::NotFound)
    }

    async fn update_role(
        &self,
        identity: Identity,
        input: UpdateRoleInput,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_by_id(
                input.role_id,
                UpdateRoleRequest {
                    description: input.description,
                    name: input.name,
                },
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(role)
    }

    async fn update_role_permissions(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: uuid::Uuid,
        permissions: Vec<String>,
    ) -> Result<Role, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_permissions_by_id(role_id, UpdateRolePermissionsRequest { permissions })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(role)
    }
}
