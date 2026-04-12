use chrono::{TimeZone, Utc};

use crate::{
    domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy},
    entity::clients::Model,
};

impl From<Model> for Client {
    fn from(model: crate::entity::clients::Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        Client {
            id: model.id,
            realm_id: model.realm_id.into(),
            name: model.name,
            client_id: model.client_id,
            secret: model.secret,
            enabled: model.enabled,
            protocol: model.protocol,
            public_client: model.public_client,
            service_account_enabled: model.service_account_enabled,
            direct_access_grants_enabled: model.direct_access_grants_enabled.unwrap_or(false),
            client_type: model
                .client_type
                .parse::<ClientType>()
                .unwrap_or(ClientType::Confidential),
            redirect_uris: None,
            access_token_lifetime: model.access_token_lifetime_secs.map(|v| v as i64),
            refresh_token_lifetime: model.refresh_token_lifetime_secs.map(|v| v as i64),
            id_token_lifetime: model.id_token_lifetime_secs.map(|v| v as i64),
            temporary_token_lifetime: model.temporary_token_lifetime_secs.map(|v| v as i64),
            maintenance_enabled: model.maintenance_enabled.unwrap_or(false),
            maintenance_reason: model.maintenance_reason,
            maintenance_session_strategy: model
                .maintenance_session_strategy
                .and_then(|s| s.parse::<MaintenanceSessionStrategy>().ok())
                .unwrap_or_default(),
            created_at,
            updated_at,
        }
    }
}
