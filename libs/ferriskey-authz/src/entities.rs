//! The engine-agnostic authorization vocabulary.
//!
//! Shaped after the OpenID AuthZEN Authorization API 1.0 request model so the
//! HTTP facade, the legacy RBAC engine and any future engine speak the same
//! language. Nothing here knows about a policy language.

use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The type half of an entity reference, e.g. `User` in `User::"<uuid>"`.
///
/// Open by construction: [`EntityType::custom`] leaves room for tenant-declared
/// resource types without reopening this enum-shaped newtype.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EntityType(Cow<'static, str>);

impl EntityType {
    pub const REALM: Self = Self(Cow::Borrowed("Realm"));
    pub const USER: Self = Self(Cow::Borrowed("User"));
    pub const CLIENT: Self = Self(Cow::Borrowed("Client"));
    pub const ROLE: Self = Self(Cow::Borrowed("Role"));
    pub const ORGANIZATION: Self = Self(Cow::Borrowed("Organization"));
    pub const GROUP: Self = Self(Cow::Borrowed("Group"));

    pub fn custom(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A typed reference to something the engine can reason about.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityRef {
    pub ty: EntityType,
    pub id: String,
}

impl EntityRef {
    pub fn new(ty: EntityType, id: impl Into<String>) -> Self {
        Self { ty, id: id.into() }
    }

    pub fn realm(id: impl Into<Uuid>) -> Self {
        Self::new(EntityType::REALM, id.into().to_string())
    }

    pub fn user(id: Uuid) -> Self {
        Self::new(EntityType::USER, id.to_string())
    }

    pub fn client(id: Uuid) -> Self {
        Self::new(EntityType::CLIENT, id.to_string())
    }

    pub fn role(id: Uuid) -> Self {
        Self::new(EntityType::ROLE, id.to_string())
    }

    pub fn organization(id: Uuid) -> Self {
        Self::new(EntityType::ORGANIZATION, id.to_string())
    }

    pub fn group(id: Uuid) -> Self {
        Self::new(EntityType::GROUP, id.to_string())
    }
}

impl std::fmt::Display for EntityRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::\"{}\"", self.ty, self.id)
    }
}

/// An action name in `resource_type:verb` form, e.g. `user:create`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ActionId(Cow<'static, str>);

impl ActionId {
    pub const fn new(name: &'static str) -> Self {
        Self(Cow::Borrowed(name))
    }

    pub fn parsed(name: impl Into<Cow<'static, str>>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// An attribute value carried by an entity or a request context.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttrValue {
    Bool(bool),
    Long(i64),
    String(String),
    Set(Vec<AttrValue>),
}

impl From<bool> for AttrValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<i64> for AttrValue {
    fn from(v: i64) -> Self {
        Self::Long(v)
    }
}

impl From<String> for AttrValue {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for AttrValue {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<Uuid> for AttrValue {
    fn from(v: Uuid) -> Self {
        Self::String(v.to_string())
    }
}

/// What the caller knows about the resource, so the engine never has to query
/// for it.
///
/// Callers already loaded the resource before checking permission on it, so
/// they pass what they have. `parents` carries the containment chain the ReBAC
/// side walks: typically the realm, plus the organization when the resource is
/// org-scoped.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceSlice {
    pub entity: EntityRef,
    pub parents: Vec<EntityRef>,
    pub attributes: BTreeMap<String, AttrValue>,
}

impl ResourceSlice {
    pub fn new(entity: EntityRef) -> Self {
        Self {
            entity,
            parents: Vec::new(),
            attributes: BTreeMap::new(),
        }
    }

    /// The realm itself as a resource — what `can_create_*` checks target,
    /// since there is no instance to point at yet.
    pub fn realm(realm_id: impl Into<Uuid>) -> Self {
        Self::new(EntityRef::realm(realm_id))
    }

    pub fn parent(mut self, parent: EntityRef) -> Self {
        self.parents.push(parent);
        self
    }

    pub fn attr(mut self, key: impl Into<String>, value: impl Into<AttrValue>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Ambient facts about the request, independent of subject and resource.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RequestContext {
    /// The realm the request targets — the one named in the URL path, not the
    /// caller's own realm.
    pub realm_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub cross_realm: bool,
    pub mfa: bool,
    pub extra: BTreeMap<String, AttrValue>,
}

impl RequestContext {
    pub fn for_realm(realm_id: impl Into<Uuid>) -> Self {
        Self {
            realm_id: Some(realm_id.into()),
            ..Default::default()
        }
    }

    pub fn attr(mut self, key: impl Into<String>, value: impl Into<AttrValue>) -> Self {
        self.extra.insert(key.into(), value.into());
        self
    }
}

/// One authorization question.
#[derive(Clone, Debug, PartialEq)]
pub struct AccessRequest {
    pub subject: EntityRef,
    pub action: ActionId,
    pub resource: ResourceSlice,
    pub context: RequestContext,
}

impl AccessRequest {
    pub fn new(subject: EntityRef, action: ActionId, resource: ResourceSlice) -> Self {
        Self {
            subject,
            action,
            resource,
            context: RequestContext::default(),
        }
    }

    pub fn context(mut self, context: RequestContext) -> Self {
        self.context = context;
        self
    }
}

/// One authorization answer.
///
/// `determining_policies` is diagnostic only — it feeds `reason_admin` in the
/// AuthZen response and the divergence logs during engine migration. It must
/// never drive control flow.
#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    pub allowed: bool,
    pub determining_policies: Vec<String>,
}

impl Decision {
    pub fn allow() -> Self {
        Self {
            allowed: true,
            determining_policies: Vec::new(),
        }
    }

    pub fn deny() -> Self {
        Self {
            allowed: false,
            determining_policies: Vec::new(),
        }
    }

    pub fn with_policies(mut self, policies: Vec<String>) -> Self {
        self.determining_policies = policies;
        self
    }
}

impl From<bool> for Decision {
    fn from(allowed: bool) -> Self {
        if allowed { Self::allow() } else { Self::deny() }
    }
}

/// A node of the group tree a principal belongs to.
///
/// Kept as direct edges rather than a flattened ancestor set: an engine with a
/// native hierarchy computes the transitive closure itself, and flattening here
/// would throw away the shape it needs.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupNode {
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub organization_id: Uuid,
}

/// Everything about a principal an engine needs, resolved once per request.
///
/// Loaded by the auth middleware, which already holds the `User` and `Client`,
/// so a decision costs no further database round-trip.
#[derive(Clone, Debug, PartialEq)]
pub struct PrincipalSlice {
    pub principal: EntityRef,
    pub realm_id: Uuid,
    pub realm_name: String,
    pub is_master_realm: bool,
    pub enabled: bool,
    pub is_service_account: bool,
    pub role_ids: Vec<Uuid>,
    pub organization_ids: Vec<Uuid>,
    pub groups: Vec<GroupNode>,
    pub attributes: BTreeMap<String, String>,
}

/// Where a policy came from, and therefore whether an admin may edit it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyOrigin {
    /// Shipped with FerrisKey.
    Builtin,
    /// Derived from the roles table; rewritten on every role change.
    Generated,
    /// Written by a realm administrator.
    Custom,
}

/// One policy document, in whatever syntax the engine reads.
#[derive(Clone, Debug, PartialEq)]
pub struct PolicySource {
    pub id: String,
    pub origin: PolicyOrigin,
    pub source: String,
}

/// The full policy set of a realm, at a given version.
#[derive(Clone, Debug, PartialEq)]
pub struct PolicyBundle {
    pub realm_id: Uuid,
    pub version: i64,
    pub policies: Vec<PolicySource>,
}
