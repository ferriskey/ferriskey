use chrono::{DateTime, Utc};
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
}
