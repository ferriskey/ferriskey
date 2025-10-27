use crate::domain::{
    authentication::{ports::AuthSessionRepository, value_objects::Identity},
    client::ports::{ClientRepository, RedirectUriRepository},
    common::{entities::app_errors::CoreError, policies::ensure_policy, services::Service},
    credential::ports::CredentialRepository,
    crypto::ports::HasherRepository,
    health::ports::HealthCheckRepository,
    jwt::ports::{KeyStoreRepository, RefreshTokenRepository},
    realm::ports::RealmRepository,
    role::{
        entities::{GetUserRolesInput, Role, UpdateRoleInput},
        ports::{RolePolicy, RoleRepository, RoleService},
        value_objects::{UpdateRolePermissionsRequest, UpdateRoleRequest},
    },
    trident::ports::RecoveryCodeRepository,
    user::ports::{UserRepository, UserRequiredActionRepository, UserRoleRepository},
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};

impl<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC> RoleService
    for Service<R, C, U, CR, H, AS, RU, RO, KS, UR, URA, HC, W, RT, RC>
where
    R: RealmRepository,
    C: ClientRepository,
    U: UserRepository,
    CR: CredentialRepository,
    H: HasherRepository,
    AS: AuthSessionRepository,
    RU: RedirectUriRepository,
    RO: RoleRepository,
    KS: KeyStoreRepository,
    UR: UserRoleRepository,
    URA: UserRequiredActionRepository,
    HC: HealthCheckRepository,
    W: WebhookRepository,
    RT: RefreshTokenRepository,
    RC: RecoveryCodeRepository,
{
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

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_delete_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self.role_repository.get_by_id(role_id).await?;
        self.role_repository.delete_by_id(role_id).await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::RoleDeleted, realm_id, Some(role)),
            )
            .await?;

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

        let realm_id = realm.id;
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

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(WebhookTrigger::RoleUpdated, realm_id, Some(role.clone())),
            )
            .await?;

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

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_update_role(identity, realm).await,
            "insufficient permissions",
        )?;

        let role = self
            .role_repository
            .update_permissions_by_id(role_id, UpdateRolePermissionsRequest { permissions })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(
                    WebhookTrigger::RolePermissionUpdated,
                    realm_id,
                    Some(role.clone()),
                ),
            )
            .await?;

        Ok(role)
    }

    async fn get_user_roles(
        &self,
        identity: Identity,
        input: GetUserRolesInput,
    ) -> Result<Vec<Role>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::InternalServerError)?;

        ensure_policy(
            self.policy.can_view_role(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_role_repository
            .get_user_roles(input.user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        authentication::value_objects::Identity,
        client::entities::Client,
        common::{
            entities::app_errors::CoreError,
            services::tests::{
                ServiceTestBuilder, assert_success, create_test_realm, create_test_realm_with_name,
                create_test_role, create_test_role_with_params, create_test_user,
                create_test_user_with_realm,
            },
        },
        role::{entities::permission::Permissions, ports::RoleService},
    };
    use std::vec;

    #[tokio::test]
    async fn test_get_role_success() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let user_role_with_permissions = create_test_role_with_params(
            realm.id,
            "viewer-role",
            vec![Permissions::ViewRoles.name()],
            None,
        );

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![user_role_with_permissions])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
        assert_eq!(returned_role.name, "test-role");
        assert_eq!(returned_role.realm_id, realm.id);
    }

    #[tokio::test]
    async fn test_get_role_success_with_manage_users_permissions() {
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);

        let identity = Identity::User(user.clone());

        let admin_role = create_test_role_with_params(
            realm.id,
            "admin-role",
            vec![Permissions::ManageUsers.name()],
            None,
        );

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        let result = service.get_role(identity, realm.name, role.id).await;

        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
    }

    #[tokio::test]
    async fn test_get_role_success_with_manage_realm_permission() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let realm_admin_role = create_test_role_with_params(
            realm.id,
            "realm-admin",
            vec![Permissions::ManageRealm.name()],
            None,
        );

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![realm_admin_role])
            .with_successful_role_lookup(role.id, role.clone())
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role.id);
    }

    #[tokio::test]
    async fn test_get_role_user_without_realm_should_fail() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user(realm.id);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .build();

        let result = service.get_role(identity, realm.name, role.id).await;

        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_insufficient_permissions() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let insufficient_role = create_test_role_with_params(
            realm.id,
            "basic-user",
            vec!["some_other_permission".to_string()],
            None,
        );

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_user_roles(user.id, vec![insufficient_role])
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_no_roles_at_all() {
        // Arrange
        let realm = create_test_realm();
        let user = create_test_user_with_realm(&realm);
        let role = create_test_role(realm.id);
        let identity = Identity::User(user.clone());

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&realm.name, realm.clone())
            .with_no_user_roles(user.id)
            .build();

        // Act
        let result = service.get_role(identity, realm.name, role.id).await;

        // Assert
        assert!(matches!(result.unwrap_err(), CoreError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_get_role_cross_realm_access_from_master() {
        // Arrange
        let master_realm = create_test_realm_with_name("master");
        let target_realm = create_test_realm_with_name("target-realm");
        let user_in_master = create_test_user_with_realm(&master_realm);
        let role_in_target = create_test_role(target_realm.id);
        let identity = Identity::User(user_in_master.clone());

        let target_realm_client_id = format!("{}-realm", target_realm.name);
        let target_realm_client =
            Client::from_realm_and_client_id(master_realm.id, target_realm_client_id.clone());

        let cross_realm_role = create_test_role_with_params(
            master_realm.id,
            "cross-realm-admin",
            vec![Permissions::ViewRoles.name()],
            Some(target_realm_client.id),
        );

        let service = ServiceTestBuilder::new()
            .with_successful_realm_lookup(&target_realm.name, target_realm.clone())
            .with_user_roles(user_in_master.id, vec![cross_realm_role])
            .with_successful_client_lookup(
                &target_realm_client_id,
                master_realm.id,
                target_realm_client,
            )
            .with_successful_role_lookup(role_in_target.id, role_in_target.clone())
            .build();

        let result = service
            .get_role(identity, target_realm.name, role_in_target.id)
            .await;

        let returned_role = assert_success(result);
        assert_eq!(returned_role.id, role_in_target.id);
    }
}
