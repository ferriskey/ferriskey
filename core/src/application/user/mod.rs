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
            entities::{ResetPasswordInput, User, UserError},
            ports::{UserPolicy, UserService},
            value_objects::{CreateUserRequest, UpdateUserRequest},
        },
    },
};

mod policies;
pub mod services;
pub mod use_cases;

impl UserService for FerriskeyService {
    async fn bulk_delete_user(&self, ids: Vec<uuid::Uuid>) -> Result<u64, UserError> {
        todo!()
    }

    async fn create_user(&self, dto: CreateUserRequest) -> Result<User, UserError> {
        todo!()
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<u64, UserError> {
        todo!()
    }

    async fn find_by_realm_id(&self, realm_id: Uuid) -> Result<Vec<User>, UserError> {
        todo!()
    }

    async fn get_by_client_id(&self, client_id: uuid::Uuid) -> Result<User, UserError> {
        todo!()
    }

    async fn get_by_id(&self, user_id: uuid::Uuid) -> Result<User, UserError> {
        todo!()
    }

    async fn get_by_username(
        &self,
        username: String,
        realm_id: uuid::Uuid,
    ) -> Result<User, UserError> {
        todo!()
    }

    async fn get_user_realms(
        &self,
        user: User,
        realm_name: String,
    ) -> Result<Vec<crate::domain::realm::entities::Realm>, UserError> {
        todo!()
    }

    async fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::domain::role::entities::Role>, UserError> {
        todo!()
    }

    async fn remove_required_action(
        &self,
        user_id: Uuid,
        required_action: crate::domain::user::entities::RequiredAction,
    ) -> Result<(), UserError> {
        todo!()
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

    async fn update_user(&self, user_id: Uuid, dto: UpdateUserRequest) -> Result<User, UserError> {
        todo!()
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
