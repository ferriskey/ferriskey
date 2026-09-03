//! Shared pieces of the builder import/export file format.
//!
//! Email templates and portal layouts are both stored as builder trees, so both
//! export the same envelope — kind, format version, name and the tree itself —
//! and the frontend can tell one file from the other before loading it into a
//! canvas.

use crate::api_entities::api_error::ApiError;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Bumped when the envelope itself changes shape, never when a builder tree does.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ExportEnvelope {
    /// Which builder the file belongs to: `email-template` or `portal-layout`.
    pub ferriskey: String,
    /// Format version, bumped when the envelope itself changes shape.
    pub version: u32,
    pub name: String,
    /// Email templates only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_type: Option<String>,
    /// The builder tree (`BuilderNode[]`).
    #[schema(value_type = Vec<Object>)]
    pub tree: serde_json::Value,
}

impl ExportEnvelope {
    pub const EMAIL_TEMPLATE: &'static str = "email-template";
    pub const PORTAL_LAYOUT: &'static str = "portal-layout";

    /// The envelope of an email template. `email_type` is carried so an import
    /// can restore the template under the type it was exported from.
    pub fn email_template(name: String, email_type: String, tree: serde_json::Value) -> Self {
        Self {
            ferriskey: Self::EMAIL_TEMPLATE.to_string(),
            version: FORMAT_VERSION,
            name,
            email_type: Some(email_type),
            tree,
        }
    }

    /// The envelope of a portal layout, which has no email type.
    pub fn portal_layout(name: String, tree: serde_json::Value) -> Self {
        Self {
            ferriskey: Self::PORTAL_LAYOUT.to_string(),
            version: FORMAT_VERSION,
            name,
            email_type: None,
            tree,
        }
    }

    /// Serialises the envelope into a download: pretty JSON, so a file a human
    /// opens is readable, under a filename derived from the envelope's name.
    pub fn into_download(self) -> Result<Response, serde_json::Error> {
        let filename = format!("{}.json", slugify(&self.name));
        let body = serde_json::to_vec_pretty(&self)?;

        Ok(download(body, "application/json", &filename))
    }
}

/// Turns a template/layout name into a filename stem: lowercase, ASCII
/// alphanumerics and dashes only, so it is safe in a `Content-Disposition`
/// header on any platform.
pub fn slugify(name: &str) -> String {
    let slug = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    let slug = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if slug.is_empty() {
        "export".to_string()
    } else {
        slug
    }
}

/// The envelope of an exported portal theme.
///
/// A theme is more than a tree: it carries the token config plus one tree per
/// page of the authentication flow, and it points at the layout that frames
/// them. The layout travels *inside* the file rather than as an id — ids are
/// realm-local, so an id would dangle the moment the file crosses into another
/// deployment, which is the whole point of exporting one.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ThemeExportEnvelope {
    /// Always `portal-theme`; the kind check reads this.
    pub ferriskey: String,
    pub version: u32,
    pub name: String,
    /// The theme's design tokens, verbatim.
    #[schema(value_type = Object)]
    pub config: serde_json::Value,
    /// One builder tree per page, keyed by page type.
    #[schema(value_type = Object)]
    pub pages: serde_json::Value,
    /// The layout this theme is framed by, when it has one. Carried by value so
    /// the import can recreate it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<ThemeExportLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct ThemeExportLayout {
    pub name: String,
    #[schema(value_type = Vec<Object>)]
    pub tree: serde_json::Value,
}

impl ThemeExportEnvelope {
    pub const KIND: &'static str = "portal-theme";

    pub fn new(
        name: String,
        config: serde_json::Value,
        pages: serde_json::Value,
        layout: Option<ThemeExportLayout>,
    ) -> Self {
        Self {
            ferriskey: Self::KIND.to_string(),
            version: FORMAT_VERSION,
            name,
            config,
            pages,
            layout,
        }
    }

    /// Same download shape as a builder export: pretty JSON under a filename
    /// derived from the theme's name.
    pub fn into_download(self) -> Result<Response, serde_json::Error> {
        let filename = format!("{}.json", slugify(&self.name));
        let body = serde_json::to_vec_pretty(&self)?;

        Ok(download(body, "application/json", &filename))
    }
}

/// Refuses an import whose envelope names another builder or a format from the
/// future. Both fields are optional: a payload assembled by hand, carrying only
/// a name and a tree, stays importable — the check only holds an exported file
/// to what it claims to be.
pub fn ensure_importable(
    ferriskey: Option<&str>,
    version: Option<u32>,
    expected_kind: &str,
) -> Result<(), ApiError> {
    if let Some(kind) = ferriskey
        && kind != expected_kind
    {
        return Err(ApiError::BadRequest(
            format!("expected a {expected_kind} export, but this file is a {kind} export").into(),
        ));
    }

    if let Some(version) = version
        && version > FORMAT_VERSION
    {
        return Err(ApiError::BadRequest(
            format!(
                "this file uses export format version {version}, which this server does not \
                 understand — it reads up to version {FORMAT_VERSION}"
            )
            .into(),
        ));
    }

    Ok(())
}

/// A response the browser saves to a file rather than renders. `filename` is
/// used as given, so callers pass a [`slugify`]ed name.
pub fn download(body: Vec<u8>, content_type: &'static str, filename: &str) -> Response {
    let mut response = (StatusCode::OK, body).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    headers.insert(header::CONTENT_DISPOSITION, content_disposition(filename));
    response
}

/// `Content-Disposition` for a download. The filename is already slugified, so
/// it never needs escaping; a header that somehow fails to build falls back to
/// a plain `attachment`.
pub fn content_disposition(filename: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_collapses_separators() {
        assert_eq!(slugify("Reset password"), "reset-password");
        assert_eq!(slugify("  Welcome — v2 !!"), "welcome-v2");
        assert_eq!(slugify("Réinitialisation"), "r-initialisation");
    }

    #[test]
    fn slugify_falls_back_when_nothing_survives() {
        assert_eq!(slugify("///"), "export");
        assert_eq!(slugify(""), "export");
    }

    #[test]
    fn envelope_omits_email_type_for_layouts() {
        let envelope = ExportEnvelope::portal_layout("Default".to_string(), serde_json::json!([]));

        let json = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(json["ferriskey"], "portal-layout");
        assert!(json.get("email_type").is_none());
    }

    #[test]
    fn an_email_template_envelope_carries_its_type() {
        let envelope = ExportEnvelope::email_template(
            "Reset password".to_string(),
            "reset_password".to_string(),
            serde_json::json!([]),
        );

        let json = serde_json::to_value(&envelope).expect("serialize");
        assert_eq!(json["ferriskey"], "email-template");
        assert_eq!(json["email_type"], "reset_password");
    }

    #[test]
    fn an_import_refuses_a_file_from_the_other_builder() {
        let refused = ensure_importable(
            Some(ExportEnvelope::EMAIL_TEMPLATE),
            Some(1),
            ExportEnvelope::PORTAL_LAYOUT,
        );

        assert!(
            refused.is_err(),
            "an email template must not import as a layout"
        );
    }

    #[test]
    fn an_import_refuses_a_format_it_cannot_read() {
        assert!(
            ensure_importable(
                None,
                Some(FORMAT_VERSION + 1),
                ExportEnvelope::PORTAL_LAYOUT
            )
            .is_err()
        );
    }

    #[test]
    fn a_hand_written_payload_carries_no_envelope_and_is_still_accepted() {
        assert!(ensure_importable(None, None, ExportEnvelope::PORTAL_LAYOUT).is_ok());
        assert!(
            ensure_importable(
                Some(ExportEnvelope::PORTAL_LAYOUT),
                Some(FORMAT_VERSION),
                ExportEnvelope::PORTAL_LAYOUT
            )
            .is_ok()
        );
    }

    #[test]
    fn a_download_names_its_file_after_the_envelope() {
        let response =
            ExportEnvelope::portal_layout("Reset password".to_string(), serde_json::json!([]))
                .into_download()
                .expect("serialize");

        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_DISPOSITION)
                .expect("a download must name its file"),
            "attachment; filename=\"reset-password.json\""
        );
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_TYPE)
                .expect("a download must declare its type"),
            "application/json"
        );
    }
}
