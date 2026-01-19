use crate::domain::identity_provider::{
    IdentityProvider, IdentityProviderConfig, IdentityProviderId,
};
use crate::domain::realm::entities::RealmId;
use crate::entity::identity_providers::Model;

impl From<Model> for IdentityProvider {
    fn from(model: Model) -> Self {
        let raw_config = model.config;
        let config = serde_json::from_value(raw_config.clone()).unwrap_or(IdentityProviderConfig {
            client_id: None,
            client_secret: None,
            extra: raw_config,
        });

        IdentityProvider {
            id: IdentityProviderId::from(model.id),
            realm_id: RealmId::from(model.realm_id),
            alias: model.alias,
            provider_id: model.provider_id,
            enabled: model.enabled,
            display_name: model.display_name,
            first_broker_login_flow_alias: model.first_broker_login_flow_alias,
            post_broker_login_flow_alias: model.post_broker_login_flow_alias,
            store_token: model.store_token,
            add_read_token_role_on_create: model.add_read_token_role_on_create,
            trust_email: model.trust_email,
            link_only: model.link_only,
            config,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
        }
    }
}
