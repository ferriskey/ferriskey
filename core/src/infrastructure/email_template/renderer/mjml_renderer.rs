use crate::domain::{
    common::entities::app_errors::CoreError, email_template::ports::TemplateRenderer,
};

/// MJML renderer that converts builder JSON structure into MJML,
/// then MJML into HTML using the `mrml` crate.
#[derive(Debug, Clone, Default)]
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

    fn parse_intermediate(&self, intermediate: &str) -> Result<serde_json::Value, CoreError> {
        mjml_to_json(intermediate)
    }
}

/// Converts an MJML document into the builder JSON structure, the inverse of
/// [`json_to_mjml`].
///
/// `mrml` parses (and thereby validates) the markup, and its AST serializes to
/// `{"type": "mj-section", "attributes": {…}, "children": […]}` — one rename away
/// from a builder node. Only the `<mj-body>` subtree is representable in the
/// builder, so `<mj-head>` and its global attributes are dropped.
fn mjml_to_json(mjml: &str) -> Result<serde_json::Value, CoreError> {
    // The markup is supplied by the caller, so a parse failure is bad input,
    // not a server fault — `InvalidEmailTemplateStructure` is the 4xx variant.
    let parsed = mrml::parse(mjml)
        .map_err(|e| CoreError::InvalidEmailTemplateStructure(format!("MJML parse error: {e}")))?;

    let document = serde_json::to_value(&parsed).map_err(|e| {
        CoreError::EmailTemplateRenderError(format!("MJML serialization error: {e}"))
    })?;

    let body = document
        .get("children")
        .and_then(|children| children.as_array())
        .and_then(|children| {
            children
                .iter()
                .find(|child| child.get("type").and_then(|t| t.as_str()) == Some("mj-body"))
        })
        .ok_or_else(|| {
            CoreError::InvalidEmailTemplateStructure("MJML document has no <mj-body>".to_string())
        })?;

    let children = node_children(body)
        .iter()
        .filter_map(mrml_node_to_builder_node)
        .collect::<Vec<_>>();

    Ok(serde_json::json!({ "children": children }))
}

fn node_children(node: &serde_json::Value) -> &[serde_json::Value] {
    node.get("children")
        .and_then(|children| children.as_array())
        .map(|children| children.as_slice())
        .unwrap_or_default()
}

/// Maps one MJML element onto a builder node. Non-MJML children (raw HTML, bare
/// text) are folded back into `content`, which is where the builder keeps them.
fn mrml_node_to_builder_node(node: &serde_json::Value) -> Option<serde_json::Value> {
    let node_type = node.get("type")?.as_str()?;
    if !node_type.starts_with("mj-") {
        return None;
    }

    let mut children = Vec::new();
    let mut content = String::new();

    for child in node_children(node) {
        match child {
            serde_json::Value::String(text) => content.push_str(text),
            serde_json::Value::Object(_) => {
                let child_type = child
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or_default();
                if child_type.starts_with("mj-") {
                    if let Some(child_node) = mrml_node_to_builder_node(child) {
                        children.push(child_node);
                    }
                } else if child_type != "comment" {
                    content.push_str(&raw_node_to_html(child));
                }
            }
            _ => {}
        }
    }

    let mut builder_node = serde_json::Map::new();
    builder_node.insert(
        "id".to_string(),
        serde_json::Value::String(uuid::Uuid::new_v4().to_string()),
    );
    builder_node.insert(
        "type".to_string(),
        serde_json::Value::String(node_type.to_string()),
    );
    builder_node.insert(
        "props".to_string(),
        node.get("attributes")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
    );
    builder_node.insert("styles".to_string(), serde_json::json!({}));
    builder_node.insert("children".to_string(), serde_json::Value::Array(children));
    if !content.is_empty() {
        builder_node.insert("content".to_string(), serde_json::Value::String(content));
    }

    Some(serde_json::Value::Object(builder_node))
}

/// Re-serializes a raw HTML node from the mrml AST back into markup.
fn raw_node_to_html(node: &serde_json::Value) -> String {
    let Some(tag) = node.get("type").and_then(|t| t.as_str()) else {
        return String::new();
    };

    let attrs = node
        .get("attributes")
        .and_then(|attrs| attrs.as_object())
        .map(|attrs| {
            attrs
                .iter()
                .map(|(key, value)| match value.as_str() {
                    Some(value) => format!(" {key}=\"{value}\""),
                    None => format!(" {key}"),
                })
                .collect::<String>()
        })
        .unwrap_or_default();

    // `children` is a string for comments, an array for every other node.
    let inner = match node.get("children") {
        Some(serde_json::Value::String(text)) => text.clone(),
        _ => node_children(node)
            .iter()
            .map(|child| match child {
                serde_json::Value::String(text) => text.clone(),
                serde_json::Value::Object(_) => raw_node_to_html(child),
                _ => String::new(),
            })
            .collect::<String>(),
    };

    // HTML void elements have no closing tag; emitting one would nest the rest
    // of the content inside them.
    const VOID_ELEMENTS: [&str; 6] = ["br", "hr", "img", "input", "meta", "link"];
    if VOID_ELEMENTS.contains(&tag) {
        return format!("<{tag}{attrs} />");
    }

    format!("<{tag}{attrs}>{inner}</{tag}>")
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

    #[test]
    fn test_mjml_to_json_builds_builder_nodes() {
        let mjml = r##"<mjml><mj-body><mj-section background-color="#ffffff"><mj-column><mj-text font-size="16px">Hello {{user.first_name}}</mj-text></mj-column></mj-section></mj-body></mjml>"##;

        let structure = mjml_to_json(mjml).expect("valid markup parses");
        let children = structure["children"]
            .as_array()
            .expect("a parsed body carries its children");

        assert_eq!(children.len(), 1);
        let section = &children[0];
        assert_eq!(section["type"], "mj-section");
        assert_eq!(section["props"]["background-color"], "#ffffff");
        assert!(section["id"].as_str().is_some_and(|id| !id.is_empty()));
        assert_eq!(section["styles"], json!({}));

        let text = &section["children"][0]["children"][0];
        assert_eq!(text["type"], "mj-text");
        assert_eq!(text["props"]["font-size"], "16px");
        assert_eq!(text["content"], "Hello {{user.first_name}}");
    }

    #[test]
    fn test_mjml_to_json_keeps_inline_html_as_content() {
        let mjml = r#"<mjml><mj-body><mj-section><mj-column><mj-text><p>Hello <b>world</b></p><br /></mj-text></mj-column></mj-section></mj-body></mjml>"#;

        let structure = mjml_to_json(mjml).expect("valid markup parses");
        let text = &structure["children"][0]["children"][0]["children"][0];

        assert_eq!(text["type"], "mj-text");
        assert_eq!(text["content"], "<p>Hello <b>world</b></p><br />");
        assert_eq!(text["children"], json!([]));
    }

    #[test]
    fn test_mjml_to_json_roundtrips_through_json_to_mjml() {
        let original = r##"<mjml><mj-body><mj-section><mj-column><mj-text font-size="14px">Hi</mj-text><mj-divider border-color="#eeeeee" /></mj-column></mj-section></mj-body></mjml>"##;

        let structure = mjml_to_json(original).expect("valid markup parses");
        let rendered = json_to_mjml(&structure).expect("a parsed tree renders back");

        // Re-parsing the rendered MJML yields the same tree, ids aside.
        let reparsed = mjml_to_json(&rendered).expect("rendered markup parses again");
        assert_eq!(strip_ids(&structure), strip_ids(&reparsed));

        let renderer = MjmlTemplateRenderer::new();
        assert!(
            renderer
                .render_to_html(&rendered)
                .expect("rendered markup compiles to html")
                .contains("Hi")
        );
    }

    #[test]
    fn test_mjml_to_json_rejects_invalid_markup() {
        assert!(mjml_to_json("<invalid>not mjml</invalid>").is_err());
    }

    #[test]
    fn test_mjml_to_json_requires_a_body() {
        assert!(mjml_to_json("<mjml><mj-head></mj-head></mjml>").is_err());
    }

    fn strip_ids(value: &serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Object(map) => serde_json::Value::Object(
                map.iter()
                    .filter(|(key, _)| key.as_str() != "id")
                    .map(|(key, value)| (key.clone(), strip_ids(value)))
                    .collect(),
            ),
            serde_json::Value::Array(items) => {
                serde_json::Value::Array(items.iter().map(strip_ids).collect())
            }
            other => other.clone(),
        }
    }
}
