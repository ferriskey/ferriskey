use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::portal_theme::entities::PortalPageType;

/// Block types that must appear somewhere in the JSONB tree for each portal
/// page to be considered renderable. Validation walks the tree collecting
/// every node's `type` field, then asserts set inclusion.
pub const REQUIRED_BLOCKS: &[(PortalPageType, &[&str])] = &[
    (
        PortalPageType::Login,
        &["email_input", "password_input", "submit_button"],
    ),
    (
        PortalPageType::Register,
        &["email_input", "password_input", "submit_button"],
    ),
    (PortalPageType::Totp, &["totp_input", "submit_button"]),
    (
        PortalPageType::ForgotPassword,
        &["email_input", "submit_button"],
    ),
    (
        PortalPageType::ResetPassword,
        &["password_input", "submit_button"],
    ),
    (PortalPageType::MagicLinkVerify, &["submit_button"]),
    (PortalPageType::VerifyEmail, &["submit_button"]),
];

pub fn required_blocks_for(page_type: PortalPageType) -> &'static [&'static str] {
    REQUIRED_BLOCKS
        .iter()
        .find_map(|(pt, blocks)| {
            if *pt == page_type {
                Some(*blocks)
            } else {
                None
            }
        })
        .unwrap_or(&[])
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MissingBlocks {
    pub page_type: PortalPageType,
    pub missing: Vec<String>,
}

/// Validate that every required block type for `page_type` appears somewhere
/// in `tree` (nested at any depth). Returns the missing block types, if any.
pub fn validate_tree(
    page_type: PortalPageType,
    tree: &serde_json::Value,
) -> Result<(), MissingBlocks> {
    let mut present = std::collections::HashSet::new();
    collect_types(tree, &mut present);

    let required = required_blocks_for(page_type);
    let missing: Vec<String> = required
        .iter()
        .filter(|required_type| !present.contains(**required_type))
        .map(|s| (*s).to_string())
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(MissingBlocks { page_type, missing })
    }
}

/// Validate every page in the supplied collection. Iterates `PortalPageType::ALL`
/// and aggregates each failure, so a single call surfaces every invalid page
/// at once (vs. having the caller validate page-by-page and stop at the first
/// error). Used when activating a theme — partial validation would hide pages
/// that also block activation.
pub fn validate_pages(
    pages: impl Fn(PortalPageType) -> serde_json::Value,
) -> Result<(), Vec<MissingBlocks>> {
    let failures: Vec<MissingBlocks> = PortalPageType::ALL
        .iter()
        .copied()
        .filter_map(|pt| validate_tree(pt, &pages(pt)).err())
        .collect();

    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

fn collect_types(value: &serde_json::Value, acc: &mut std::collections::HashSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(t)) = map.get("type") {
                acc.insert(t.clone());
            }
            for (_, v) in map {
                collect_types(v, acc);
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_types(item, acc);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn login_with_all_required_blocks_passes() {
        let tree = json!([
            { "type": "container", "children": [
                { "type": "email_input" },
                { "type": "password_input" },
                { "type": "submit_button" }
            ]}
        ]);
        assert!(validate_tree(PortalPageType::Login, &tree).is_ok());
    }

    #[test]
    fn login_missing_password_input_fails() {
        let tree = json!([
            { "type": "email_input" },
            { "type": "submit_button" }
        ]);
        let err = validate_tree(PortalPageType::Login, &tree).unwrap_err();
        assert_eq!(err.page_type, PortalPageType::Login);
        assert_eq!(err.missing, vec!["password_input".to_string()]);
    }

    #[test]
    fn deeply_nested_required_block_is_found() {
        let tree = json!({
            "type": "container",
            "children": [
                { "type": "container", "children": [
                    { "type": "container", "children": [
                        { "type": "submit_button" }
                    ]}
                ]}
            ]
        });
        assert!(validate_tree(PortalPageType::VerifyEmail, &tree).is_ok());
    }

    #[test]
    fn empty_tree_reports_all_required_missing_for_login() {
        let tree = json!([]);
        let err = validate_tree(PortalPageType::Login, &tree).unwrap_err();
        assert_eq!(
            err.missing,
            vec![
                "email_input".to_string(),
                "password_input".to_string(),
                "submit_button".to_string()
            ]
        );
    }

    #[test]
    fn every_page_type_has_an_entry_in_required_blocks() {
        for pt in PortalPageType::ALL {
            assert!(
                !required_blocks_for(pt).is_empty(),
                "missing REQUIRED_BLOCKS entry for {pt:?}"
            );
        }
    }
}
