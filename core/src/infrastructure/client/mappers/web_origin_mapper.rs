use std::str::FromStr;

use chrono::{TimeZone, Utc};

use crate::{
    domain::client::entities::web_origin::{WebOrigin, WebOriginValue},
    domain::common::entities::app_errors::CoreError,
    entity::client_web_origins::Model,
};

impl TryFrom<Model> for WebOrigin {
    type Error = CoreError;

    fn try_from(model: Model) -> Result<Self, Self::Error> {
        let value = WebOriginValue::from_str(&model.value)?;

        Ok(WebOrigin {
            id: model.id,
            client_id: model.client_id,
            value,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        })
    }
}
