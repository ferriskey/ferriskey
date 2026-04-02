use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateRealmValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateRealmValidator {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,
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
}

fn deserialize_optional_field<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::deserialize(deserializer)?))
}

fn validate_encryption(value: &str) -> Result<(), validator::ValidationError> {
    match value {
        "tls" | "starttls" | "none" => Ok(()),
        _ => Err(validator::ValidationError::new(
            "encryption must be one of: tls, starttls, none",
        )),
    }
}
