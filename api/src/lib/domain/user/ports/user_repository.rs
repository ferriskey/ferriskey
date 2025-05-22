use crate::domain::role::entities::models::Role;
use crate::domain::user::dtos::user_dto::CreateUserDto;
use crate::domain::user::entities::{error::UserError, model::User};
use uuid::Uuid;

pub trait UserRepository: Clone + Send + Sync + 'static {
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
    fn get_roles_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Role>, UserError>> + Send;

    fn find_by_realm_id(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<User>, UserError>> + Send;

    fn bulk_delete_user(
        &self,
        ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<(), UserError>> + Send;
}
