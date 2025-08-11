use uuid::Uuid;

use crate::domain::{
    realm::entities::Realm,
    role::entities::Role,
    user::{
        entities::{RequiredAction, RequiredActionError, User, UserError},
        value_objects::{CreateUserRequest, UpdateUserRequest},
    },
};

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        dto: CreateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_username(
        &self,
        username: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_id(&self, user_id: Uuid) -> impl Future<Output = Result<User, UserError>> + Send;
    fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, UserError>> + Send;

    fn get_user_realms(
        &self,
        user: User,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<Realm>, UserError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<User>, UserError>> + Send;

    fn bulk_delete_user(
        &self,
        ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<u64, UserError>> + Send;

    fn delete_user(&self, user_id: Uuid) -> impl Future<Output = Result<u64, UserError>> + Send;

    fn update_user(
        &self,
        user_id: Uuid,
        dto: UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn remove_required_action(
        &self,
        user_id: Uuid,
        required_action: RequiredAction,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
}

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        dto: CreateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_username(
        &self,
        username: String,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<User, UserError>> + Send;

    fn get_by_id(&self, user_id: Uuid) -> impl Future<Output = Result<User, UserError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<User>, UserError>> + Send;

    fn bulk_delete_user(
        &self,
        ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<u64, UserError>> + Send;

    fn delete_user(&self, user_id: Uuid) -> impl Future<Output = Result<u64, UserError>> + Send;

    fn update_user(
        &self,
        user_id: Uuid,
        dto: UpdateUserRequest,
    ) -> impl Future<Output = Result<User, UserError>> + Send;
}

pub trait UserRequiredActionRepository: Clone + Send + Sync + 'static {
    fn add_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> impl Future<Output = Result<(), RequiredActionError>> + Send;

    fn remove_required_action(
        &self,
        user_id: Uuid,
        action: RequiredAction,
    ) -> impl Future<Output = Result<(), RequiredActionError>> + Send;

    fn get_required_actions(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<RequiredAction>, RequiredActionError>> + Send;

    fn clear_required_actions(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<u64, RequiredActionError>> + Send;
}

pub trait UserRoleService: Send + Sync {
    fn assign_role(
        &self,
        realm_name: String,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), UserError>> + Send;

    fn revoke_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), UserError>> + Send;

    fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, UserError>> + Send;

    fn has_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<bool, UserError>> + Send;
}

pub trait UserRoleRepository: Clone + Send + Sync + 'static {
    fn assign_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
    fn revoke_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
    fn get_user_roles(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, UserError>> + Send;
    fn has_role(
        &self,
        user_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<bool, UserError>> + Send;
}
