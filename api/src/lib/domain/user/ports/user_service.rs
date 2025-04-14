use crate::domain::user::dtos::user_dto::CreateUserDto;
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
        realm_id: Uuid,
    ) -> impl Future<Output = Result<User, UserError>> + Send;
}
