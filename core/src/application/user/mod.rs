use uuid::Uuid;

use crate::{
    application::common::{
        FerriskeyService, permissions::FerriskeyPolicy, policies::ensure_policy,
    },
    domain::{
        authentication::value_objects::Identity,
        common::{entities::app_errors::CoreError, policies::Policy},
        credential::ports::CredentialRepository,
        crypto::ports::HasherRepository,
        realm::{entities::Realm, ports::RealmRepository},
        role::entities::permission::Permissions,
        user::{
            entities::{
                AssignRoleInput, CreateUserInput, GetUserInput, RequiredAction, ResetPasswordInput,
                UnassignRoleInput, UpdateUserInput, User,
            },
            ports::{
                UserPolicy, UserRepository, UserRequiredActionRepository, UserRoleRepository,
                UserService,
            },
            value_objects::{CreateUserRequest, UpdateUserRequest},
        },
    },
};

pub mod services;

impl UserService for FerriskeyService {
    async fn delete_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(identity, realm).await,
            "insufficient permissions",
        )?;

        let count = self
            .user_repository
            .delete_user(user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(count)
    }

    async fn reset_password(
        &self,
        identity: Identity,
        input: ResetPasswordInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(identity, realm).await,
            "insufficient permissions",
        )?;

        let password_credential = self
            .credential_repository
            .get_password_credential(input.user_id)
            .await;

        if password_credential.is_ok() {
            self.credential_repository
                .delete_password_credential(input.user_id)
                .await
                .map_err(|_| CoreError::DeletePasswordCredentialError)?;
        }

        let hash_result = self
            .hasher_repository
            .hash_password(&input.password)
            .await
            .map_err(|e| CoreError::HashPasswordError(e.to_string()))?;

        self.credential_repository
            .create_credential(
                input.user_id,
                "password".into(),
                hash_result,
                "".into(),
                input.temporary,
            )
            .await
            .map_err(|_| CoreError::CreateCredentialError)?;

        // @TODO: webhook call action

        Ok(())
    }

    async fn update_user(
        &self,
        identity: Identity,
        input: UpdateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(identity, realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        let user = self
            .user_repository
            .update_user(
                input.user_id,
                UpdateUserRequest {
                    email: input.email,
                    email_verified: input.email_verified.unwrap_or(false),
                    enabled: input.enabled,
                    firstname: input.firstname,
                    lastname: input.lastname,
                    required_actions: None,
                },
            )
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if let Some(required_actions) = input.required_actions {
            self.user_required_action_repository
                .clear_required_actions(user.id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            for action in required_actions {
                let required_action: RequiredAction =
                    RequiredAction::try_from(action).map_err(|_| CoreError::InternalServerError)?;
                self.user_required_action_repository
                    .add_required_action(user.id, required_action)
                    .await
                    .map_err(|_| CoreError::InternalServerError)?;
            }
        }

        Ok(user)
    }

    async fn get_users(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<User>, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;

        ensure_policy(
            self.policy.can_view_user(identity, realm).await,
            "You are not allowed to view users in this realm.",
        )?;

        self.user_repository
            .find_by_realm_id(realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_role_repository
            .assign_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }

    async fn bulk_delete_users(
        &self,
        identity: Identity,
        input: crate::domain::user::entities::BulkDeleteUsersInput,
    ) -> Result<u64, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_delete_user(identity, realm).await,
            "insufficient permissions",
        )?;

        let count = self
            .user_repository
            .bulk_delete_user(input.ids)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(count)
    }

    async fn create_user(
        &self,
        identity: Identity,
        input: CreateUserInput,
    ) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let realm_id = realm.id;
        ensure_policy(
            self.policy.can_create_user(identity, realm.clone()).await,
            "insufficient permissions",
        )?;

        let mut user = self
            .user_repository
            .create_user(CreateUserRequest {
                client_id: None,
                realm_id,
                username: input.username,
                firstname: input.firstname,
                lastname: input.lastname,
                email: input.email,
                email_verified: input.email_verified.unwrap_or(false),
                enabled: true,
            })
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        user.realm = Some(realm);

        Ok(user)
    }

    async fn get_user(&self, identity: Identity, input: GetUserInput) -> Result<User, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_view_user(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_repository
            .get_by_id(input.user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)
    }

    async fn unassign_role(
        &self,
        identity: Identity,
        input: UnassignRoleInput,
    ) -> Result<(), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.policy.can_update_user(identity, realm).await,
            "insufficient permissions",
        )?;

        self.user_role_repository
            .revoke_role(input.user_id, input.role_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(())
    }
}

impl UserPolicy for FerriskeyPolicy {
    async fn can_create_user(
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

    async fn can_delete_user(
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

    async fn can_update_user(
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

    async fn can_view_user(
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
            &[Permissions::ManageRealm, Permissions::ViewUsers],
        );

        Ok(has_permission)
    }
}
