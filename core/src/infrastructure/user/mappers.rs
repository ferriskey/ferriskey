use chrono::{TimeZone, Utc};

use crate::domain::user::entities::{RequiredAction, RequiredActionError, User, UserAttribute};

impl From<crate::entity::users::Model> for User {
    fn from(value: crate::entity::users::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        User {
            id: value.id,
            realm_id: value.realm_id.into(),
            username: value.username,
            firstname: value.firstname,
            lastname: value.lastname,
            email: value.email,
            email_verified: value.email_verified,
            enabled: value.enabled,
            client_id: value.client_id,
            roles: None,
            realm: None,
            required_actions: Vec::new(),
            created_at,
            updated_at,
        }
    }
}

impl TryFrom<crate::entity::user_required_actions::Model> for RequiredAction {
    type Error = RequiredActionError;
    fn try_from(value: crate::entity::user_required_actions::Model) -> Result<Self, Self::Error> {
        RequiredAction::try_from(value.action)
    }
}

impl From<crate::entity::user_attributes::Model> for UserAttribute {
    fn from(value: crate::entity::user_attributes::Model) -> Self {
        let created_at = value.created_at.with_timezone(&Utc);
        let updated_at = value.updated_at.with_timezone(&Utc);

        UserAttribute {
            id: value.id,
            user_id: value.user_id,
            realm_id: value.realm_id.into(),
            key: value.key,
            value: value.value,
            created_at,
            updated_at,
        }
    }
}
