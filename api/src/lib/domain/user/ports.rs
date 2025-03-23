use super::entities::{error::UserError, model::User};

pub trait UserService: Clone + Send + Sync + 'static {
    fn create_user(&self) -> impl Future<Output = Result<User, UserError>> + Send;
}
