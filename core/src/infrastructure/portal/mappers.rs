use chrono::{TimeZone, Utc};

use crate::{
    domain::portal::entities::PortalConfig,
    entity::realm_portal_configs::Model as PortalConfigModel,
};

impl From<PortalConfigModel> for PortalConfig {
    fn from(value: PortalConfigModel) -> Self {
        Self {
            id: value.id,
            realm_id: value.realm_id,
            is_active: value.is_active,
            layout: value.layout,
            created_at: Utc.from_utc_datetime(&value.created_at),
            updated_at: Utc.from_utc_datetime(&value.updated_at),
        }
    }
}
