use chrono::{TimeZone, Utc};

use crate::domain::aegis::entities::{
    ClientScope, ClientScopeAttribute, ClientScopeMapping, ProtocolMapper,
};
use crate::entity::{
    client_scope_attributes, client_scope_mappings, client_scope_protocol_mappers, client_scopes,
};

impl From<client_scopes::Model> for ClientScope {
    fn from(model: client_scopes::Model) -> Self {
        ClientScope {
            id: model.id,
            realm_id: model.realm_id.into(),
            name: model.name,
            description: model.description,
            protocol: model.protocol,
            attributes: None,
            protocol_mappers: None,
            default_scope_type: model.default_scope_type.into(),
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        }
    }
}

impl From<client_scope_attributes::Model> for ClientScopeAttribute {
    fn from(model: client_scope_attributes::Model) -> Self {
        ClientScopeAttribute {
            id: model.id,
            scope_id: model.client_scope_id,
            name: model.name,
            value: model.value,
        }
    }
}

impl From<client_scope_protocol_mappers::Model> for ProtocolMapper {
    fn from(model: client_scope_protocol_mappers::Model) -> Self {
        ProtocolMapper {
            id: model.id,
            client_scope_id: model.client_scope_id,
            name: model.name,
            mapper_type: model.mapper_type,
            config: model.config,
            created_at: Utc.from_utc_datetime(&model.created_at),
        }
    }
}

impl From<client_scope_mappings::Model> for ClientScopeMapping {
    fn from(model: client_scope_mappings::Model) -> Self {
        ClientScopeMapping {
            client_id: model.client_id,
            scope_id: model.client_scope_id,
            default_scope_type: model.default_scope_type.into(),
        }
    }
}
