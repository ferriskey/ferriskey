use crate::domain::role::entities::RoleError;

pub mod policies;
pub mod use_cases;

#[inline]
pub(in crate::application::role) fn ensure_permissions(
    result_has_permission: Result<bool, RoleError>,
    error_message: &str,
) -> Result<(), RoleError> {
    result_has_permission
        .map_err(|_| RoleError::Forbidden(error_message.to_string()))?
        .then_some(())
        .ok_or_else(|| RoleError::Forbidden(error_message.to_string()))
}
