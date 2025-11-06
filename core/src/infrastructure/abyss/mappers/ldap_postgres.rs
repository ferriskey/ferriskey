use chrono::{TimeZone, Utc};

use crate::{domain::abyss::entities::ldap_provider::LdapProvider, entity::ldap_providers::Model};

impl From<Model> for LdapProvider {
    fn from(value: Model) -> Self {
        let created_at = Utc.from_utc_datetime(&value.created_at);
        let updated_at = Utc.from_utc_datetime(&value.updated_at);

        LdapProvider {
            id: value.id,
            realm_id: value.realm_id,
            url: value.url,
            bind_dn: value.bind_dn,
            bind_password: value.bind_password,
            user_base_dn: value.user_base_dn,
            user_filter: value.user_filter,
            display_name_attr: value.display_name_attr,
            email_attr: value.email_attr,
            enabled: value.enabled,
            username_attr: value.username_attr,
            created_at,
            updated_at,
        }
    }
}
