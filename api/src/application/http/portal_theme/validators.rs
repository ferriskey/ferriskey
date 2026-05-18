use ferriskey_core::domain::portal_theme::entities::PortalThemeConfig;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateThemeValidator {
    pub config: PortalThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateThemeValidator {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[serde(default)]
    pub layout_id: Option<Uuid>,
    #[serde(default)]
    pub config: PortalThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateThemeMetadataValidator {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[serde(default)]
    pub layout_id: Option<Uuid>,
    #[serde(default)]
    pub config: PortalThemeConfig,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateThemePageValidator {
    pub tree: serde_json::Value,
}
