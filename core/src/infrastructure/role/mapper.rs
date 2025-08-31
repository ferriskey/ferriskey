use chrono::{TimeZone, Utc};

use crate::{
    domain::role::entities::{Role, permission::Permissions},
    entity::roles::Model,
};

impl From<Model> for Role {
    fn from(model: crate::entity::roles::Model) -> Self {
        let permissions = Permissions::from_bitfield(model.permissions as u64);
        let permissions = permissions
            .iter()
            .map(|p| p.name().to_string())
            .collect::<Vec<String>>();

        Role {
            id: model.id,
            name: model.name,
            description: model.description,
            permissions,
            realm_id: model.realm_id,
            client_id: model.client_id,
            client: None,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        }
    }
}
