use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        user::{
            entities::{
                AssignRoleInput, CreateUserInput, GetUserInput, ResetPasswordInput,
                UnassignRoleInput, UpdateUserInput, User,
            },
            ports::UserService,
        },
    },
};

impl UserService for ApplicationService {
    async fn assign_role(
        &self,
        identity: Identity,
        input: AssignRoleInput,
    ) -> Result<(), CoreError> {
        self.user_service.assign_role(identity, input).await
    }

    async fn bulk_delete_users(
        &self,
        identity: Identity,
        input: crate::domain::user::entities::BulkDeleteUsersInput,
    ) -> Result<u64, CoreError> {
        self.user_service.bulk_delete_users(identity, input).await
    }

    async fn create_user(
        &self,
        identity: Identity,
        input: CreateUserInput,
    ) -> Result<User, CoreError> {
        self.user_service.create_user(identity, input).await
    }

    async fn delete_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: uuid::Uuid,
    ) -> Result<u64, CoreError> {
        self.user_service
            .delete_user(identity, realm_name, user_id)
            .await
    }

    async fn get_user(&self, identity: Identity, input: GetUserInput) -> Result<User, CoreError> {
        self.user_service.get_user(identity, input).await
    }

    async fn get_users(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<User>, CoreError> {
        self.user_service.get_users(identity, realm_name).await
    }

    async fn reset_password(
        &self,
        identity: Identity,
        input: ResetPasswordInput,
    ) -> Result<(), CoreError> {
        self.user_service.reset_password(identity, input).await
    }

    async fn unassign_role(
        &self,
        identity: Identity,
        input: UnassignRoleInput,
    ) -> Result<(), CoreError> {
        self.user_service.unassign_role(identity, input).await
    }

    async fn update_user(
        &self,
        identity: Identity,
        input: UpdateUserInput,
    ) -> Result<User, CoreError> {
        self.user_service.update_user(identity, input).await
    }
}
