use sea_orm::ExprTrait;
use std::collections::HashMap;

use crate::{
    domain::common::entities::app_errors::CoreError,
    entity::clients::{ActiveModel, Entity as ClientEntity},
    infrastructure::common::is_unique_violation,
};
use chrono::{DateTime, Utc};
use ferriskey_security::crypto::secret_cipher::SecretCipher;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::Set,
    ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait, Select,
    Statement, TransactionTrait,
    sea_query::{Expr, Func},
};
use tracing::{error, instrument, warn};
use uuid::Uuid;

use crate::domain::client::entities::ExpiredClientSecret;
use crate::domain::realm::entities::{RealmId, RealmScope, Scoped, Unscoped};
use crate::domain::{
    client::{
        entities::{
            ApplicationType, Client, ClientFilter, ClientRegistrationSource, ClientSortField,
            redirect_uri::RedirectUri,
        },
        ports::ClientRepository,
        value_objects::{CreateClientRequest, UpdateClientRequest},
    },
    common::{
        generate_timestamp, generate_uuid_v7,
        pagination::{Page, PageRequest},
    },
};
use crate::entity::{clients, redirect_uris};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within_naive};
use ferriskey_domain::client::commands::SecretRotation;

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

fn device_grant() -> Expr {
    Expr::expr(Func::coalesce([
        Expr::col((
            clients::Entity,
            clients::Column::OauthDeviceCodeGrantEnabled,
        )),
        Expr::val(false),
    ]))
}

fn with_redirect_uris() -> sea_orm::sea_query::SelectStatement {
    redirect_uris::Entity::find()
        .select_only()
        .column(redirect_uris::Column::ClientId)
        .into_query()
}

fn device_only() -> Condition {
    Condition::all()
        .add(device_grant().eq(true))
        .add(clients::Column::Id.not_in_subquery(with_redirect_uris()))
}

fn application(kind: ApplicationType) -> Condition {
    let interactive = Condition::all()
        .add(clients::Column::ServiceAccountEnabled.eq(false))
        .add(device_only().not());
    let public_type = |public_client: bool| {
        interactive
            .clone()
            .add(clients::Column::ClientType.eq("public"))
            .add(clients::Column::PublicClient.eq(public_client))
    };
    match kind {
        ApplicationType::M2m => {
            Condition::all().add(clients::Column::ServiceAccountEnabled.eq(true))
        }
        ApplicationType::Device => Condition::all()
            .add(clients::Column::ServiceAccountEnabled.eq(false))
            .add(device_only()),
        ApplicationType::Spa => public_type(true),
        ApplicationType::Native => public_type(false),
        ApplicationType::Web => interactive.add(clients::Column::ClientType.ne("public")),
    }
}

fn search(value: &str) -> Condition {
    Condition::any()
        .add(contains(clients::Column::Name, value))
        .add(contains(clients::Column::ClientId, value))
}

fn listing_select(realm_id: Uuid, filter: &ClientFilter) -> Select<clients::Entity> {
    clients::Entity::find()
        .filter(clients::Column::RealmId.eq(realm_id))
        .filter(within_naive(clients::Column::CreatedAt, &filter.created))
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(search(value))
        })
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(clients::Column::Name, value))
        })
        .apply_if(filter.client_id.as_deref(), |select, value| {
            select.filter(contains(clients::Column::ClientId, value))
        })
        .apply_if(filter.client_id_exact.as_deref(), |select, value| {
            select.filter(clients::Column::ClientId.eq(value))
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
        .apply_if(filter.oauth_device_code_grant_enabled, |select, value| {
            select.filter(device_grant().eq(value))
        })
        .apply_if(filter.application_type, |select, kind| {
            select.filter(application(kind))
        })
        .apply_if(filter.protocol, |select, protocol| {
            select.filter(clients::Column::Protocol.eq(protocol.as_str()))
        })
        .apply_if(filter.client_type.as_ref(), |select, client_type| {
            select.filter(clients::Column::ClientType.eq(client_type.to_string()))
        })
        .apply_if(filter.has_redirect_uris, |select, present| {
            select.filter(if present {
                clients::Column::Id.in_subquery(with_redirect_uris())
            } else {
                clients::Column::Id.not_in_subquery(with_redirect_uris())
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

pub const CLIENT_SECRET_PURPOSE: &str = "clients.secret";
pub const CLIENT_PREVIOUS_SECRET_PURPOSE: &str = "clients.previous_secret";

#[derive(Debug, Clone)]
pub struct PostgresClientRepository {
    pub db: DatabaseConnection,
    cipher: SecretCipher,
}

impl PostgresClientRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_cipher(db, SecretCipher::disabled())
    }

    pub fn with_cipher(db: DatabaseConnection, cipher: SecretCipher) -> Self {
        Self { db, cipher }
    }

    fn seal_secret(&self, id: Uuid, secret: Option<String>) -> Result<Option<String>, CoreError> {
        secret
            .map(|secret| {
                self.cipher
                    .seal(CLIENT_SECRET_PURPOSE, &id.to_string(), &secret)
                    .map_err(|error| CoreError::Configuration(error.to_string()))
            })
            .transpose()
    }

    fn seal_previous_secret(&self, id: Uuid, secret: &str) -> Result<String, CoreError> {
        self.cipher
            .seal(CLIENT_PREVIOUS_SECRET_PURPOSE, &id.to_string(), secret)
            .map_err(|error| CoreError::Configuration(error.to_string()))
    }

    fn open_secret(&self, purpose: &str, id: Uuid, stored: &str) -> Result<String, CoreError> {
        self.cipher
            .open(purpose, &id.to_string(), stored)
            .map_err(|error| CoreError::Configuration(error.to_string()))
    }

    fn client_of(&self, mut model: clients::Model) -> Result<Client, CoreError> {
        if let Some(stored) = model.secret.as_deref() {
            model.secret = Some(self.open_secret(CLIENT_SECRET_PURPOSE, model.id, stored)?);
        }
        if let Some(stored) = model.previous_secret.take() {
            match self.open_secret(CLIENT_PREVIOUS_SECRET_PURPOSE, model.id, &stored) {
                Ok(previous) => model.previous_secret = Some(previous),
                Err(error) => {
                    warn!(
                        client = %model.id,
                        ?error,
                        "the previous client secret cannot be opened, it is ignored until it expires"
                    );
                    model.previous_secret_expires_at = None;
                }
            }
        }

        Ok(Client::from(model))
    }

    /// Every field but the secrets, which are never opened: a stored one reads as an
    /// empty placeholder, so the client still shows one is configured. For listing and
    /// for the result of an edit, which only reach the API (where the secret always
    /// serializes masked) and never client authentication.
    fn metadata_of(mut model: clients::Model) -> Client {
        if model.secret.is_some() {
            model.secret = Some(String::new());
        }
        if model.previous_secret.is_some() {
            model.previous_secret = Some(String::new());
        }

        Client::from(model)
    }
}

impl ClientRepository for PostgresClientRepository {
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, CoreError> {
        let (now, _) = generate_timestamp();
        let client_id_for_error = data.client_id.clone();

        let id = generate_uuid_v7();
        let payload = ActiveModel {
            id: Set(id),
            realm_id: Set(data.realm_id.into()),
            name: Set(data.name),
            client_id: Set(data.client_id),
            secret: Set(self.seal_secret(id, data.secret)?),
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
            require_pushed_authorization_requests: Set(false),
            token_exchange_enabled: Set(data.token_exchange_enabled),
            backchannel_logout_uri: Set(None),
            backchannel_logout_session_required: Set(true),
            consent_required: Set(data.consent_required),
            registration_source: Set(data.registration_source.to_string()),
            token_endpoint_auth_method: Set(data.auth.method.to_string()),
            jwks: Set(data.auth.jwks.map(|jwks| jwks.0)),
            jwks_uri: Set(data.auth.jwks_uri),
            previous_secret: Set(None),
            previous_secret_expires_at: Set(None),
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

        self.client_of(client)
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
            .ok_or(CoreError::NotFound)?;

        Ok(Unscoped::new(self.client_of(client)?))
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

        let mut client = self.client_of(client_model.clone())?;

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

        clients
            .into_iter()
            .map(|model| self.client_of(model))
            .collect()
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
        let ids: Vec<Uuid> = models.iter().map(|model| model.id).collect();
        let mut uris: HashMap<Uuid, Vec<RedirectUri>> = HashMap::new();
        if !ids.is_empty() {
            let rows = redirect_uris::Entity::find()
                .filter(redirect_uris::Column::ClientId.is_in(ids))
                .order_by_asc(redirect_uris::Column::CreatedAt)
                .order_by_asc(redirect_uris::Column::Id)
                .all(&self.db)
                .await
                .map_err(|e| {
                    error!("error loading the redirect uris of listed clients: {:?}", e);
                    CoreError::InternalServerError
                })?;
            for row in rows {
                uris.entry(row.client_id).or_default().push(row.into());
            }
        }

        let clients = models
            .into_iter()
            .map(|model| {
                let redirect_uris = uris.remove(&model.id).unwrap_or_default();
                let mut client = Self::metadata_of(model);
                client.redirect_uris = Some(redirect_uris);
                client
            })
            .collect();

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

        if let Some(required) = data.require_pushed_authorization_requests {
            client.require_pushed_authorization_requests = Set(required);
        }

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

        if let Some(method) = data.token_endpoint_auth_method {
            client.token_endpoint_auth_method = Set(method.to_string());
        }

        if let Some(jwks) = data.jwks {
            client.jwks = Set(jwks.map(|jwks| jwks.0));
        }

        if let Some(jwks_uri) = data.jwks_uri {
            client.jwks_uri = Set(jwks_uri);
        }

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

        Ok(Self::metadata_of(client))
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

    async fn rotate_secret(
        &self,
        client: &Scoped<Client>,
        new_secret: String,
        grace_period: Option<chrono::Duration>,
    ) -> Result<SecretRotation, CoreError> {
        let id = client.get().id;
        let transaction = self.db.begin().await.map_err(|e| {
            error!("failed to begin a client secret rotation: {:?}", e);
            CoreError::InternalServerError
        })?;

        let row = ClientEntity::find()
            .filter(clients::Column::Id.eq(id))
            .filter(clients::Column::RealmId.eq(Uuid::from(client.get().realm_id)))
            .lock_exclusive()
            .one(&transaction)
            .await
            .map_err(|_| CoreError::InternalServerError)?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now();
        let replaced_expired = match (&row.previous_secret, row.previous_secret_expires_at) {
            (Some(_), Some(expired_at)) if expired_at.and_utc() <= now => {
                Some(ExpiredClientSecret {
                    client_id: id,
                    realm_id: RealmId::from(row.realm_id),
                    expired_at: expired_at.and_utc(),
                })
            }
            _ => None,
        };
        let previous_expires_at = grace_period.map(|grace| now + grace);

        let current = row
            .secret
            .as_deref()
            .map(|stored| self.open_secret(CLIENT_SECRET_PURPOSE, id, stored))
            .transpose()?;
        let previous = match (current, previous_expires_at) {
            (Some(secret), Some(expires_at)) => {
                Some((self.seal_previous_secret(id, &secret)?, expires_at))
            }
            _ => None,
        };

        let mut active = row.into_active_model();
        active.secret = Set(self.seal_secret(id, Some(new_secret))?);
        active.previous_secret = Set(previous.as_ref().map(|(secret, _)| secret.clone()));
        active.previous_secret_expires_at =
            Set(previous.map(|(_, expires_at)| expires_at.naive_utc()));
        active.updated_at = Set(now.naive_utc());

        let updated = active.update(&transaction).await.map_err(|e| {
            error!("failed to rotate a client secret: {:?}", e);
            CoreError::InternalServerError
        })?;
        transaction.commit().await.map_err(|e| {
            error!("failed to commit a client secret rotation: {:?}", e);
            CoreError::InternalServerError
        })?;

        Ok(SecretRotation {
            client: self.client_of(updated)?,
            replaced_expired,
        })
    }

    async fn revoke_previous_secret(&self, client: &Scoped<Client>) -> Result<bool, CoreError> {
        let result = ClientEntity::update_many()
            .col_expr(
                clients::Column::PreviousSecret,
                Expr::value(Option::<String>::None),
            )
            .col_expr(
                clients::Column::PreviousSecretExpiresAt,
                Expr::value(Option::<chrono::NaiveDateTime>::None),
            )
            .filter(clients::Column::Id.eq(client.get().id))
            .filter(clients::Column::RealmId.eq(Uuid::from(client.get().realm_id)))
            .filter(clients::Column::PreviousSecret.is_not_null())
            .filter(Expr::cust(
                "previous_secret_expires_at > (clock_timestamp() AT TIME ZONE 'UTC')",
            ))
            .exec(&self.db)
            .await
            .map_err(|e| {
                error!("failed to revoke a previous client secret: {:?}", e);
                CoreError::InternalServerError
            })?;

        Ok(result.rows_affected > 0)
    }

    async fn expire_previous_secrets(
        &self,
        now: DateTime<Utc>,
    ) -> Result<Vec<ExpiredClientSecret>, CoreError> {
        let rows = self
            .db
            .query_all_raw(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "WITH due AS (\
                     SELECT id, realm_id, previous_secret_expires_at AS expired_at FROM clients \
                     WHERE previous_secret_expires_at IS NOT NULL AND previous_secret_expires_at <= $1 \
                     FOR UPDATE), \
                 cleared AS (\
                     UPDATE clients SET previous_secret = NULL, previous_secret_expires_at = NULL \
                     FROM due WHERE clients.id = due.id RETURNING clients.id) \
                 SELECT due.id, due.realm_id, due.expired_at FROM due JOIN cleared ON cleared.id = due.id",
                [now.naive_utc().into()],
            ))
            .await
            .map_err(|e| {
                error!("failed to expire previous client secrets: {:?}", e);
                CoreError::InternalServerError
            })?;

        rows.into_iter()
            .map(|row| {
                let read = |e: sea_orm::DbErr| CoreError::Database(e.to_string());
                let expired_at: chrono::NaiveDateTime =
                    row.try_get("", "expired_at").map_err(read)?;
                Ok(ExpiredClientSecret {
                    client_id: row.try_get("", "id").map_err(read)?,
                    realm_id: RealmId::from(row.try_get::<Uuid>("", "realm_id").map_err(read)?),
                    expired_at: expired_at.and_utc(),
                })
            })
            .collect()
    }

    async fn count_by_source(
        &self,
        realm_id: RealmId,
        source: ClientRegistrationSource,
    ) -> Result<u64, CoreError> {
        ClientEntity::find()
            .filter(clients::Column::RealmId.eq(Uuid::from(realm_id)))
            .filter(clients::Column::RegistrationSource.eq(source.to_string()))
            .count(&self.db)
            .await
            .map_err(|e| {
                error!("error counting clients by registration source: {:?}", e);
                CoreError::InternalServerError
            })
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::{ApplicationType, ClientFilter, ClientType};
    use crate::domain::common::pagination::DateRange;

    fn sql(filter: &ClientFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn an_unbounded_created_range_adds_no_predicate() {
        let sql = sql(&ClientFilter::default());
        assert!(!str::contains(&sql, r#""clients"."created_at" >"#), "{sql}");
        assert!(!str::contains(&sql, r#""clients"."created_at" <"#), "{sql}");
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&ClientFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..ClientFilter::default()
        });
        assert!(
            str::contains(
                &sql,
                r#""clients"."created_at" >= '2026-01-01 00:00:00.000000'"#
            ),
            "{sql}"
        );
        assert!(
            str::contains(
                &sql,
                r#""clients"."created_at" < '2026-02-01 00:00:00.000000'"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&ClientFilter::default());
        assert!(
            str::contains(
                &sql,
                r#""clients"."realm_id" = '00000000-0000-0000-0000-000000000000'"#
            ),
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
        assert!(
            str::contains(&sql, r#""clients"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(
            str::contains(&sql, r#""clients"."client_id" ILIKE '%b%'"#),
            "{sql}"
        );
    }

    #[test]
    fn exact_client_id_is_a_case_sensitive_equality() {
        let sql = sql(&ClientFilter {
            client_id_exact: Some("App_%".to_string()),
            ..ClientFilter::default()
        });
        assert!(
            str::contains(&sql, r#""clients"."client_id" = 'App_%'"#),
            "{sql}"
        );
        assert!(!str::contains(&sql, "ILIKE"), "{sql}");
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
        assert!(
            str::contains(&sql, r#""clients"."enabled" = TRUE"#),
            "{sql}"
        );
        assert!(
            str::contains(&sql, r#""clients"."public_client" = FALSE"#),
            "{sql}"
        );
        assert!(
            str::contains(&sql, r#""clients"."service_account_enabled" = TRUE"#),
            "{sql}"
        );
        assert!(
            str::contains(&sql, r#""clients"."protocol" = 'saml'"#),
            "{sql}"
        );
        assert!(
            str::contains(&sql, r#""clients"."client_type" = 'system'"#),
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
            str::contains(
                &sql,
                &format!(r#""clients"."id" IN ('{first}', '{second}')"#)
            ),
            "{sql}"
        );
    }

    const DEVICE: &str = r#"COALESCE("clients"."oauth_device_code_grant_enabled", FALSE) = TRUE AND "clients"."id" NOT IN (SELECT "redirect_uris"."client_id" FROM "redirect_uris")"#;

    fn application(kind: ApplicationType) -> String {
        sql(&ClientFilter {
            application_type: Some(kind),
            ..ClientFilter::default()
        })
    }

    #[test]
    fn search_matches_the_name_or_the_client_id() {
        let sql = sql(&ClientFilter {
            search: Some("a%".to_string()),
            ..ClientFilter::default()
        });
        assert!(
            str::contains(
                &sql,
                r#"(("clients"."name" ILIKE E'%a\\%%') OR ("clients"."client_id" ILIKE E'%a\\%%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn device_grant_treats_unset_as_disabled() {
        let on = sql(&ClientFilter {
            oauth_device_code_grant_enabled: Some(true),
            ..ClientFilter::default()
        });
        assert!(
            on.contains(r#"COALESCE("clients"."oauth_device_code_grant_enabled", FALSE) = TRUE"#),
            "{on}"
        );
        let off = sql(&ClientFilter {
            oauth_device_code_grant_enabled: Some(false),
            ..ClientFilter::default()
        });
        assert!(
            off.contains(r#"COALESCE("clients"."oauth_device_code_grant_enabled", FALSE) = FALSE"#),
            "{off}"
        );
    }

    #[test]
    fn m2m_applications_hold_a_service_account() {
        let sql = application(ApplicationType::M2m);
        assert!(
            str::contains(&sql, r#""clients"."service_account_enabled" = TRUE"#),
            "{sql}"
        );
        assert!(!str::contains(&sql, "COALESCE"), "{sql}");
    }

    #[test]
    fn device_applications_use_the_device_grant_without_redirect() {
        let sql = application(ApplicationType::Device);
        assert!(
            str::contains(
                &sql,
                &format!(r#""clients"."service_account_enabled" = FALSE AND ({DEVICE})"#)
            ),
            "{sql}"
        );
    }

    #[test]
    fn spa_applications_are_public_and_public_clients() {
        let sql = application(ApplicationType::Spa);
        assert!(
            str::contains(
                &sql,
                &format!(
                    r#""clients"."service_account_enabled" = FALSE AND (NOT ({DEVICE})) AND "clients"."client_type" = 'public' AND "clients"."public_client" = TRUE"#
                )
            ),
            "{sql}"
        );
    }

    #[test]
    fn native_applications_are_public_without_public_client() {
        let sql = application(ApplicationType::Native);
        assert!(
            str::contains(
                &sql,
                &format!(
                    r#""clients"."service_account_enabled" = FALSE AND (NOT ({DEVICE})) AND "clients"."client_type" = 'public' AND "clients"."public_client" = FALSE"#
                )
            ),
            "{sql}"
        );
    }

    #[test]
    fn web_applications_are_every_other_client_type() {
        let sql = application(ApplicationType::Web);
        assert!(
            str::contains(
                &sql,
                &format!(
                    r#""clients"."service_account_enabled" = FALSE AND (NOT ({DEVICE})) AND "clients"."client_type" <> 'public'"#
                )
            ),
            "{sql}"
        );
    }
}

#[cfg(test)]
mod sealed_secret_tests {
    use super::*;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::ClientType;
    use crate::domain::client::entities::auth_method::ClientAuthSettings;
    use crate::infrastructure::sealed_secrets_testing::{migrated_schema, sealing_cipher, stored};

    fn confidential(realm_id: RealmId, client_id: &str, secret: &str) -> CreateClientRequest {
        CreateClientRequest {
            realm_id,
            name: client_id.to_string(),
            client_id: client_id.to_string(),
            secret: Some(secret.to_string()),
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: false,
            service_account_enabled: true,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            token_exchange_enabled: false,
            client_type: ClientType::Confidential,
            require_pkce: false,
            consent_required: false,
            registration_source: ClientRegistrationSource::Admin,
            auth: ClientAuthSettings::default_for(false),
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_secret_is_sealed_at_rest_and_read_back_in_clear() {
        let schema = migrated_schema("client_secret_test").await;
        let realm_id = schema.realm.id();
        let repository = PostgresClientRepository::with_cipher(schema.db, sealing_cipher());

        let created = repository
            .create_client(confidential(realm_id, "backend", "client-secret"))
            .await
            .expect("create the client");
        assert_eq!(created.secret_str(), Some("client-secret"));

        let at_rest = stored(&schema.pool, "clients", "secret", created.id)
            .await
            .expect("a stored secret");
        assert!(at_rest.starts_with("fk1:kid-a:"), "{at_rest}");
        assert!(!at_rest.contains("client-secret"));

        let by_client_id = repository
            .get_by_client_id("backend".to_string(), realm_id)
            .await
            .expect("read by client id")
            .across_realms();
        let by_id = repository
            .get_by_id(realm_id, created.id)
            .await
            .expect("read by id")
            .across_realms();
        let in_realm = repository
            .get_by_realm_id(realm_id)
            .await
            .expect("read the realm clients");
        assert_eq!(by_client_id.secret_str(), Some("client-secret"));
        assert_eq!(by_id.secret_str(), Some("client-secret"));
        assert_eq!(in_realm[0].secret_str(), Some("client-secret"));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_clear_secret_is_still_read_when_a_cipher_is_configured() {
        let schema = migrated_schema("client_secret_test").await;
        let realm_id = schema.realm.id();
        PostgresClientRepository::new(schema.db.clone())
            .create_client(confidential(realm_id, "legacy", "legacy-secret"))
            .await
            .expect("create a legacy client");

        let read = PostgresClientRepository::with_cipher(schema.db, sealing_cipher())
            .get_by_client_id("legacy".to_string(), realm_id)
            .await
            .expect("read the legacy client")
            .across_realms();

        assert_eq!(read.secret_str(), Some("legacy-secret"));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_sealed_secret_is_refused_by_an_instance_without_the_key() {
        let schema = migrated_schema("client_secret_test").await;
        let realm_id = schema.realm.id();
        PostgresClientRepository::with_cipher(schema.db.clone(), sealing_cipher())
            .create_client(confidential(realm_id, "sealed", "client-secret"))
            .await
            .expect("create the client");

        let result = PostgresClientRepository::new(schema.db)
            .get_by_client_id("sealed".to_string(), realm_id)
            .await;

        assert!(result.is_err(), "{result:?}");
    }

    fn public(realm_id: RealmId, client_id: &str) -> CreateClientRequest {
        CreateClientRequest {
            secret: None,
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            auth: ClientAuthSettings::default_for(true),
            ..confidential(realm_id, client_id, "unused")
        }
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_client_whose_secret_cannot_be_opened_is_still_listed() {
        let schema = migrated_schema("client_secret_test").await;
        let realm_id = schema.realm.id();
        let sealed = PostgresClientRepository::with_cipher(schema.db.clone(), sealing_cipher())
            .create_client(confidential(realm_id, "sealed", "client-secret"))
            .await
            .expect("create the sealed client");
        let without_key = PostgresClientRepository::new(schema.db);
        let open = without_key
            .create_client(public(realm_id, "spa"))
            .await
            .expect("create the public client");

        let page = without_key
            .list(&schema.realm, &PageRequest::default())
            .await
            .expect("list the clients");

        let listed = |id: Uuid| {
            page.data()
                .iter()
                .find(|client| client.id == id)
                .unwrap_or_else(|| panic!("client {id} is listed"))
        };
        assert_eq!(listed(sealed.id).secret_str(), Some(""));
        assert_eq!(listed(open.id).secret_str(), None);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn an_edit_of_a_client_whose_secret_cannot_be_opened_reports_success() {
        let schema = migrated_schema("client_secret_test").await;
        let realm_id = schema.realm.id();
        let with_key = PostgresClientRepository::with_cipher(schema.db.clone(), sealing_cipher());
        let created = with_key
            .create_client(confidential(realm_id, "sealed", "client-secret"))
            .await
            .expect("create the sealed client");
        let target = with_key
            .get_by_id(realm_id, created.id)
            .await
            .expect("read the client")
            .in_realm(&schema.realm)
            .expect("the client is in the realm");

        let updated = PostgresClientRepository::new(schema.db)
            .update_client(
                &target,
                UpdateClientRequest {
                    name: Some("renamed".to_string()),
                    client_id: None,
                    enabled: None,
                    direct_access_grants_enabled: None,
                    oauth_device_code_grant_enabled: None,
                    token_exchange_enabled: None,
                    require_pkce: None,
                    require_pushed_authorization_requests: None,
                    backchannel_logout_uri: None,
                    backchannel_logout_session_required: None,
                    access_token_lifetime: None,
                    refresh_token_lifetime: None,
                    id_token_lifetime: None,
                    temporary_token_lifetime: None,
                    maintenance_enabled: None,
                    maintenance_reason: None,
                    maintenance_session_strategy: None,
                    consent_required: None,
                    token_endpoint_auth_method: None,
                    jwks: None,
                    jwks_uri: None,
                },
            )
            .await
            .expect("the edit reports success");

        assert_eq!(updated.name, "renamed");
        assert_eq!(updated.secret_str(), Some(""));
        let reread = with_key
            .get_by_id(realm_id, created.id)
            .await
            .expect("read the client back")
            .across_realms();
        assert_eq!(reread.secret_str(), Some("client-secret"));
    }
}

#[cfg(test)]
mod secret_rotation_tests {
    use chrono::Duration;

    use super::*;
    use crate::domain::authentication::entities::AuthProtocol;
    use crate::domain::client::entities::ClientType;
    use crate::domain::client::entities::auth_method::ClientAuthSettings;
    use crate::infrastructure::sealed_secrets_testing::{
        TestSchema, migrated_schema, sealing_cipher, stored,
    };

    fn confidential(realm_id: RealmId, client_id: &str) -> CreateClientRequest {
        CreateClientRequest {
            realm_id,
            name: client_id.to_string(),
            client_id: client_id.to_string(),
            secret: Some("first-secret".to_string()),
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: false,
            service_account_enabled: true,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            token_exchange_enabled: false,
            client_type: ClientType::Confidential,
            require_pkce: false,
            consent_required: false,
            registration_source: ClientRegistrationSource::Admin,
            auth: ClientAuthSettings::default_for(false),
        }
    }

    async fn seeded(client_id: &str) -> (PostgresClientRepository, TestSchema, Scoped<Client>) {
        let schema = migrated_schema("client_rotation_test").await;
        let repository = PostgresClientRepository::with_cipher(schema.db.clone(), sealing_cipher());
        let created = repository
            .create_client(confidential(schema.realm.id(), client_id))
            .await
            .expect("create the client");
        let client = repository
            .get_by_id(schema.realm.id(), created.id)
            .await
            .expect("read the client")
            .in_realm(&schema.realm)
            .expect("the client belongs to the realm");
        (repository, schema, client)
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn listing_never_hands_out_a_stored_previous_secret() {
        let (repository, schema, client) = seeded("backend").await;
        let expires_at = repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate")
            .client
            .previous_secret
            .expect("a previous secret")
            .expires_at;

        let page = repository
            .list(&schema.realm, &PageRequest::default())
            .await
            .expect("list the clients");

        let listed = page
            .data()
            .iter()
            .find(|listed| listed.id == client.get().id)
            .expect("the client is listed");
        let previous = listed.previous_secret.as_ref().expect("a previous secret");
        assert_eq!(previous.secret.expose(), "");
        assert_eq!(previous.expires_at, expires_at);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn rotating_keeps_the_old_secret_as_previous_until_its_expiry_and_seals_both() {
        let (repository, schema, client) = seeded("backend").await;
        let before = Utc::now();

        let rotated = repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate")
            .client;

        assert_eq!(rotated.secret_str(), Some("second-secret"));
        let previous = rotated.previous_secret.as_ref().expect("a previous secret");
        assert_eq!(previous.secret.expose(), "first-secret");
        assert!(previous.expires_at >= before + Duration::hours(1));
        assert!(previous.expires_at <= Utc::now() + Duration::hours(1));

        let current_at_rest = stored(&schema.pool, "clients", "secret", client.get().id)
            .await
            .expect("a stored secret");
        let previous_at_rest = stored(&schema.pool, "clients", "previous_secret", client.get().id)
            .await
            .expect("a stored previous secret");
        for value in [&current_at_rest, &previous_at_rest] {
            assert!(value.starts_with("fk1:kid-a:"), "{value}");
            assert!(!value.contains("-secret"));
        }
        assert_ne!(current_at_rest, previous_at_rest);

        let read = repository
            .get_by_client_id("backend".to_string(), schema.realm.id())
            .await
            .expect("read back")
            .across_realms();
        assert_eq!(read.secret_str(), Some("second-secret"));
        assert_eq!(
            read.previous_secret
                .expect("a previous secret")
                .secret
                .expose(),
            "first-secret"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn rotating_without_a_grace_period_keeps_no_previous_secret() {
        let (repository, schema, client) = seeded("backend").await;

        let rotated = repository
            .rotate_secret(&client, "second-secret".to_string(), None)
            .await
            .expect("rotate")
            .client;

        assert_eq!(rotated.secret_str(), Some("second-secret"));
        assert!(rotated.previous_secret.is_none());
        assert_eq!(
            stored(&schema.pool, "clients", "previous_secret", client.get().id).await,
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn rotating_again_replaces_the_previous_secret_and_drops_the_older_one() {
        let (repository, _schema, client) = seeded("backend").await;
        repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("first rotation");

        let rotated = repository
            .rotate_secret(
                &client,
                "third-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("second rotation")
            .client;

        assert_eq!(rotated.secret_str(), Some("third-secret"));
        assert_eq!(
            rotated
                .previous_secret
                .expect("a previous secret")
                .secret
                .expose(),
            "second-secret"
        );
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_drops_the_previous_secret_once() {
        let (repository, schema, client) = seeded("backend").await;
        repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate");

        assert!(
            repository
                .revoke_previous_secret(&client)
                .await
                .expect("revoke")
        );
        assert!(
            !repository
                .revoke_previous_secret(&client)
                .await
                .expect("revoke again")
        );

        let read = repository
            .get_by_id(schema.realm.id(), client.get().id)
            .await
            .expect("read back")
            .across_realms();
        assert!(read.previous_secret.is_none());
        assert_eq!(read.secret_str(), Some("second-secret"));
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn expiring_reports_each_ended_grace_period_once_and_spares_the_others() {
        let (repository, schema, ended) = seeded("ended").await;
        let running = repository
            .create_client(confidential(schema.realm.id(), "running"))
            .await
            .expect("create the second client");
        let running = repository
            .get_by_id(schema.realm.id(), running.id)
            .await
            .expect("read the second client")
            .in_realm(&schema.realm)
            .expect("in realm");
        let now = Utc::now();
        repository
            .rotate_secret(&ended, "next".to_string(), Some(Duration::minutes(1)))
            .await
            .expect("rotate the first");
        repository
            .rotate_secret(&running, "next".to_string(), Some(Duration::hours(2)))
            .await
            .expect("rotate the second");

        assert!(
            repository
                .expire_previous_secrets(now)
                .await
                .expect("nothing is due yet")
                .is_empty()
        );

        let expired = repository
            .expire_previous_secrets(now + Duration::minutes(2))
            .await
            .expect("expire");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].client_id, ended.get().id);
        assert_eq!(expired[0].realm_id, schema.realm.id());
        assert!(
            repository
                .expire_previous_secrets(now + Duration::minutes(2))
                .await
                .expect("expire again")
                .is_empty()
        );

        let still_running = repository
            .get_by_id(schema.realm.id(), running.get().id)
            .await
            .expect("read back")
            .across_realms();
        assert!(still_running.previous_secret.is_some());
    }

    async fn end_the_grace_period(schema: &TestSchema, client: &Scoped<Client>) {
        sqlx::query(
            "UPDATE clients SET previous_secret_expires_at = (now() AT TIME ZONE 'UTC') - interval '1 minute' WHERE id = $1",
        )
        .bind(client.get().id)
        .execute(&schema.pool)
        .await
        .expect("end the grace period");
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn revoking_leaves_an_ended_grace_period_to_the_sweep() {
        let (repository, schema, client) = seeded("backend").await;
        repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate");
        end_the_grace_period(&schema, &client).await;

        assert!(
            !repository
                .revoke_previous_secret(&client)
                .await
                .expect("revoke")
        );
        let expired = repository
            .expire_previous_secrets(Utc::now())
            .await
            .expect("expire");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].client_id, client.get().id);
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn rotating_over_an_ended_grace_period_reports_its_expiry() {
        let (repository, schema, client) = seeded("backend").await;
        repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("first rotation");
        end_the_grace_period(&schema, &client).await;

        let rotation = repository
            .rotate_secret(
                &client,
                "third-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("second rotation");

        let expired = rotation
            .replaced_expired
            .expect("the ended secret is reported");
        assert_eq!(expired.client_id, client.get().id);
        assert_eq!(expired.realm_id, schema.realm.id());
        assert!(expired.expired_at < Utc::now());

        let again = repository
            .rotate_secret(&client, "fourth-secret".to_string(), None)
            .await
            .expect("third rotation");
        assert!(again.replaced_expired.is_none());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn a_previous_secret_that_cannot_be_opened_does_not_lock_the_client_out() {
        let (repository, schema, client) = seeded("backend").await;
        repository
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate");
        sqlx::query("UPDATE clients SET previous_secret = 'fk1:kid-gone:AAAA' WHERE id = $1")
            .bind(client.get().id)
            .execute(&schema.pool)
            .await
            .expect("seal the previous secret under a lost key");

        let read = repository
            .get_by_client_id("backend".to_string(), schema.realm.id())
            .await
            .expect("the client is still read")
            .across_realms();

        assert_eq!(read.secret_str(), Some("second-secret"));
        assert!(read.previous_secret.is_none());
    }

    #[tokio::test]
    #[ignore = "requires PostgreSQL — run with: cargo test -p ferriskey-core -- --ignored"]
    async fn the_current_secret_and_the_previous_one_need_the_key_to_be_read() {
        let (_, schema, client) = seeded("backend").await;
        PostgresClientRepository::with_cipher(schema.db.clone(), sealing_cipher())
            .rotate_secret(
                &client,
                "second-secret".to_string(),
                Some(Duration::hours(1)),
            )
            .await
            .expect("rotate");

        let result = PostgresClientRepository::new(schema.db)
            .get_by_client_id("backend".to_string(), schema.realm.id())
            .await;

        assert!(result.is_err(), "{result:?}");
    }
}
