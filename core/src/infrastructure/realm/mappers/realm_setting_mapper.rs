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
            magic_link_enabled: value.magic_link_enabled,
            magic_link_ttl: value.magic_link_ttl_minutes.try_into().unwrap_or(15),
            passkey_enabled: value.passkey_enabled,
            compass_enabled: value.compass_enabled,
            access_token_lifetime: value.access_token_lifetime_secs as i64,
            refresh_token_lifetime: value.refresh_token_lifetime_secs as i64,
            id_token_lifetime: value.id_token_lifetime_secs as i64,
            temporary_token_lifetime: value.temporary_token_lifetime_secs as i64,
            reset_password_template_id: value.reset_password_template_id,
            magic_link_template_id: value.magic_link_template_id,
            email_verification_template_id: value.email_verification_template_id,
            email_verification_enabled: value.email_verification_enabled,
            email_verification_ttl_hours: value.email_verification_ttl_hours as i64,
            updated_at,
        }
    }
}
