use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ResetPasswordValidator {
    #[serde(default)]
    pub temporary: bool,

    #[serde(default)]
    pub credential_type: String,

    #[validate(length(min = 1, message = "value is required"))]
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct ImportPasswordCredentialValidator {
    #[validate(length(min = 1, message = "algorithm is required"))]
    #[serde(default)]
    pub algorithm: String,

    #[validate(length(min = 1, message = "secret_data is required"))]
    #[serde(default)]
    pub secret_data: String,

    pub hash_iterations: u32,

    #[serde(default)]
    pub salt: Option<String>,

    #[serde(default)]
    pub temporary: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateUserValidator {
    #[serde(default)]
    pub id: Option<Uuid>,

    #[validate(length(min = 1, message = "username is required"))]
    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub firstname: Option<String>,

    #[serde(default)]
    pub lastname: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub email_verified: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct BulkDeleteUserValidator {
    #[serde(default)]
    pub ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateUserValidator {
    #[serde(default)]
    pub firstname: Option<String>,

    #[serde(default)]
    pub lastname: Option<String>,

    #[serde(default)]
    pub email: Option<String>,

    #[serde(default)]
    pub email_verified: Option<bool>,

    #[serde(default)]
    pub enabled: Option<bool>,

    #[serde(default)]
    pub required_actions: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::CreateUserValidator;

    #[test]
    fn create_user_defaults_the_id_to_none() {
        let payload: CreateUserValidator =
            serde_json::from_str(r#"{"username":"alice"}"#).expect("payload deserializes");

        assert_eq!(payload.id, None);
    }

    #[test]
    fn create_user_accepts_a_supplied_id() {
        let payload: CreateUserValidator = serde_json::from_str(
            r#"{"username":"alice","id":"0196a0d5-6ba6-7c1e-8f0a-3d4b5c6d7e8f"}"#,
        )
        .expect("payload deserializes");

        assert_eq!(
            payload.id.map(|id| id.to_string()),
            Some("0196a0d5-6ba6-7c1e-8f0a-3d4b5c6d7e8f".to_string())
        );
    }

    #[test]
    fn create_user_refuses_an_unknown_field() {
        let result: Result<CreateUserValidator, _> =
            serde_json::from_str(r#"{"username":"alice","enabled":true}"#);

        assert!(result.is_err());
    }
}
