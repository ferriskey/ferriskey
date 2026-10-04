use crate::{
    domain::common::entities::app_errors::CoreError,
    entity::clients::{ActiveModel, Entity as ClientEntity},
    infrastructure::common::is_unique_violation,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QuerySelect, QueryTrait, Select,
};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::realm::entities::{RealmId, RealmScope, Scoped, Unscoped};
use crate::domain::{
    client::{
        entities::{Client, ClientFilter, ClientSortField, redirect_uri::RedirectUri},
        ports::ClientRepository,
        value_objects::{CreateClientRequest, UpdateClientRequest},
    },
    common::{
        generate_timestamp, generate_uuid_v7,
        pagination::{Page, PageRequest},
    },
};
use crate::entity::{clients, redirect_uris};
use crate::infrastructure::pagination::{SortColumn, contains, paginate};

impl SortColumn<clients::Entity> for ClientSortField {
    fn column(&self) -> clients::Column {
        match self {
            ClientSortField::Name => clients::Column::Name,
            ClientSortField::ClientId => clients::Column::ClientId,
            ClientSortField::Enabled => clients::Column::Enabled,
            ClientSortField::CreatedAt => clients::Column::CreatedAt,
            ClientSortField::UpdatedAt => clients::Column::UpdatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &ClientFilter) -> Select<clients::Entity> {
    clients::Entity::find()
        .filter(clients::Column::RealmId.eq(realm_id))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(clients::Column::Name, value))
        })
        .apply_if(filter.client_id.as_deref(), |select, value| {
            select.filter(contains(clients::Column::ClientId, value))
        })
        .apply_if(filter.enabled, |select, value| {
            select.filter(clients::Column::Enabled.eq(value))
        })
        .apply_if(filter.public_client, |select, value| {
            select.filter(clients::Column::PublicClient.eq(value))
        })
        .apply_if(filter.service_account_enabled, |select, value| {
            select.filter(clients::Column::ServiceAccountEnabled.eq(value))
        })
        .apply_if(filter.protocol, |select, protocol| {
            select.filter(clients::Column::Protocol.eq(protocol.as_str()))
        })
        .apply_if(filter.client_type.as_ref(), |select, client_type| {
            select.filter(clients::Column::ClientType.eq(client_type.to_string()))
        })
        .apply_if(filter.has_redirect_uris, |select, present| {
            let with_redirects = redirect_uris::Entity::find()
                .select_only()
                .column(redirect_uris::Column::ClientId)
                .into_query();
            select.filter(if present {
                clients::Column::Id.in_subquery(with_redirects)
            } else {
                clients::Column::Id.not_in_subquery(with_redirects)
            })
        })
        .apply_if(filter.maintenance_enabled, |select, value| {
            select.filter(if value {
                Condition::all().add(clients::Column::MaintenanceEnabled.eq(true))
            } else {
                Condition::any()
                    .add(clients::Column::MaintenanceEnabled.eq(false))
                    .add(clients::Column::MaintenanceEnabled.is_null())
            })
        })
        .apply_if(filter.ids.as_deref(), |select, ids| {
            select.filter(clients::Column::Id.is_in(ids.iter().copied()))
        })
}

#[derive(Debug, Clone)]
pub struct PostgresClientRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientRepository for PostgresClientRepository {
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, CoreError> {
        let (now, _) = generate_timestamp();
        let client_id_for_error = data.client_id.clone();

        let payload = ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(data.realm_id.into()),
            name: Set(data.name),
            client_id: Set(data.client_id),
            secret: Set(data.secret),
            enabled: Set(data.enabled),
            protocol: Set(data.protocol.to_string()),
            public_client: Set(data.public_client),
            service_account_enabled: Set(data.service_account_enabled),
            direct_access_grants_enabled: Set(Some(data.direct_access_grants_enabled)),
            oauth_device_code_grant_enabled: Set(Some(data.oauth_device_code_grant_enabled)),
            client_type: Set(data.client_type.to_string()),
            access_token_lifetime_secs: Set(None),
            refresh_token_lifetime_secs: Set(None),
            id_token_lifetime_secs: Set(None),
            temporary_token_lifetime_secs: Set(None),
            maintenance_enabled: Set(Some(false)),
            maintenance_reason: Set(None),
            maintenance_session_strategy: Set(None),
            require_pkce: Set(Some(data.require_pkce)),
            token_exchange_enabled: Set(data.token_exchange_enabled),
            backchannel_logout_uri: Set(None),
            backchannel_logout_session_required: Set(true),
            consent_required: Set(data.consent_required),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_local()),
        };

        let client = payload.insert(&self.db).await.map_err(|e| {
            if is_unique_violation(&e) {
                return CoreError::ClientIdAlreadyExists(client_id_for_error);
            }
            tracing::error!("Failed to insert client: {}", e);
            CoreError::InternalServerError
        })?;

        let client = client.into();

        Ok(client)
    }

    #[instrument]
    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: RealmId,
    ) -> Result<Unscoped<Client>, CoreError> {
        let client = ClientEntity::find()
            .filter(crate::entity::clients::Column::ClientId.eq(client_id))
            .filter(crate::entity::clients::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .map(Client::from)
            .ok_or(CoreError::NotFound)?;

        Ok(Unscoped::new(client))
    }

    async fn get_by_id(
        &self,
        realm_id: RealmId,
        id: uuid::Uuid,
    ) -> Result<Unscoped<Client>, CoreError> {
        let clients_model = ClientEntity::find()
            .filter(crate::entity::clients::Column::Id.eq(id))
            .filter(crate::entity::clients::Column::RealmId.eq(Uuid::from(realm_id)))
            .find_with_related(crate::entity::redirect_uris::Entity)
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if clients_model.is_empty() {
            return Err(CoreError::NotFound);
        }

        let (client_model, uri_models) = &clients_model[0];

        let mut client: Client = client_model.clone().into();

        let redirect_uris: Vec<RedirectUri> = uri_models
            .iter()
            .map(|uri_model| uri_model.clone().into())
            .collect();

        client.redirect_uris = Some(redirect_uris);

        Ok(Unscoped::new(client))
    }

    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<Client>, CoreError> {
        let clients = ClientEntity::find()
            .filter(crate::entity::clients::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let clients: Vec<Client> = clients.into_iter().map(|c| c.into()).collect();

        Ok(clients)
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<ClientFilter, ClientSortField>,
    ) -> Result<Page<Client>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("error listing clients: {:?}", e);
            CoreError::InternalServerError
        })?;
        let clients = models.into_iter().map(Client::from).collect();

        Ok(Page::new(clients, total, request.page, request.limit))
    }

    async fn update_client(
        &self,
        client: &Scoped<Client>,
        data: UpdateClientRequest,
    ) -> Result<Client, CoreError> {
        let client = ClientEntity::find()
            .filter(crate::entity::clients::Column::Id.eq(client.get().id))
            .filter(crate::entity::clients::Column::RealmId.eq(Uuid::from(client.get().realm_id)))
            .one(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let mut client: ActiveModel = client.into();
        client.name = match data.name {
            Some(name) => Set(name),
            None => client.name,
        };

        client.client_id = match data.client_id {
            Some(client_id) => Set(client_id),
            None => client.client_id,
        };

        client.enabled = match data.enabled {
            Some(enabled) => Set(enabled),
            None => client.enabled,
        };

        client.direct_access_grants_enabled = match data.direct_access_grants_enabled {
            Some(enabled) => Set(Some(enabled)),
            None => client.direct_access_grants_enabled,
        };

        client.oauth_device_code_grant_enabled = match data.oauth_device_code_grant_enabled {
            Some(enabled) => Set(Some(enabled)),
            None => client.oauth_device_code_grant_enabled,
        };

        client.require_pkce = match data.require_pkce {
            Some(v) => Set(Some(v)),
            None => client.require_pkce,
        };

        client.token_exchange_enabled = match data.token_exchange_enabled {
            Some(enabled) => Set(enabled),
            None => client.token_exchange_enabled,
        };

        client.backchannel_logout_uri = match data.backchannel_logout_uri {
            Some(uri) => Set(uri),
            None => client.backchannel_logout_uri,
        };

        client.backchannel_logout_session_required = match data.backchannel_logout_session_required
        {
            Some(required) => Set(required),
            None => client.backchannel_logout_session_required,
        };

        client.consent_required = match data.consent_required {
            Some(required) => Set(required),
            None => client.consent_required,
        };

        client.access_token_lifetime_secs = Set(data.access_token_lifetime.map(|v| v as i32));
        client.refresh_token_lifetime_secs = Set(data.refresh_token_lifetime.map(|v| v as i32));
        client.id_token_lifetime_secs = Set(data.id_token_lifetime.map(|v| v as i32));
        client.temporary_token_lifetime_secs = Set(data.temporary_token_lifetime.map(|v| v as i32));

        client.maintenance_enabled = match data.maintenance_enabled {
            Some(enabled) => Set(Some(enabled)),
            None => client.maintenance_enabled,
        };
        client.maintenance_reason = match data.maintenance_reason {
            Some(reason) => Set(reason),
            None => client.maintenance_reason,
        };
        client.maintenance_session_strategy = match data.maintenance_session_strategy {
            Some(strategy) => Set(Some(strategy.to_string())),
            None => client.maintenance_session_strategy,
        };

        client.updated_at = Set(Utc::now().naive_utc());

        let client = client
            .update(&self.db)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(client.into())
    }

    async fn delete_by_id(&self, client: &Scoped<Client>) -> Result<(), CoreError> {
        let result = ClientEntity::delete_many()
            .filter(crate::entity::clients::Column::Id.eq(client.get().id))
            .filter(crate::entity::clients::Column::RealmId.eq(Uuid::from(client.get().realm_id)))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete client: {}", e);
                CoreError::InternalServerError
            })?;

        if result.rows_affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::{ClientFilter, ClientType};

    fn sql(filter: &ClientFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&ClientFilter::default());
        assert!(
            sql.contains(r#""clients"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&ClientFilter {
            name: Some("a%".to_string()),
            client_id: Some("b".to_string()),
            ..ClientFilter::default()
        });
        assert!(sql.contains(r#""clients"."name" ILIKE E'%a\\%%'"#), "{sql}");
        assert!(
            sql.contains(r#""clients"."client_id" ILIKE '%b%'"#),
            "{sql}"
        );
    }

    #[test]
    fn exact_filters_are_equalities() {
        let sql = sql(&ClientFilter {
            enabled: Some(true),
            public_client: Some(false),
            service_account_enabled: Some(true),
            protocol: Some(AuthProtocol::Saml),
            client_type: Some(ClientType::System),
            ..ClientFilter::default()
        });
        assert!(sql.contains(r#""clients"."enabled" = TRUE"#), "{sql}");
        assert!(
            sql.contains(r#""clients"."public_client" = FALSE"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""clients"."service_account_enabled" = TRUE"#),
            "{sql}"
        );
        assert!(sql.contains(r#""clients"."protocol" = 'saml'"#), "{sql}");
        assert!(
            sql.contains(r#""clients"."client_type" = 'system'"#),
            "{sql}"
        );
    }

    #[test]
    fn redirect_presence_is_a_subquery_on_redirect_uris() {
        let with = sql(&ClientFilter {
            has_redirect_uris: Some(true),
            ..ClientFilter::default()
        });
        assert!(
            with.contains(
                r#""clients"."id" IN (SELECT "redirect_uris"."client_id" FROM "redirect_uris")"#
            ),
            "{with}"
        );
        let without = sql(&ClientFilter {
            has_redirect_uris: Some(false),
            ..ClientFilter::default()
        });
        assert!(
            without.contains(
                r#""clients"."id" NOT IN (SELECT "redirect_uris"."client_id" FROM "redirect_uris")"#
            ),
            "{without}"
        );
    }

    #[test]
    fn maintenance_off_includes_unset_rows() {
        let on = sql(&ClientFilter {
            maintenance_enabled: Some(true),
            ..ClientFilter::default()
        });
        assert!(
            on.contains(r#""clients"."maintenance_enabled" = TRUE"#),
            "{on}"
        );
        let off = sql(&ClientFilter {
            maintenance_enabled: Some(false),
            ..ClientFilter::default()
        });
        assert!(
            off.contains(r#"("clients"."maintenance_enabled" = FALSE OR "clients"."maintenance_enabled" IS NULL)"#),
            "{off}"
        );
    }

    #[test]
    fn ids_filter_is_an_in_list() {
        let first = Uuid::from_u128(1);
        let second = Uuid::from_u128(2);
        let sql = sql(&ClientFilter {
            ids: Some(vec![first, second]),
            ..ClientFilter::default()
        });
        assert!(
            sql.contains(&format!(r#""clients"."id" IN ('{first}', '{second}')"#)),
            "{sql}"
        );
    }
}
