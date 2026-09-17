use chrono::{DateTime, TimeZone, Utc};

use crate::{
    domain::common::locale::{Locale, SupportedLocales},
    domain::realm::entities::RealmSetting,
    entity::realm_settings::Model,
};

fn english_only() -> SupportedLocales {
    SupportedLocales::new(Locale::en(), vec![Locale::en()])
        .expect("english alone is a non-empty set holding its own default")
}

fn stored_locales(default_locale: &str, supported_locales: &[String]) -> SupportedLocales {
    let Ok(default) = Locale::parse(default_locale) else {
        return english_only();
    };

    let mut all: Vec<Locale> = supported_locales
        .iter()
        .filter_map(|raw| Locale::parse(raw).ok())
        .collect();

    if !all.contains(&default) {
        all.push(default.clone());
    }

    SupportedLocales::new(default, all).unwrap_or_else(|_| english_only())
}

impl From<Model> for RealmSetting {
    fn from(value: crate::entity::realm_settings::Model) -> Self {
        let updated_at: DateTime<Utc> = Utc.from_utc_datetime(&value.updated_at);
        let locales = stored_locales(&value.default_locale, &value.supported_locales);

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
            default_locale: locales.default_locale().to_string(),
            supported_locales: locales.all().iter().map(Locale::to_string).collect(),
            require_mfa: value.require_mfa,
            edit_username_enabled: value.edit_username_enabled,
            updated_at,
            lockout_threshold: value.lockout_threshold,
            lockout_duration_seconds: value.lockout_duration_seconds,
            seawatch_pii_mode: value.seawatch_pii_mode,
            seawatch_pseudo_key: value.seawatch_pseudo_key,
            webhook_retry_max_attempts: value.webhook_retry_max_attempts,
            webhook_retry_base_delay_ms: value.webhook_retry_base_delay_ms,
            webhook_retry_max_delay_ms: value.webhook_retry_max_delay_ms,
            webhook_retry_max_total_delay_ms: value.webhook_retry_max_total_delay_ms,
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
            require_mfa: false,
            edit_username_enabled: false,
            webhook_retry_max_attempts: None,
            webhook_retry_base_delay_ms: None,
            webhook_retry_max_delay_ms: None,
            webhook_retry_max_total_delay_ms: None,
            default_locale: "en".to_string(),
            supported_locales: vec!["en".to_string()],
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

    #[test]
    fn maps_the_stored_locales_in_their_declared_order() {
        let mut model = base_model();
        model.default_locale = "zh-CN".to_string();
        model.supported_locales = vec!["zh-CN".to_string(), "en".to_string()];

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "zh-CN");
        assert_eq!(setting.supported_locales, vec!["zh-CN", "en"]);
    }

    #[test]
    fn normalizes_the_case_of_stored_locales() {
        let mut model = base_model();
        model.default_locale = "ZH-cn".to_string();
        model.supported_locales = vec!["ZH-cn".to_string(), "EN".to_string()];

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "zh-CN");
        assert_eq!(setting.supported_locales, vec!["zh-CN", "en"]);
    }

    #[test]
    fn drops_malformed_stored_locales_and_keeps_the_default() {
        let mut model = base_model();
        model.default_locale = "en".to_string();
        model.supported_locales = vec!["en".to_string(), "garbage".to_string()];

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "en");
        assert_eq!(setting.supported_locales, vec!["en"]);
    }

    #[test]
    fn a_default_absent_from_the_stored_set_is_added_back() {
        let mut model = base_model();
        model.default_locale = "en".to_string();
        model.supported_locales = vec!["zh-CN".to_string()];

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "en");
        assert_eq!(setting.supported_locales, vec!["zh-CN", "en"]);
    }

    #[test]
    fn an_empty_or_malformed_stored_default_falls_back_to_english() {
        let mut model = base_model();
        model.default_locale = "garbage".to_string();
        model.supported_locales = vec!["zh-CN".to_string()];

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "en");
        assert_eq!(setting.supported_locales, vec!["en"]);

        let mut model = base_model();
        model.supported_locales = Vec::new();

        let setting = RealmSetting::from(model);

        assert_eq!(setting.default_locale, "en");
        assert_eq!(setting.supported_locales, vec!["en"]);
    }
}
