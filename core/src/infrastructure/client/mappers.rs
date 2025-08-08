use crate::domain::client::entities::Client;
use chrono::{TimeZone, Utc};

impl From<entity::clients::Model> for Client {
    fn from(model: entity::clients::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        Client {
            id: model.id,
            realm_id: model.realm_id,
            name: model.name,
            client_id: model.client_id,
            secret: model.secret,
            enabled: model.enabled,
            protocol: model.protocol,
            public_client: model.public_client,
            service_account_enabled: model.service_account_enabled,
            client_type: model.client_type,
            redirect_uris: None,
            created_at,
            updated_at,
        }
    }
}
