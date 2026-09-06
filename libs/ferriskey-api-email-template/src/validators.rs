use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateEmailTemplateValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(length(min = 1, message = "email_type is required"))]
    pub email_type: String,

    pub structure: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateEmailTemplateValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    pub structure: serde_json::Value,
}

/// Import payload. Exactly one of `structure`, `tree` or `mjml` carries the
/// template body: `structure` is the persisted `{ children: [...] }` wrapper,
/// `tree` the bare `BuilderNode[]` array exported by the builder, and `mjml`
/// raw markup that is parsed back into a builder structure.
#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportEmailTemplateValidator {
    /// Which builder the file came from, when it came from an export. Checked
    /// so a portal layout cannot be imported as an email template.
    #[serde(default)]
    pub ferriskey: Option<String>,

    /// Export format version, when the payload came from an export.
    #[serde(default)]
    pub version: Option<u32>,

    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    /// Optional at deserialisation so a file from the other builder is answered
    /// by the envelope check — which says what the file actually is — rather
    /// than by a "missing field email_type" the person importing cannot act on.
    /// The handler still requires it.
    #[serde(default)]
    pub email_type: Option<String>,

    #[serde(default)]
    pub structure: Option<serde_json::Value>,

    #[serde(default)]
    pub tree: Option<serde_json::Value>,

    #[serde(default)]
    pub mjml: Option<String>,
}
