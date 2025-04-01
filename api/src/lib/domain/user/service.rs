use super::{
    entities::{error::UserError, model::User},
    ports::{CreateUserDto, UserRepository, UserService},
};

#[derive(Debug, Clone)]
pub struct UserServiceImpl<U>
where
    U: UserRepository,
{
    pub user_repository: U,
}

impl<U> UserServiceImpl<U>
where
    U: UserRepository,
{
    pub fn new(user_repository: U) -> Self {
        Self { user_repository }
    }
}

impl<U> UserService for UserServiceImpl<U>
where
    U: UserRepository,
{
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, UserError> {
        self.user_repository.create_user(dto).await
    }
}
