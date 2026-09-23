use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::realm::RealmId;
use crate::realm::scope::RealmOwned;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElevationId(Uuid);

impl ElevationId {
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ElevationId> for Uuid {
    fn from(value: ElevationId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elevation {
    pub id: ElevationId,
    pub user_id: Uuid,
    pub realm_id: RealmId,
    pub expires_at: DateTime<Utc>,
}

impl RealmOwned for Elevation {
    fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}

#[must_use = "an elevation proof is what authorises a sensitive operation; dropping it discards the re-authentication the user just performed"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elevated {
    user_id: Uuid,
    realm_id: RealmId,
}

impl Elevated {
    pub(super) fn over(user_id: Uuid, realm_id: RealmId) -> Self {
        Self { user_id, realm_id }
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn realm_id(&self) -> RealmId {
        self.realm_id
    }
}
