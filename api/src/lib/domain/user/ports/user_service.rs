use crate::domain::realm::entities::realm::Realm;
use crate::domain::role::entities::models::Role;
use crate::domain::user::dtos::user_dto::{CreateUserDto, UpdateUserDto};
use crate::domain::user::entities::{error::UserError, model::User};
use uuid::Uuid;

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        dto: CreateUserDto,
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
        dto: UpdateUserDto,
    ) -> impl Future<Output = Result<User, UserError>> + Send;
}
