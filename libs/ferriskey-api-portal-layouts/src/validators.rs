use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreatePortalLayoutValidator {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,

    pub tree: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportPortalLayoutValidator {
    /// Which builder the file came from, when it came from an export. Checked
    /// so an email template cannot be imported as a layout.
    #[serde(default)]
    pub ferriskey: Option<String>,

    /// Export format version, when the payload came from an export.
    #[serde(default)]
    pub version: Option<u32>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,

    pub tree: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdatePortalLayoutValidator {
    #[validate(length(
        min = 1,
        max = 255,
        message = "name must be between 1 and 255 characters"
    ))]
    pub name: String,

    pub tree: serde_json::Value,
}
