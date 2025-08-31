use chrono::{TimeZone, Utc};

use crate::{domain::client::entities::redirect_uri::RedirectUri, entity::redirect_uris::Model};

impl From<Model> for RedirectUri {
    fn from(model: crate::entity::redirect_uris::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        RedirectUri {
            id: model.id,
            client_id: model.client_id,
            value: model.value,
            enabled: model.enabled,
            created_at,
            updated_at,
        }
    }
}
