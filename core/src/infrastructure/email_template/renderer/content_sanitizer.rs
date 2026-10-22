use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use ammonia::{Builder, UrlRelative};
use serde_json::{Map, Value};

use crate::domain::common::entities::app_errors::CoreError;

pub const MAX_DEPTH: usize = 32;

const TEXT_TAGS: [&str; 30] = [
    "p",
    "br",
    "hr",
    "strong",
    "b",
    "em",
    "i",
    "u",
    "s",
    "a",
    "ul",
    "ol",
    "li",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "blockquote",
    "code",
    "pre",
    "span",
    "table",
    "thead",
    "tbody",
    "tfoot",
    "tr",
    "td",
    "th",
];
const CELL_ATTRIBUTES: [&str; 5] = ["colspan", "rowspan", "align", "valign", "width"];
const URL_SCHEMES: [&str; 4] = ["http", "https", "mailto", "tel"];
const STYLE_PROPERTIES: [&str; 8] = [
    "color",
    "font-size",
    "font-weight",
    "font-style",
    "font-family",
    "text-align",
    "line-height",
    "text-decoration",
];
const FORBIDDEN_STYLE_FRAGMENTS: [&str; 6] = [
    "url(",
    "expression(",
    "@import",
    "javascript:",
    "behavior",
    "binding",
];
const FORBIDDEN_STYLE_CHARS: [char; 5] = ['<', '>', '{', '}', '\\'];
const FORBIDDEN_STYLESHEET_CHARS: [char; 2] = ['<', '\\'];

static TEXT_SANITIZER: LazyLock<Builder<'static>> = LazyLock::new(|| {
    let mut builder = Builder::default();
    builder
        .tags(HashSet::from(TEXT_TAGS))
        .tag_attributes(HashMap::from([
            ("a", HashSet::from(["href", "target"])),
            (
                "table",
                HashSet::from(["width", "align", "border", "cellpadding", "cellspacing"]),
            ),
            ("td", HashSet::from(CELL_ATTRIBUTES)),
            ("th", HashSet::from(CELL_ATTRIBUTES)),
        ]))
        .generic_attributes(HashSet::from(["style"]))
        .url_schemes(HashSet::from(URL_SCHEMES))
        .url_relative(UrlRelative::PassThrough)
        .link_rel(Some("noopener noreferrer"))
        .strip_comments(true)
        .attribute_filter(|_, attribute, value| {
            if attribute != "style" {
                return Some(Cow::Borrowed(value));
            }
            let cleaned = clean_style(value);
            (!cleaned.is_empty()).then_some(Cow::Owned(cleaned))
        });
    builder
});

pub fn sanitize_structure(structure: &Value) -> Result<Value, CoreError> {
    sanitize_node(structure, 0)
}

pub fn clean_html(html: &str) -> String {
    TEXT_SANITIZER.clean(html).to_string()
}

pub fn is_attribute_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '-')
        && !name.to_ascii_lowercase().starts_with("on")
}

pub fn escape_attribute_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '"' => escaped.push_str("&quot;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn sanitize_node(node: &Value, depth: usize) -> Result<Value, CoreError> {
    if depth >= MAX_DEPTH {
        return Err(CoreError::InvalidEmailTemplateStructure(format!(
            "nesting deeper than {MAX_DEPTH} levels"
        )));
    }
    let Value::Object(fields) = node else {
        return Ok(node.clone());
    };
    let node_type = node_type_of(fields)?;
    let mut clean = Map::with_capacity(fields.len());
    for (key, value) in fields {
        let value = match key.as_str() {
            "attributes" | "props" | "styles" => sanitize_attributes(value),
            "content" => sanitize_content(node_type, value),
            "children" => sanitize_children(value, depth)?,
            "head" => sanitize_node(value, depth + 1)?,
            _ => value.clone(),
        };
        clean.insert(key.clone(), value);
    }
    Ok(Value::Object(clean))
}

fn node_type_of(fields: &Map<String, Value>) -> Result<Option<&str>, CoreError> {
    match fields.get("type") {
        None => Ok(None),
        Some(Value::String(node_type)) if is_node_type(node_type) => Ok(Some(node_type)),
        Some(Value::String(node_type)) => Err(CoreError::InvalidEmailTemplateStructure(format!(
            "unsupported node type '{node_type}'"
        ))),
        Some(_) => Err(CoreError::InvalidEmailTemplateStructure(
            "'type' must be a string".to_string(),
        )),
    }
}

fn is_node_type(node_type: &str) -> bool {
    node_type == "mjml"
        || node_type == "root"
        || node_type.strip_prefix("mj-").is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        })
}

fn sanitize_children(value: &Value, depth: usize) -> Result<Value, CoreError> {
    match value {
        Value::Array(children) => children
            .iter()
            .map(|child| sanitize_node(child, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        other => Ok(other.clone()),
    }
}

fn sanitize_attributes(value: &Value) -> Value {
    match value {
        Value::Object(attributes) => Value::Object(
            attributes
                .iter()
                .filter(|(name, value)| is_attribute_name(name) && is_safe_attribute(name, value))
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn is_safe_attribute(name: &str, value: &Value) -> bool {
    match value {
        Value::String(url) if is_url_attribute(name) => is_allowed_url(url),
        _ => true,
    }
}

fn is_url_attribute(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name == "href" || name == "src" || name.ends_with("-url") || name.ends_with("-src")
}

fn is_allowed_url(url: &str) -> bool {
    let compact: String = url.chars().filter(|c| *c as u32 > 0x20).collect();
    match scheme_of(&compact) {
        None => true,
        Some(scheme) => URL_SCHEMES.contains(&scheme.to_ascii_lowercase().as_str()),
    }
}

fn scheme_of(url: &str) -> Option<&str> {
    let (scheme, _) = url.split_once(':')?;
    let mut chars = scheme.chars();
    let first = chars.next()?;
    (first.is_ascii_alphabetic()
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')))
    .then_some(scheme)
}

fn sanitize_content(node_type: Option<&str>, value: &Value) -> Value {
    match value {
        Value::String(css) if node_type == Some("mj-style") => Value::String(clean_stylesheet(css)),
        Value::String(rows) if node_type == Some("mj-table") => {
            Value::String(clean_table_rows(rows))
        }
        Value::String(html) => Value::String(clean_html(html)),
        other => other.clone(),
    }
}

fn clean_table_rows(rows: &str) -> String {
    let cleaned = clean_html(&format!("<table>{rows}</table>"));
    cleaned
        .strip_prefix("<table>")
        .and_then(|inner| inner.strip_suffix("</table>"))
        .unwrap_or(&cleaned)
        .to_string()
}

fn clean_stylesheet(css: &str) -> String {
    if has_forbidden_style_construct(css, &FORBIDDEN_STYLESHEET_CHARS) {
        String::new()
    } else {
        css.to_string()
    }
}

fn clean_style(style: &str) -> String {
    style
        .split(';')
        .filter_map(|declaration| {
            let (property, value) = declaration.split_once(':')?;
            let property = property.trim().to_ascii_lowercase();
            let value = value.trim();
            (STYLE_PROPERTIES.contains(&property.as_str())
                && !value.is_empty()
                && !has_forbidden_style_construct(value, &FORBIDDEN_STYLE_CHARS))
            .then(|| format!("{property}: {value}"))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn has_forbidden_style_construct(text: &str, forbidden_chars: &[char]) -> bool {
    let compact = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    FORBIDDEN_STYLE_FRAGMENTS
        .iter()
        .any(|fragment| compact.contains(fragment))
        || text.contains(forbidden_chars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn text_node(content: &str) -> serde_json::Value {
        json!({"type": "mj-text", "content": content})
    }

    fn content_of(node: &serde_json::Value) -> String {
        node["content"].as_str().unwrap_or_default().to_string()
    }

    fn sanitized_content(content: &str) -> String {
        content_of(&sanitize_structure(&text_node(content)).expect("sanitized"))
    }

    #[test]
    fn scripts_and_event_handlers_are_removed_from_text_content() {
        let content = sanitized_content(
            r#"<p onclick="steal()">Hi</p><script>alert(1)</script><img src="x" onerror="alert(1)">"#,
        );
        assert_eq!(content, "<p>Hi</p>");
    }

    #[test]
    fn unsafe_links_lose_their_target_while_placeholders_and_web_links_stay() {
        let content = sanitized_content(
            r#"<a href="javascript:alert(1)">x</a><a href="{{reset_link}}">r</a><a href="https://ferriskey.rs" target="_blank">f</a>"#,
        );
        assert!(!content.contains("javascript:"), "{content}");
        assert!(content.contains(r#"href="{{reset_link}}""#), "{content}");
        assert!(
            content.contains(r#"href="https://ferriskey.rs""#),
            "{content}"
        );
        assert!(
            content.contains(r#"rel="noopener noreferrer""#),
            "{content}"
        );
    }

    #[test]
    fn inline_styles_keep_only_the_allowed_properties() {
        let content = sanitized_content(
            r#"<span style="color: red; background: url(evil); position: fixed">a</span><b style="position: fixed">b</b>"#,
        );
        assert!(
            content.contains(r#"<span style="color: red">a</span>"#),
            "{content}"
        );
        assert!(content.contains("<b>b</b>"), "{content}");
        assert!(!content.contains("url("), "{content}");
    }

    #[test]
    fn placeholders_and_entities_survive() {
        let content = sanitized_content("<p>Hello&nbsp;{{user.first_name}} &amp; co</p>");
        assert!(content.contains("{{user.first_name}}"), "{content}");
        assert!(content.contains("&amp; co"), "{content}");
    }

    #[test]
    fn table_rows_are_kept_for_table_blocks() {
        let table = json!({
            "type": "mj-table",
            "content": r#"<tr><td colspan="2" onclick="x">a</td><th align="left">b</th></tr><script>x</script>"#
        });
        let content = content_of(&sanitize_structure(&table).expect("sanitized"));
        assert!(content.contains(r#"<td colspan="2">a</td>"#), "{content}");
        assert!(content.contains(r#"<th align="left">b</th>"#), "{content}");
        assert!(!content.contains("<table"), "{content}");
        assert!(!content.contains("script"), "{content}");
    }

    #[test]
    fn node_attributes_with_unsafe_urls_or_handlers_are_dropped() {
        let button = json!({
            "type": "mj-button",
            "attributes": {"href": "javascript:alert(1)", "onclick": "steal()", "background-color": "#000", "x\" y": "z"},
            "content": "Go"
        });
        let clean = sanitize_structure(&button).expect("sanitized");
        assert_eq!(clean["attributes"], json!({"background-color": "#000"}));

        let image = json!({
            "type": "mj-image",
            "props": {"src": "data:text/html,boom", "href": "{{reset_link}}"},
            "styles": {"width": "100px"}
        });
        let clean = sanitize_structure(&image).expect("sanitized");
        assert_eq!(clean["props"], json!({"href": "{{reset_link}}"}));
        assert_eq!(clean["styles"], json!({"width": "100px"}));
    }

    #[test]
    fn web_mailto_and_relative_urls_are_kept() {
        for url in [
            "https://ferriskey.rs/a?b=1&c=2",
            "HTTP://x",
            "mailto:a@b.c",
            "/relative/path",
            "{{reset_link}}",
        ] {
            let node = json!({"type": "mj-button", "attributes": {"href": url}});
            let clean = sanitize_structure(&node).expect("sanitized");
            assert_eq!(clean["attributes"]["href"], url, "{url}");
        }
    }

    #[test]
    fn unknown_node_types_are_refused() {
        for node_type in ["script", "mj-raw><script>", "MJ-TEXT", ""] {
            let result = sanitize_structure(&json!({"type": node_type}));
            assert!(
                matches!(result, Err(CoreError::InvalidEmailTemplateStructure(_))),
                "{node_type}"
            );
        }
        assert!(sanitize_structure(&json!({"type": 42})).is_err());
    }

    #[test]
    fn nesting_deeper_than_the_limit_is_refused() {
        let mut node = text_node("deep");
        for _ in 0..MAX_DEPTH {
            node = json!({"type": "mj-section", "children": [node]});
        }
        assert!(matches!(
            sanitize_structure(&node),
            Err(CoreError::InvalidEmailTemplateStructure(_))
        ));

        let mut node = text_node("ok");
        for _ in 0..8 {
            node = json!({"type": "mj-section", "children": [node]});
        }
        assert!(sanitize_structure(&node).is_ok());
    }

    #[test]
    fn style_blocks_with_forbidden_constructs_are_emptied() {
        let safe = json!({"type": "mj-style", "content": "a { color: red }"});
        assert_eq!(
            content_of(&sanitize_structure(&safe).expect("sanitized")),
            "a { color: red }"
        );
        for css in [
            "a { color: red } </mj-style><mj-raw>x",
            "a { background: url(x) }",
            "@import 'x';",
            "a { behavior: url(x) }",
            "a { color: expression(alert(1)) }",
            "a { b: \\41 }",
        ] {
            let node = json!({"type": "mj-style", "content": css});
            assert_eq!(
                content_of(&sanitize_structure(&node).expect("sanitized")),
                "",
                "{css}"
            );
        }
    }

    #[test]
    fn children_and_head_are_sanitized_recursively() {
        let structure = json!({
            "children": [{"type": "mj-section", "children": [{"type": "mj-column", "children": [text_node("<p>ok</p><script>x</script>")]}]}],
            "head": {"type": "mj-head", "children": [{"type": "mj-title", "content": "<b>t</b><script>x</script>"}]}
        });
        let clean = sanitize_structure(&structure).expect("sanitized");
        assert_eq!(
            clean["children"][0]["children"][0]["children"][0]["content"],
            "<p>ok</p>"
        );
        assert_eq!(clean["head"]["children"][0]["content"], "<b>t</b>");
    }

    #[test]
    fn attribute_values_are_escaped_for_markup() {
        assert_eq!(
            escape_attribute_value(r#"#fff" onload="x<>&amp;"#),
            "#fff&quot; onload=&quot;x&lt;&gt;&amp;"
        );
    }

    #[test]
    fn attribute_names_follow_the_markup_grammar() {
        assert!(is_attribute_name("background-color"));
        assert!(is_attribute_name("css-class"));
        assert!(!is_attribute_name("onclick"));
        assert!(!is_attribute_name("x\" y"));
        assert!(!is_attribute_name("-leading"));
        assert!(!is_attribute_name(""));
    }
}
