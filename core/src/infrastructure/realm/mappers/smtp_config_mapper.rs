use chrono::{DateTime, Utc};

use crate::{domain::realm::entities::SmtpConfig, entity::smtp_configs::Model};

impl From<Model> for SmtpConfig {
    fn from(value: Model) -> Self {
        let created_at: DateTime<Utc> = value.created_at.into();
        let updated_at: DateTime<Utc> = value.updated_at.into();

        SmtpConfig {
            id: value.id,
            realm_id: value.realm_id,
            host: value.host,
            port: value.port.try_into().unwrap_or(587),
            username: value.username,
            password: value.password,
            from_email: value.from_email,
            from_name: value.from_name,
            encryption: value.encryption.parse().unwrap(),
            created_at,
            updated_at,
        }
    }
}
