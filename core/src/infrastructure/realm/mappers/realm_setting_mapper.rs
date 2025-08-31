use chrono::{DateTime, TimeZone, Utc};

use crate::{domain::realm::entities::RealmSetting, entity::realm_settings::Model};

impl From<Model> for RealmSetting {
    fn from(value: crate::entity::realm_settings::Model) -> Self {
        let updated_at: DateTime<Utc> = Utc.from_utc_datetime(&value.updated_at);

        RealmSetting {
            id: value.id,
            realm_id: value.realm_id,
            default_signing_algorithm: value.default_signing_algorithm,
            updated_at,
        }
    }
}
