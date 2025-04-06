use uuid::Uuid;

use super::entities::{error::UserError, model::User};

#[derive(Debug, Clone)]
pub struct CreateUserDto {
    pub realm_id: Uuid,
    pub username: String,
    pub firstname: String,
    pub lastname: String,
    pub email: String,
    pub email_verified: bool,
    pub enabled: bool,
}

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
}

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
}
