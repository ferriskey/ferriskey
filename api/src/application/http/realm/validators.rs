use ferriskey_core::domain::realm::entities::LoginAliases;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRealmValidator {
    #[validate(
        length(min = 1, message = "name is required"),
        custom(function = "validate_realm_slug")
    )]
    #[serde(default)]
    pub name: String,
    #[validate(length(max = 255, message = "display_name must be at most 255 characters"))]
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRealmValidator {
    #[validate(
        length(min = 1, message = "name is required"),
        custom(function = "validate_realm_slug")
    )]
    pub name: String,
    #[validate(length(max = 255, message = "display_name must be at most 255 characters"))]
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRealmSettingValidator {
    pub default_signing_algorithm: Option<String>,

    pub user_registration_enabled: Option<bool>,
    pub forgot_password_enabled: Option<bool>,
    pub remember_me_enabled: Option<bool>,
    pub magic_link_enabled: Option<bool>,
    #[validate(range(min = 1, message = "magic_link_ttl must be greater than 0"))]
    pub magic_link_ttl: Option<u32>,
    pub passkey_enabled: Option<bool>,
    pub compass_enabled: Option<bool>,

    pub access_token_lifetime: Option<i64>,
    pub refresh_token_lifetime: Option<i64>,
    pub id_token_lifetime: Option<i64>,
    pub temporary_token_lifetime: Option<i64>,

    #[serde(default, deserialize_with = "deserialize_optional_field")]
    #[schema(value_type = Option<Uuid>)]
    pub reset_password_template_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    #[schema(value_type = Option<Uuid>)]
    pub magic_link_template_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    #[schema(value_type = Option<Uuid>)]
    pub email_verification_template_id: Option<Option<Uuid>>,
    pub email_verification_enabled: Option<bool>,
    #[validate(range(
        min = 1,
        max = 720,
        message = "email_verification_ttl_hours must be between 1 and 720"
    ))]
    pub email_verification_ttl_hours: Option<i64>,
    #[validate(range(
        min = 1,
        max = 100,
        message = "lockout_threshold must be between 1 and 100"
    ))]
    pub lockout_threshold: Option<i32>,
    #[validate(range(min = 0, message = "lockout_duration_seconds must be 0 or greater"))]
    pub lockout_duration_seconds: Option<i32>,
    #[schema(value_type = Option<Vec<String>>)]
    pub login_aliases: Option<LoginAliases>,
    pub seawatch_pii_mode: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    #[schema(value_type = Option<String>)]
    pub seawatch_pseudo_key: Option<Option<String>>,
    pub require_mfa: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpsertSmtpConfigValidator {
    #[validate(length(min = 1, message = "host is required"))]
    pub host: String,
    #[validate(range(min = 1, max = 65535, message = "port must be between 1 and 65535"))]
    pub port: u16,
    #[validate(length(min = 1, message = "username is required"))]
    pub username: String,
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
    #[validate(email(message = "from_email must be a valid email"))]
    pub from_email: String,
    #[validate(length(min = 1, message = "from_name is required"))]
    pub from_name: String,
    #[validate(custom(function = "validate_encryption"))]
    pub encryption: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdatePasswordPolicyValidator {
    #[validate(range(min = 1, max = 128, message = "min_length must be between 1 and 128"))]
    pub min_length: Option<i32>,
    pub require_uppercase: Option<bool>,
    pub require_lowercase: Option<bool>,
    pub require_number: Option<bool>,
    pub require_special: Option<bool>,
    #[validate(range(min = 0, message = "max_age_days must be 0 or greater"))]
    pub max_age_days: Option<i32>,
    #[validate(range(
        min = 0,
        max = 256,
        message = "min_entropy_bits must be between 0 and 256"
    ))]
    pub min_entropy_bits: Option<i32>,
    pub forbid_common: Option<bool>,
    pub check_breached: Option<bool>,
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

/// Ensures a realm `name` is safe to use as a URL slug.
///
/// The realm name is embedded in every realm URL (`/realms/{name}/...`), so it
/// must not contain whitespace or characters that would need URL-encoding.
/// Human-readable labels belong in `display_name`, which is free-form.
fn validate_realm_slug(value: &str) -> Result<(), validator::ValidationError> {
    if !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err(validator::ValidationError::new(
            "name must contain only letters, digits, hyphens or underscores",
        ))
    }
}

fn validate_encryption(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "tls" | "starttls" | "none" => Ok(()),
        _ => Err(validator::ValidationError::new(
            "encryption must be one of: tls, starttls, none",
        )),
    }
}

#[cfg(test)]
mod update_realm_setting_validator_tests {
    use super::UpdateRealmSettingValidator;
    use ferriskey_core::domain::realm::entities::LoginAlias;
    use serde_json::json;

    #[test]
    fn empty_login_aliases_is_rejected() {
        let v: Result<UpdateRealmSettingValidator, _> =
            serde_json::from_value(json!({ "login_aliases": [] }));
        assert!(
            v.is_err(),
            "expected deserialization to fail for empty login_aliases"
        );
    }

    #[test]
    fn unknown_login_alias_is_rejected() {
        let v: Result<UpdateRealmSettingValidator, _> =
            serde_json::from_value(json!({ "login_aliases": ["phone"] }));
        assert!(
            v.is_err(),
            "expected deserialization to fail for unknown alias 'phone'"
        );
    }

    #[test]
    fn valid_login_aliases_are_accepted() {
        let v: UpdateRealmSettingValidator =
            serde_json::from_value(json!({ "login_aliases": ["email", "username"] }))
                .expect("deserialization should succeed for [email, username]");
        let aliases = v.login_aliases.expect("login_aliases should be Some");
        assert_eq!(
            aliases.as_slice(),
            &[LoginAlias::Email, LoginAlias::Username]
        );
    }

    #[test]
    fn missing_login_aliases_is_none_and_backward_compatible() {
        let v: UpdateRealmSettingValidator =
            serde_json::from_value(json!({ "user_registration_enabled": true }))
                .expect("deserialization should succeed when login_aliases is absent");
        assert!(
            v.login_aliases.is_none(),
            "login_aliases should be None when omitted"
        );
        assert_eq!(v.user_registration_enabled, Some(true));
    }
}
