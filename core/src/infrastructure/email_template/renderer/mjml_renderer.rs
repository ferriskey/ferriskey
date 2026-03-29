use crate::domain::{
    common::entities::app_errors::CoreError, email_template::ports::TemplateRenderer,
};

/// MJML renderer that converts builder JSON structure into MJML,
/// then MJML into HTML using the `mrml` crate.
#[derive(Debug, Clone)]
pub struct MjmlTemplateRenderer;

impl MjmlTemplateRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TemplateRenderer for MjmlTemplateRenderer {
    fn render_to_intermediate(&self, structure: &serde_json::Value) -> Result<String, CoreError> {
        json_to_mjml(structure)
    }

    fn render_to_html(&self, intermediate: &str) -> Result<String, CoreError> {
        let opts = mrml::prelude::render::RenderOptions::default();
        let parsed = mrml::parse(intermediate)
            .map_err(|e| CoreError::EmailTemplateRenderError(format!("MJML parse error: {e}")))?;
        let html = parsed
            .render(&opts)
            .map_err(|e| CoreError::EmailTemplateRenderError(format!("MJML render error: {e}")))?;
        Ok(html)
    }
}

/// Converts a builder JSON structure into an MJML string.
///
/// The JSON structure follows a tree format:
/// ```json
/// {
///   "type": "mj-body",
///   "attributes": { "background-color": "#ffffff" },
///   "children": [
///     {
///       "type": "mj-section",
///       "attributes": {},
///       "children": [
///         {
///           "type": "mj-column",
///           "children": [
///             {
///               "type": "mj-text",
///               "attributes": { "font-size": "20px" },
///               "content": "<p>Hello {{user.first_name}}</p>"
///             }
///           ]
///         }
///       ]
///     }
///   ]
/// }
/// ```
fn json_to_mjml(node: &serde_json::Value) -> Result<String, CoreError> {
    // Support wrapper object: { children: [...] } without a type (root from frontend)
    if node.get("type").is_none() {
        if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
            let body_children = children
                .iter()
                .map(json_to_mjml)
                .collect::<Result<Vec<String>, CoreError>>()?
                .join("");
            return Ok(format!("<mjml><mj-body>{body_children}</mj-body></mjml>"));
        }
        return Err(CoreError::InvalidEmailTemplateStructure(
            "missing 'type' field in node".to_string(),
        ));
    }

    let node_type = node["type"].as_str().ok_or_else(|| {
        CoreError::InvalidEmailTemplateStructure("'type' must be a string".to_string())
    })?;

    // Build attributes from "attributes", "props", or "styles" objects
    let mut attr_parts = Vec::new();
    for key in &["attributes", "props", "styles"] {
        if let Some(obj) = node.get(*key).and_then(|v| v.as_object()) {
            for (k, v) in obj {
                let val = match v {
                    serde_json::Value::String(s) if !s.is_empty() => s.clone(),
                    serde_json::Value::String(_) => continue,
                    serde_json::Value::Null => continue,
                    other => other.to_string(),
                };
                attr_parts.push(format!(" {k}=\"{val}\""));
            }
        }
    }
    let attrs = attr_parts.join("");

    // Get content (for leaf nodes like mj-text, mj-button)
    let content = node.get("content").and_then(|v| v.as_str()).unwrap_or("");

    // Get children
    let children = if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        children
            .iter()
            .map(json_to_mjml)
            .collect::<Result<Vec<String>, CoreError>>()?
            .join("")
    } else {
        String::new()
    };

    // Root node wraps in <mjml>
    if node_type == "mjml" || node_type == "root" {
        let head = node
            .get("head")
            .map(json_to_mjml)
            .transpose()?
            .unwrap_or_default();
        return Ok(format!("<mjml>{head}{children}</mjml>"));
    }

    if node_type == "mj-head" {
        return Ok(format!("<mj-head>{children}</mj-head>"));
    }

    // Self-closing tags (no children, no content)
    let self_closing = matches!(node_type, "mj-divider" | "mj-spacer" | "mj-image");
    if self_closing && content.is_empty() && children.is_empty() {
        return Ok(format!("<{node_type}{attrs} />"));
    }

    Ok(format!(
        "<{node_type}{attrs}>{content}{children}</{node_type}>"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_structure_to_mjml() {
        let structure = json!({
            "type": "mjml",
            "children": [
                {
                    "type": "mj-body",
                    "children": [
                        {
                            "type": "mj-section",
                            "children": [
                                {
                                    "type": "mj-column",
                                    "children": [
                                        {
                                            "type": "mj-text",
                                            "attributes": {
                                                "font-size": "20px",
                                                "color": "#333333"
                                            },
                                            "content": "Hello World"
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let renderer = MjmlTemplateRenderer::new();
        let mjml = renderer.render_to_intermediate(&structure).unwrap();

        assert!(mjml.contains("<mjml>"));
        assert!(mjml.contains("<mj-body>"));
        assert!(mjml.contains("<mj-section>"));
        assert!(mjml.contains("<mj-text"));
        assert!(mjml.contains("Hello World"));
        assert!(mjml.contains("font-size=\"20px\""));
    }

    #[test]
    fn test_mjml_to_html() {
        let mjml = r#"<mjml><mj-body><mj-section><mj-column><mj-text>Hello</mj-text></mj-column></mj-section></mj-body></mjml>"#;

        let renderer = MjmlTemplateRenderer::new();
        let html = renderer.render_to_html(mjml).unwrap();

        assert!(html.contains("Hello"));
        assert!(html.contains("<!doctype html>") || html.contains("<html"));
    }

    #[test]
    fn test_self_closing_tags() {
        let structure = json!({
            "type": "mj-divider",
            "attributes": {
                "border-color": "#cccccc"
            }
        });

        let mjml = json_to_mjml(&structure).unwrap();
        assert!(mjml.contains("<mj-divider"));
        assert!(mjml.contains("/>"));
    }

    #[test]
    fn test_invalid_mjml_returns_error() {
        let renderer = MjmlTemplateRenderer::new();
        let result = renderer.render_to_html("<invalid>not mjml</invalid>");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_type_returns_error() {
        let structure = json!({"content": "no type"});
        let result = json_to_mjml(&structure);
        assert!(result.is_err());
    }

    #[test]
    fn test_full_roundtrip() {
        let structure = json!({
            "type": "mjml",
            "children": [
                {
                    "type": "mj-body",
                    "attributes": {"background-color": "#f4f4f4"},
                    "children": [
                        {
                            "type": "mj-section",
                            "attributes": {"background-color": "#ffffff"},
                            "children": [
                                {
                                    "type": "mj-column",
                                    "children": [
                                        {
                                            "type": "mj-text",
                                            "attributes": {"font-size": "16px"},
                                            "content": "<p>Hello {{user.first_name}},</p><p>Click below to reset your password.</p>"
                                        },
                                        {
                                            "type": "mj-button",
                                            "attributes": {
                                                "href": "{{reset_link}}",
                                                "background-color": "#007bff"
                                            },
                                            "content": "Reset Password"
                                        },
                                        {
                                            "type": "mj-divider",
                                            "attributes": {"border-color": "#eeeeee"}
                                        }
                                    ]
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let renderer = MjmlTemplateRenderer::new();
        let mjml = renderer.render_to_intermediate(&structure).unwrap();
        let html = renderer.render_to_html(&mjml).unwrap();

        assert!(html.contains("{{user.first_name}}"));
        assert!(html.contains("{{reset_link}}"));
        assert!(html.contains("Reset Password"));
    }

    #[test]
    fn test_frontend_builder_format() {
        // Format sent by the frontend builder: { children: [...] } with props/styles
        let structure = json!({
            "children": [
                {
                    "id": "node-123",
                    "type": "mj-section",
                    "props": {},
                    "styles": {},
                    "children": [
                        {
                            "id": "node-456",
                            "type": "mj-column",
                            "props": {},
                            "styles": {},
                            "children": [
                                {
                                    "id": "node-789",
                                    "type": "mj-text",
                                    "props": {
                                        "color": "#333333",
                                        "font-size": "14px"
                                    },
                                    "styles": {},
                                    "children": [],
                                    "content": "<p>Hello {{user.first_name}}</p>"
                                }
                            ]
                        }
                    ]
                }
            ]
        });

        let renderer = MjmlTemplateRenderer::new();
        let mjml = renderer.render_to_intermediate(&structure).unwrap();
        assert!(mjml.contains("<mjml>"));
        assert!(mjml.contains("<mj-body>"));
        assert!(mjml.contains("<mj-text"));
        assert!(mjml.contains("color=\"#333333\""));
        assert!(mjml.contains("Hello {{user.first_name}}"));

        let html = renderer.render_to_html(&mjml).unwrap();
        assert!(html.contains("Hello {{user.first_name}}"));
    }
}
