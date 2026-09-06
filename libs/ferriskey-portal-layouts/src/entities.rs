use chrono::{DateTime, Utc};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::realm::RealmId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct PortalLayout {
    pub id: Uuid,
    pub realm_id: RealmId,
    pub name: String,
    #[schema(value_type = Object)]
    pub tree: serde_json::Value,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Checks that a layout tree is a well-formed builder tree before it is stored.
///
/// The builder itself only ever produces valid trees, so this exists for
/// externally supplied ones (imports): a malformed tree would otherwise be
/// accepted here and only blow up in the portal renderer.
pub fn validate_tree(tree: &serde_json::Value) -> Result<(), CoreError> {
    let nodes = tree.as_array().ok_or_else(|| {
        CoreError::PortalLayoutInvalidTree("tree must be an array of nodes".to_string())
    })?;

    for (index, node) in nodes.iter().enumerate() {
        validate_node(node, &format!("tree[{index}]"))?;
    }

    Ok(())
}

fn validate_node(node: &serde_json::Value, path: &str) -> Result<(), CoreError> {
    let node = node
        .as_object()
        .ok_or_else(|| CoreError::PortalLayoutInvalidTree(format!("{path} must be an object")))?;

    let node_type = node.get("type").and_then(|value| value.as_str());
    if node_type.is_none_or(str::is_empty) {
        return Err(CoreError::PortalLayoutInvalidTree(format!(
            "{path} is missing a non-empty 'type'"
        )));
    }

    // `children` is required, not merely well-typed when present: the builder
    // walks a tree assuming every node has one (`node.children.length`), so a
    // node without it is accepted here and then crashes the editor the first
    // time someone opens the imported layout.
    match node.get("children") {
        None => Err(CoreError::PortalLayoutInvalidTree(format!(
            "{path} is missing 'children'; a node with no child still needs an empty array"
        ))),
        Some(serde_json::Value::Array(children)) => {
            for (index, child) in children.iter().enumerate() {
                validate_node(child, &format!("{path}.children[{index}]"))?;
            }
            Ok(())
        }
        Some(_) => Err(CoreError::PortalLayoutInvalidTree(format!(
            "{path}.children must be an array"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tree() -> serde_json::Value {
        serde_json::json!([
            {
                "id": "root",
                "type": "container",
                "props": { "direction": "column" },
                "styles": {},
                "children": [
                    { "id": "slot", "type": "page-content", "props": {}, "styles": {}, "children": [] }
                ]
            }
        ])
    }

    #[test]
    fn round_trips_through_json() {
        let original = PortalLayout {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4().into(),
            name: "Default layout".to_string(),
            tree: sample_tree(),
            is_default: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: PortalLayout = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, parsed);
    }

    #[test]
    fn tree_accepts_arbitrary_nested_json() {
        let tree = serde_json::json!([
            { "id": "a", "type": "container", "children": [
                { "id": "b", "type": "heading", "content": "Hi" }
            ]}
        ]);
        let layout = PortalLayout {
            id: Uuid::new_v4(),
            realm_id: Uuid::new_v4().into(),
            name: "n".to_string(),
            tree,
            is_default: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_value(&layout).expect("to_value");
        assert!(json.get("tree").unwrap().is_array());
    }

    #[test]
    fn validate_tree_accepts_a_nested_builder_tree() {
        assert!(validate_tree(&sample_tree()).is_ok());
        assert!(validate_tree(&serde_json::json!([])).is_ok());
    }

    #[test]
    fn validate_tree_rejects_non_array_roots() {
        let result = validate_tree(&serde_json::json!({ "children": [] }));
        assert!(matches!(result, Err(CoreError::PortalLayoutInvalidTree(_))));
    }

    #[test]
    fn validate_tree_rejects_a_node_without_children() {
        let tree = serde_json::json!([
            { "type": "container", "children": [{ "type": "text", "props": {} }] }
        ]);

        let error = validate_tree(&tree).expect_err("a node without children must be refused");
        assert!(
            format!("{error:?}").contains("children"),
            "the message should name the missing field: {error:?}"
        );
    }

    #[test]
    fn validate_tree_rejects_nodes_without_a_type() {
        let result = validate_tree(&serde_json::json!([{ "id": "a", "children": [] }]));
        assert!(matches!(result, Err(CoreError::PortalLayoutInvalidTree(_))));
    }

    #[test]
    fn validate_tree_reports_the_path_of_a_bad_descendant() {
        let tree = serde_json::json!([
            { "type": "container", "children": [{ "type": "row", "children": "nope" }] }
        ]);

        let Err(CoreError::PortalLayoutInvalidTree(message)) = validate_tree(&tree) else {
            panic!("expected an invalid tree error");
        };
        assert_eq!(message, "tree[0].children[0].children must be an array");
    }
}
