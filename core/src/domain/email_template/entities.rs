use std::collections::HashMap;
use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::domain::common::entities::app_errors::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EmailType {
    ResetPassword,
    MagicLink,
    EmailVerification,
}

impl Display for EmailType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailType::ResetPassword => write!(f, "reset_password"),
            EmailType::MagicLink => write!(f, "magic_link"),
            EmailType::EmailVerification => write!(f, "email_verification"),
        }
    }
}

impl TryFrom<String> for EmailType {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "reset_password" => Ok(EmailType::ResetPassword),
            "magic_link" => Ok(EmailType::MagicLink),
            "email_verification" => Ok(EmailType::EmailVerification),
            _ => Err(CoreError::InvalidEmailTemplateStructure(format!(
                "unknown email type: {value}"
            ))),
        }
    }
}

impl EmailType {
    pub fn available_variables(&self) -> Vec<TemplateVariable> {
        let mut vars = vec![
            TemplateVariable {
                name: "user.first_name".to_string(),
                description: "User's first name".to_string(),
            },
            TemplateVariable {
                name: "user.last_name".to_string(),
                description: "User's last name".to_string(),
            },
            TemplateVariable {
                name: "user.email".to_string(),
                description: "User's email address".to_string(),
            },
            TemplateVariable {
                name: "expiration".to_string(),
                description: "Expiration time".to_string(),
            },
        ];

        match self {
            EmailType::ResetPassword => {
                vars.push(TemplateVariable {
                    name: "reset_link".to_string(),
                    description: "Password reset link".to_string(),
                });
            }
            EmailType::MagicLink => {
                vars.push(TemplateVariable {
                    name: "magic_link".to_string(),
                    description: "Magic link URL".to_string(),
                });
            }
            EmailType::EmailVerification => {
                vars.push(TemplateVariable {
                    name: "verification_link".to_string(),
                    description: "Email verification link".to_string(),
                });
            }
        }

        vars
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct EmailTemplate {
    pub id: Uuid,
    pub realm_id: Uuid,
    pub name: String,
    pub email_type: EmailType,
    pub structure: serde_json::Value,
    pub mjml: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interpolates template variables into HTML content.
/// All variable values are HTML-escaped before substitution.
pub fn interpolate_variables(html: &str, variables: &HashMap<String, String>) -> String {
    let mut result = html.to_string();
    for (key, value) in variables {
        let escaped = html_escape(value);
        result = result.replace(&format!("{{{{{key}}}}}"), &escaped);
    }
    result
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_type_display() {
        assert_eq!(EmailType::ResetPassword.to_string(), "reset_password");
        assert_eq!(EmailType::MagicLink.to_string(), "magic_link");
        assert_eq!(
            EmailType::EmailVerification.to_string(),
            "email_verification"
        );
    }

    #[test]
    fn test_email_type_try_from() {
        assert_eq!(
            EmailType::try_from("reset_password".to_string()).unwrap(),
            EmailType::ResetPassword
        );
        assert!(EmailType::try_from("invalid".to_string()).is_err());
    }

    #[test]
    fn test_interpolate_variables() {
        let html = "<p>Hello {{user.first_name}}, click <a href=\"{{reset_link}}\">here</a></p>";
        let mut vars = HashMap::new();
        vars.insert("user.first_name".to_string(), "John".to_string());
        vars.insert(
            "reset_link".to_string(),
            "https://example.com/reset?token=abc".to_string(),
        );

        let result = interpolate_variables(html, &vars);
        assert_eq!(
            result,
            "<p>Hello John, click <a href=\"https://example.com/reset?token=abc\">here</a></p>"
        );
    }

    #[test]
    fn test_interpolate_variables_escapes_html() {
        let html = "<p>{{user.first_name}}</p>";
        let mut vars = HashMap::new();
        vars.insert(
            "user.first_name".to_string(),
            "<script>alert('xss')</script>".to_string(),
        );

        let result = interpolate_variables(html, &vars);
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }

    #[test]
    fn test_available_variables_per_type() {
        let vars = EmailType::ResetPassword.available_variables();
        assert!(vars.iter().any(|v| v.name == "reset_link"));
        assert!(vars.iter().any(|v| v.name == "user.first_name"));

        let vars = EmailType::MagicLink.available_variables();
        assert!(vars.iter().any(|v| v.name == "magic_link"));

        let vars = EmailType::EmailVerification.available_variables();
        assert!(vars.iter().any(|v| v.name == "verification_link"));
    }
}
