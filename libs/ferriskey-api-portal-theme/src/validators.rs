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

/// Import payload — an exported theme envelope, sent back verbatim.
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportPortalThemeValidator {
    /// Which builder the file came from, when it came from an export. Checked
    /// so a layout or an email template cannot be imported as a theme.
    #[serde(default)]
    pub ferriskey: Option<String>,

    /// Export format version, when the payload came from an export.
    #[serde(default)]
    pub version: Option<u32>,

    #[validate(length(min = 1, max = 255))]
    pub name: String,

    /// The theme's design tokens. Missing keys fall back to their defaults.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub config: serde_json::Value,

    /// One builder tree per page, keyed by page type.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub pages: serde_json::Value,

    /// The layout carried inside the file, recreated on import.
    #[serde(default)]
    pub layout: Option<ImportPortalThemeLayoutValidator>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportPortalThemeLayoutValidator {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[schema(value_type = Object)]
    pub tree: serde_json::Value,
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
