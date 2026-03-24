use chrono::{TimeZone, Utc};

use crate::{
    domain::email_template::entities::{EmailTemplate, EmailType},
    entity::email_templates::Model as EmailTemplateModel,
};

impl From<EmailTemplateModel> for EmailTemplate {
    fn from(value: EmailTemplateModel) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        let email_type =
            EmailType::try_from(value.email_type.clone()).unwrap_or(EmailType::ResetPassword);

        Self {
            id: value.id,
            realm_id: value.realm_id,
            name: value.name,
            email_type,
            structure: value.structure,
            mjml: value.mjml,
            is_active: value.is_active,
            created_at,
            updated_at,
        }
    }
}

impl From<&EmailTemplateModel> for EmailTemplate {
    fn from(value: &EmailTemplateModel) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        let email_type =
            EmailType::try_from(value.email_type.clone()).unwrap_or(EmailType::ResetPassword);

        Self {
            id: value.id,
            realm_id: value.realm_id,
            name: value.name.clone(),
            email_type,
            structure: value.structure.clone(),
            mjml: value.mjml.clone(),
            is_active: value.is_active,
            created_at,
            updated_at,
        }
    }
}
