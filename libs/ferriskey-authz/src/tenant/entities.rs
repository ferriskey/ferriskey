use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::names::{Namespace, PolicyName, RelationName, SlotId};
use crate::entities::{EntityRef, PolicyOrigin};

/// Whether a realm answers authorization questions, and how many times its
/// rules have changed.
///
/// `version` moves on every write to the schema, a policy or a template. A
/// cache keyed on it never serves rules older than the version it checked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AuthzState {
    pub enabled: bool,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthzSchema {
    pub id: Uuid,
    pub namespace: Namespace,
    /// Cedar schema text. Opaque to the domain; only a `PolicyCompiler` reads it.
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyTemplate {
    pub id: Uuid,
    pub name: PolicyName,
    pub description: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolicyBody {
    Static {
        source: String,
    },
    Linked {
        template_id: Uuid,
        links: BTreeMap<SlotId, EntityRef>,
    },
}

impl PolicyBody {
    pub fn template_id(&self) -> Option<Uuid> {
        match self {
            Self::Static { .. } => None,
            Self::Linked { template_id, .. } => Some(*template_id),
        }
    }

    /// A policy written by hand is `Custom`; one instantiated from a template
    /// is `Generated`, so the UI can tell which ones it may rewrite.
    pub fn origin(&self) -> PolicyOrigin {
        match self {
            Self::Static { .. } => PolicyOrigin::Custom,
            Self::Linked { .. } => PolicyOrigin::Generated,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub id: Uuid,
    pub name: PolicyName,
    pub description: Option<String>,
    pub origin: PolicyOrigin,
    pub enabled: bool,
    pub body: PolicyBody,
}

/// A stored edge `object#relation@subject`, or `object#relation@subject#subject_relation`
/// when the subject is a set (every member of a team).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RelationTuple {
    pub object: EntityRef,
    pub relation: RelationName,
    pub subject: EntityRef,
    pub subject_relation: Option<RelationName>,
}
