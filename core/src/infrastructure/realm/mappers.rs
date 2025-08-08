use crate::domain::realm::entities::Realm;
use chrono::{TimeZone, Utc};
use entity::realms::Model;

impl From<Model> for Realm {
    fn from(value: Model) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        Realm {
            id: value.id,
            name: value.name,
            created_at,
            updated_at,
        }
    }
}

impl From<&Model> for Realm {
    fn from(model: &Model) -> Self {
        let created_at = Utc.from_utc_datetime(&model.created_at);
        let updated_at = Utc.from_utc_datetime(&model.updated_at);

        Realm {
            id: model.id,
            name: model.name.clone(),
            created_at,
            updated_at,
        }
    }
}
