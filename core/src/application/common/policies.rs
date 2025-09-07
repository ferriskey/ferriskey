use crate::domain::common::entities::app_errors::CoreError;

pub fn ensure_policy(
    result_has_permission: Result<bool, CoreError>,
    error_message: &str,
) -> Result<(), CoreError> {
    match result_has_permission {
        Ok(true) => Ok(()),
        Ok(false) => Err(CoreError::Forbidden(error_message.to_string())),
        Err(_) => Err(CoreError::Forbidden(error_message.to_string())),
    }
}
