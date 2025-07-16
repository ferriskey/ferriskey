use uuid::Uuid;

use crate::domain::user::entities::required_action::{RequiredAction, RequiredActionError};

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
