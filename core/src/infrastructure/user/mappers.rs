use chrono::{TimeZone, Utc};

use crate::domain::user::entities::{RequiredAction, RequiredActionError, User};

impl From<crate::entity::users::Model> for User {
    fn from(value: crate::entity::users::Model) -> Self {
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
