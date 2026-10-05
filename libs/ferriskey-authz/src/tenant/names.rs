use std::fmt;

use ferriskey_domain::common::app_errors::{CoreError, PolicyIssue};
use serde::{Deserialize, Serialize};

use super::FERRISKEY_NAMESPACE;

const MAX_IDENTIFIER_LEN: usize = 64;
const MAX_POLICY_NAME_LEN: usize = 128;

fn is_cedar_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn invalid(what: &str, value: &str) -> CoreError {
    CoreError::InvalidAuthorizationPolicy(vec![PolicyIssue {
        policy: None,
        message: format!("invalid {what}: {value:?}"),
        line: None,
        column: None,
    }])
}

/// The namespace a realm declares its own entity types and actions in.
///
/// A Cedar identifier, and never `FerrisKey`: that namespace carries the
/// users, roles, groups and organizations FerrisKey resolves itself, and a
/// tenant declaring it could shadow them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Namespace(String);

impl Namespace {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Namespace {
    fn default() -> Self {
        Self("App".to_string())
    }
}

impl TryFrom<String> for Namespace {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > MAX_IDENTIFIER_LEN
            || !is_cedar_identifier(&value)
            || value == FERRISKEY_NAMESPACE
        {
            return Err(invalid("namespace", &value));
        }
        Ok(Self(value))
    }
}

impl From<Namespace> for String {
    fn from(value: Namespace) -> Self {
        value.0
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The name of a policy or a template, unique within its realm.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PolicyName(String);

impl PolicyName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PolicyName {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let valid = !value.is_empty()
            && value.len() <= MAX_POLICY_NAME_LEN
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'));
        if !valid {
            return Err(invalid("policy name", &value));
        }
        Ok(Self(value))
    }
}

impl From<PolicyName> for String {
    fn from(value: PolicyName) -> Self {
        value.0
    }
}

impl fmt::Display for PolicyName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The relation half of a tuple, e.g. `viewer` in `doc:42#viewer@user:7`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelationName(String);

impl RelationName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RelationName {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > MAX_IDENTIFIER_LEN || !is_cedar_identifier(&value) {
            return Err(invalid("relation", &value));
        }
        Ok(Self(value))
    }
}

impl From<RelationName> for String {
    fn from(value: RelationName) -> Self {
        value.0
    }
}

/// A hole in a policy template. Cedar only has these two.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlotId {
    #[serde(rename = "?principal")]
    Principal,
    #[serde(rename = "?resource")]
    Resource,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_namespace_is_a_cedar_identifier() {
        assert!(Namespace::try_from("Billing_v2".to_string()).is_ok());
        assert!(Namespace::try_from("2fast".to_string()).is_err());
        assert!(Namespace::try_from("App::Sub".to_string()).is_err());
        assert!(Namespace::try_from(String::new()).is_err());
        assert!(Namespace::try_from("a".repeat(65)).is_err());
    }

    #[test]
    fn the_ferriskey_namespace_is_reserved() {
        assert!(matches!(
            Namespace::try_from("FerrisKey".to_string()),
            Err(CoreError::InvalidAuthorizationPolicy(_))
        ));
    }

    #[test]
    fn a_namespace_rejected_on_input_is_rejected_on_deserialization() {
        assert!(serde_json::from_str::<Namespace>("\"FerrisKey\"").is_err());
        assert_eq!(
            serde_json::from_str::<Namespace>("\"App\"").ok(),
            Some(Namespace::default())
        );
    }

    #[test]
    fn a_policy_name_is_short_and_printable() {
        assert!(PolicyName::try_from("editors-can-update.v2".to_string()).is_ok());
        assert!(PolicyName::try_from(String::new()).is_err());
        assert!(PolicyName::try_from("has space".to_string()).is_err());
        assert!(PolicyName::try_from("x".repeat(129)).is_err());
    }

    #[test]
    fn a_relation_is_a_cedar_identifier() {
        assert!(RelationName::try_from("viewer".to_string()).is_ok());
        assert!(RelationName::try_from("can-view".to_string()).is_err());
    }
}
