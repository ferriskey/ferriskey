use uuid::Uuid;

use crate::domain::{role::entities::models::Role, user::entities::error::UserError};

pub trait UserRoleService: Send + Sync {
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
