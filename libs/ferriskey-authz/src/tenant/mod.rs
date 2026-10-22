//! Authorization rules a realm writes for its own applications.
//!
//! Distinct from the rest of this crate, which guards FerrisKey's own admin
//! API: here FerrisKey is the policy decision point, and the rules belong to
//! the tenant.

pub mod entities;
pub mod evaluation;
pub mod evaluation_service;
pub mod evaluator;
pub mod names;
pub mod ports;
pub mod services;
pub mod value_objects;

mod policies;

#[cfg(test)]
mod fakes;

/// Reserved for the types FerrisKey resolves itself (`FerrisKey::User`, …).
pub const FERRISKEY_NAMESPACE: &str = "FerrisKey";

pub const AGENT_ACTIONS: [&str; 3] = ["call_tool", "read_resource", "get_prompt"];

pub const DESCRIBABLE_TYPES: [&str; 5] = [
    "FerrisKey::Agent",
    "FerrisKey::Tool",
    "FerrisKey::McpServer",
    "FerrisKey::Resource",
    "FerrisKey::Prompt",
];

pub const ASSERTED_SUBJECT_TYPES: [&str; 2] = ["FerrisKey::User", "FerrisKey::Agent"];

pub const ASSERTED_PARENT_TYPES: [&str; 3] = [
    "FerrisKey::Group",
    "FerrisKey::Role",
    "FerrisKey::Organization",
];

pub fn is_agent_action(name: &str) -> bool {
    AGENT_ACTIONS.contains(&name)
}
