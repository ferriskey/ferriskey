use ferriskey_core::domain::common::pagination::{Page, PageMetadata};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Paginated<T> {
    pub data: Vec<T>,
    pub metadata: PageMetadata,
}

impl<T> From<Page<T>> for Paginated<T> {
    fn from(page: Page<T>) -> Self {
        let (data, metadata) = page.into_parts();
        Self { data, metadata }
    }
}

#[cfg(test)]
mod tests {
    use ferriskey_core::domain::common::pagination::{Page, PageLimit, PageNumber};

    use super::*;

    #[test]
    fn serializes_data_and_metadata() {
        let page = Page::new(vec!["a"], 21, PageNumber::default(), PageLimit::default());
        let json = serde_json::to_value(Paginated::from(page)).expect("json");
        assert_eq!(json["data"], serde_json::json!(["a"]));
        assert_eq!(json["metadata"]["total"], 21);
        assert_eq!(json["metadata"]["total_pages"], 2);
        assert_eq!(json["metadata"]["next_page"], 2);
        assert_eq!(json["metadata"]["prev_page"], serde_json::Value::Null);
    }
}
