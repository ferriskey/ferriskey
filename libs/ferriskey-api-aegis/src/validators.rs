use ferriskey_core::domain::authentication::mapper_engine::claim_name_is_acceptable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::{Validate, ValidationError};

const CLAIM_NAME_KEYS: [&str; 2] = ["claim.name", "token.claim.name"];

fn validate_mapper_config(config: &serde_json::Value) -> Result<(), ValidationError> {
    for key in CLAIM_NAME_KEYS {
        let Some(value) = config.get(key) else {
            continue;
        };
        let acceptable = value.as_str().is_some_and(claim_name_is_acceptable);
        if !acceptable {
            let mut error = ValidationError::new("invalid_claim_name");
            error.message = Some(
                format!(
                    "{key} must be a dotted path of at most 8 non-empty segments and 255 characters"
                )
                .into(),
            );
            return Err(error);
        }
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateClientScopeValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[validate(length(min = 1, message = "protocol is required"))]
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub dynamic_registration_allowed: bool,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateClientScopeValidator {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub dynamic_registration_allowed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct CreateProtocolMapperValidator {
    #[validate(length(min = 1, message = "name is required"))]
    #[serde(default)]
    pub name: String,
    #[validate(length(min = 1, message = "mapper_type is required"))]
    #[serde(default)]
    pub mapper_type: String,
    #[validate(custom(function = "validate_mapper_config"))]
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
}

fn default_config() -> serde_json::Value {
    serde_json::Value::Object(serde_json::Map::new())
}

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UpdateProtocolMapperValidator {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub mapper_type: Option<String>,
    #[validate(custom(function = "validate_mapper_config"))]
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn create(config: serde_json::Value) -> CreateProtocolMapperValidator {
        CreateProtocolMapperValidator {
            name: "mapper".to_string(),
            mapper_type: "oidc-hardcoded-claim-mapper".to_string(),
            config,
        }
    }

    #[test]
    fn a_mapper_with_an_ordinary_claim_name_is_accepted() {
        assert!(
            create(json!({"claim.name": "realm_access.roles"}))
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn a_mapper_without_a_claim_name_is_accepted() {
        assert!(create(json!({})).validate().is_ok());
    }

    #[test]
    fn a_mapper_whose_claim_name_nests_too_deep_is_refused() {
        let deep = ["a"; 10_000].join(".");

        assert!(create(json!({"claim.name": deep})).validate().is_err());
        assert!(
            create(json!({"token.claim.name": deep}))
                .validate()
                .is_err()
        );
    }

    #[test]
    fn a_mapper_whose_claim_name_is_not_a_string_is_refused() {
        assert!(create(json!({"claim.name": {"a": 1}})).validate().is_err());
    }

    #[test]
    fn an_update_is_checked_the_same_way() {
        let update = |config| UpdateProtocolMapperValidator {
            name: None,
            mapper_type: None,
            config: Some(config),
        };

        let deep = ["a"; 10_000].join(".");

        assert!(update(json!({"claim.name": deep})).validate().is_err());
        assert!(update(json!({"claim.name": "email"})).validate().is_ok());
    }
}
