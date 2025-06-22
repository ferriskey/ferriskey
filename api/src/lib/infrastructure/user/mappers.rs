use chrono::{TimeZone, Utc};

use crate::domain::user::entities::model::User;

impl From<entity::users::Model> for User {
    fn from(value: entity::users::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        User {
            id: value.id,
            realm_id: value.realm_id,
            username: value.username,
            firstname: value.firstname,
            lastname: value.lastname,
            email: value.email,
            email_verified: value.email_verified,
            enabled: value.enabled,
            client_id: value.client_id,
            roles: Vec::new(),
            realm: None,
            created_at,
            updated_at,
        }
    }
}
