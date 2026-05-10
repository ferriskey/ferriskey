use ferriskey_core::domain::portal_theme::entities::PortalThemeConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateThemeValidator {
    pub config: PortalThemeConfig,
}
