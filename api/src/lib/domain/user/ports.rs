use uuid::Uuid;

use super::entities::{
    error::UserError,
    model::{User, UserConfig},
};

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(&self) -> impl Future<Output = Result<User, UserError>> + Send;
}

pub trait UserRepository: Clone + Send + Sync + 'static {
    fn create_user(
        &self,
        user_config: UserConfig,
    ) -> impl Future<Output = Result<User, UserError>> + Send;
}
