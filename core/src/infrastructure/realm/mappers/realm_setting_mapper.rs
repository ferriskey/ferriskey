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
            login_aliases: value
                .login_aliases
                .iter()
                .filter_map(|s| s.parse::<ferriskey_domain::realm::LoginAlias>().ok())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap_or_default(),
            updated_at,
            lockout_threshold: value.lockout_threshold,
            lockout_duration_seconds: value.lockout_duration_seconds,
            seawatch_pii_mode: value.seawatch_pii_mode,
            seawatch_pseudo_key: value.seawatch_pseudo_key,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferriskey_domain::realm::{LoginAlias, LoginAliases};

    fn base_model() -> crate::entity::realm_settings::Model {
        crate::entity::realm_settings::Model {
            id: uuid::Uuid::now_v7(),
            realm_id: uuid::Uuid::now_v7(),
            default_signing_algorithm: None,
            updated_at: chrono::Utc::now().naive_utc(),
            user_registration_enabled: false,
            forgot_password_enabled: false,
            remember_me_enabled: false,
            magic_link_enabled: false,
            magic_link_ttl_minutes: 15,
            compass_enabled: false,
            access_token_lifetime_secs: 300,
            refresh_token_lifetime_secs: 3600,
            id_token_lifetime_secs: 300,
            temporary_token_lifetime_secs: 300,
            passkey_enabled: false,
            reset_password_template_id: None,
            magic_link_template_id: None,
            email_verification_template_id: None,
            email_verification_enabled: false,
            email_verification_ttl_hours: 24,
            portal_theme_id: None,
            lockout_threshold: 10,
            lockout_duration_seconds: 900,
            login_aliases: vec!["email".to_string(), "username".to_string()],
            seawatch_pii_mode: "off".to_string(),
            seawatch_pseudo_key: None,
        }
    }

    #[test]
    fn maps_login_aliases_in_order() {
        let setting = RealmSetting::from(base_model());
        assert_eq!(
            setting.login_aliases.as_slice(),
            &[LoginAlias::Email, LoginAlias::Username]
        );
    }

    #[test]
    fn maps_unknown_or_empty_aliases_to_default() {
        let mut model = base_model();
        model.login_aliases = vec![];
        let setting = RealmSetting::from(model);
        assert_eq!(setting.login_aliases, LoginAliases::default());

        let mut model = base_model();
        model.login_aliases = vec!["garbage".to_string()];
        let setting = RealmSetting::from(model);
        assert_eq!(setting.login_aliases, LoginAliases::default());
    }
}
