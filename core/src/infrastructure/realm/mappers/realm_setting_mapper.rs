use chrono::{DateTime, TimeZone, Utc};

use crate::{domain::realm::entities::RealmSetting, entity::realm_settings::Model};

impl From<Model> for RealmSetting {
    fn from(value: crate::entity::realm_settings::Model) -> Self {
        let updated_at: DateTime<Utc> = Utc.from_utc_datetime(&value.updated_at);

        RealmSetting {
            id: value.id,
            realm_id: value.realm_id.into(),
            default_signing_algorithm: value.default_signing_algorithm,
            forgot_password_enabled: value.forgot_password_enabled,
            remember_me_enabled: value.remember_me_enabled,
            user_registration_enabled: value.user_registration_enabled,
            magic_link_enabled: Some(value.magic_link_enabled),
            magic_link_ttl_minutes: Some(value.magic_link_ttl_minutes as u32),
            updated_at,
        }
    }
}
