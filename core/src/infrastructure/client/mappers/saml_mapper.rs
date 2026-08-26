use std::str::FromStr;

use chrono::{TimeZone, Utc};

use crate::{
    domain::client::entities::saml::{
        AcsUrl, ClientSamlConfig, NameIdFormat, SamlAttributeMapper, SamlAttributeName,
        SamlAttributeNameFormat, SamlAttributeSource, SpEntityId,
    },
    domain::common::entities::app_errors::CoreError,
    entity::{client_saml_attribute_mappers, client_saml_configs},
};

impl TryFrom<client_saml_configs::Model> for ClientSamlConfig {
    type Error = CoreError;

    fn try_from(model: client_saml_configs::Model) -> Result<Self, Self::Error> {
        Ok(ClientSamlConfig {
            client_id: model.client_id,
            realm_id: model.realm_id.into(),
            sp_entity_id: SpEntityId::from_str(&model.sp_entity_id)?,
            acs_url: AcsUrl::from_str(&model.acs_url)?,
            name_id_format: NameIdFormat::from_str(&model.name_id_format)?,
            sign_assertions: model.sign_assertions,
            sign_documents: model.sign_documents,
            want_authn_requests_signed: model.want_authn_requests_signed,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        })
    }
}

impl TryFrom<client_saml_attribute_mappers::Model> for SamlAttributeMapper {
    type Error = CoreError;

    fn try_from(model: client_saml_attribute_mappers::Model) -> Result<Self, Self::Error> {
        Ok(SamlAttributeMapper {
            id: model.id,
            client_id: model.client_id,
            name: SamlAttributeName::from_str(&model.name)?,
            name_format: SamlAttributeNameFormat::from_str(&model.name_format)?,
            source: SamlAttributeSource::from_str(&model.source)?,
            created_at: Utc.from_utc_datetime(&model.created_at),
            updated_at: Utc.from_utc_datetime(&model.updated_at),
        })
    }
}
