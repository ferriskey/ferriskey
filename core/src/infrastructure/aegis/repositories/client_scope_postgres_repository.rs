use std::collections::HashMap;

use chrono::Utc;
use ferriskey_aegis::entities::ScopeType;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, Select,
};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::aegis::entities::{
    ClientScope, ClientScopeFilter, ClientScopeSortField, ProtocolMapper,
};
use crate::domain::aegis::ports::ClientScopeRepository;
use crate::domain::aegis::value_objects::{CreateClientScopeRequest, UpdateClientScopeRequest};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_uuid_v7;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::realm::entities::{RealmId, RealmScope, Scoped, Unscoped};
use crate::entity::{client_scope_mappings, client_scope_protocol_mappers, client_scopes};
use crate::infrastructure::pagination::{SortColumn, contains, paginate, within_naive};

impl SortColumn<client_scopes::Entity> for ClientScopeSortField {
    fn column(&self) -> client_scopes::Column {
        match self {
            ClientScopeSortField::Name => client_scopes::Column::Name,
            ClientScopeSortField::CreatedAt => client_scopes::Column::CreatedAt,
            ClientScopeSortField::UpdatedAt => client_scopes::Column::UpdatedAt,
        }
    }
}

fn listing_select(realm_id: Uuid, filter: &ClientScopeFilter) -> Select<client_scopes::Entity> {
    client_scopes::Entity::find()
        .filter(client_scopes::Column::RealmId.eq(realm_id))
        .filter(within_naive(
            client_scopes::Column::CreatedAt,
            &filter.created,
        ))
        .apply_if(filter.name.as_deref(), |select, value| {
            select.filter(contains(client_scopes::Column::Name, value))
        })
        .apply_if(filter.description.as_deref(), |select, value| {
            select.filter(contains(client_scopes::Column::Description, value))
        })
        .apply_if(filter.search.as_deref(), |select, value| {
            select.filter(
                Condition::any()
                    .add(contains(client_scopes::Column::Name, value))
                    .add(contains(client_scopes::Column::Description, value)),
            )
        })
        .apply_if(filter.protocol.as_deref(), |select, value| {
            select.filter(client_scopes::Column::Protocol.eq(value))
        })
        .apply_if(filter.default_scope_type.as_ref(), |select, value| {
            select.filter(client_scopes::Column::DefaultScopeType.eq(value.as_str()))
        })
        .apply_if(filter.has_protocol_mappers, |select, present| {
            let with_mappers = client_scope_protocol_mappers::Entity::find()
                .select_only()
                .column(client_scope_protocol_mappers::Column::ClientScopeId)
                .into_query();
            select.filter(if present {
                client_scopes::Column::Id.in_subquery(with_mappers)
            } else {
                client_scopes::Column::Id.not_in_subquery(with_mappers)
            })
        })
        .apply_if(filter.not_assigned_to_client, |select, client_id| {
            let assigned = client_scope_mappings::Entity::find()
                .select_only()
                .column(client_scope_mappings::Column::ClientScopeId)
                .filter(client_scope_mappings::Column::ClientId.eq(client_id))
                .into_query();
            select.filter(client_scopes::Column::Id.not_in_subquery(assigned))
        })
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PostgresClientScopeRepository {
    pub db: DatabaseConnection,
}

impl PostgresClientScopeRepository {
    #[allow(dead_code)]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl ClientScopeRepository for PostgresClientScopeRepository {
    #[instrument(skip(self, payload), fields(realm.id = ?payload.realm_id))]
    async fn create(&self, payload: CreateClientScopeRequest) -> Result<ClientScope, CoreError> {
        let now = Utc::now().naive_utc();

        let active_model = client_scopes::ActiveModel {
            id: Set(generate_uuid_v7()),
            realm_id: Set(payload.realm_id.into()),
            name: Set(payload.name),
            description: Set(payload.description),
            protocol: Set(payload.protocol),
            default_scope_type: Set(if payload.is_default {
                ScopeType::Default.to_string()
            } else {
                ScopeType::Optional.to_string()
            }),
            dynamic_registration_allowed: Set(payload.dynamic_registration_allowed),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active_model.insert(&self.db).await.map_err(|e| {
            tracing::error!("Failed to insert client scope: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    #[instrument(skip(self))]
    async fn get_by_id(
        &self,
        realm_id: RealmId,
        id: Uuid,
    ) -> Result<Option<Unscoped<ClientScope>>, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.eq(id))
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get client scope by id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(ClientScope::from).map(Unscoped::new))
    }

    #[instrument(skip(self))]
    async fn find_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<ClientScope>, CoreError> {
        let models = client_scopes::Entity::find()
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .all(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scopes by realm id: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(models.into_iter().map(ClientScope::from).collect())
    }

    #[instrument(skip(self, request), fields(realm.id = ?scope.id()))]
    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<ClientScopeFilter, ClientScopeSortField>,
    ) -> Result<Page<ClientScope>, CoreError> {
        let select = listing_select(scope.id().into(), &request.filter);
        let (models, total) = paginate(&self.db, select, request).await.map_err(|e| {
            error!("error listing client scopes: {:?}", e);
            CoreError::InternalServerError
        })?;

        let scope_ids: Vec<Uuid> = models.iter().map(|model| model.id).collect();
        let mut mappers: HashMap<Uuid, Vec<ProtocolMapper>> = HashMap::new();
        if !scope_ids.is_empty() {
            let rows = client_scope_protocol_mappers::Entity::find()
                .filter(client_scope_protocol_mappers::Column::ClientScopeId.is_in(scope_ids))
                .order_by_asc(client_scope_protocol_mappers::Column::CreatedAt)
                .order_by_asc(client_scope_protocol_mappers::Column::Id)
                .all(&self.db)
                .await
                .map_err(|e| {
                    error!(
                        "error loading protocol mappers of listed client scopes: {:?}",
                        e
                    );
                    CoreError::InternalServerError
                })?;
            for row in rows {
                mappers
                    .entry(row.client_scope_id)
                    .or_default()
                    .push(ProtocolMapper::from(row));
            }
        }

        let client_scopes = models
            .into_iter()
            .map(|model| {
                let id = model.id;
                let mut client_scope = ClientScope::from(model);
                client_scope.protocol_mappers = Some(mappers.remove(&id).unwrap_or_default());
                client_scope
            })
            .collect();

        Ok(Page::new(client_scopes, total, request.page, request.limit))
    }

    #[instrument(skip(self))]
    async fn find_by_name(
        &self,
        name: String,
        realm_id: RealmId,
    ) -> Result<Option<Unscoped<ClientScope>>, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Name.eq(name))
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scope by name: {}", e);
                CoreError::InternalServerError
            })?;

        Ok(model.map(ClientScope::from).map(Unscoped::new))
    }

    #[instrument(skip(self, payload), fields(scope.id = %client_scope.get().id))]
    async fn update_by_id(
        &self,
        client_scope: &Scoped<ClientScope>,
        payload: UpdateClientScopeRequest,
    ) -> Result<ClientScope, CoreError> {
        let model = client_scopes::Entity::find()
            .filter(client_scopes::Column::Id.eq(client_scope.get().id))
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(client_scope.get().realm_id.into()))
            .one(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to find client scope for update: {}", e);
                CoreError::InternalServerError
            })?
            .ok_or(CoreError::NotFound)?;

        let mut active: client_scopes::ActiveModel = model.into();

        active.name = match payload.name {
            Some(v) => Set(v),
            None => active.name,
        };
        active.description = match payload.description {
            Some(v) => Set(Some(v)),
            None => active.description,
        };
        active.protocol = match payload.protocol {
            Some(v) => Set(v),
            None => active.protocol,
        };
        active.default_scope_type = match payload.is_default {
            Some(v) => Set(if v {
                ScopeType::Default.to_string()
            } else {
                ScopeType::Optional.to_string()
            }),
            None => active.default_scope_type,
        };
        active.dynamic_registration_allowed = match payload.dynamic_registration_allowed {
            Some(v) => Set(v),
            None => active.dynamic_registration_allowed,
        };
        active.updated_at = Set(Utc::now().naive_utc());

        let model = active.update(&self.db).await.map_err(|e| {
            tracing::error!("Failed to update client scope: {}", e);
            CoreError::InternalServerError
        })?;

        Ok(model.into())
    }

    #[instrument(skip(self), fields(scope.id = %client_scope.get().id))]
    async fn delete_by_id(&self, client_scope: &Scoped<ClientScope>) -> Result<(), CoreError> {
        let result = client_scopes::Entity::delete_many()
            .filter(client_scopes::Column::Id.eq(client_scope.get().id))
            .filter(client_scopes::Column::RealmId.eq::<Uuid>(client_scope.get().realm_id.into()))
            .exec(&self.db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to delete client scope: {}", e);
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
    use chrono::{TimeZone, Utc};
    use sea_orm::{DbBackend, QueryTrait};
    use uuid::Uuid;

    use super::listing_select;
    use crate::domain::aegis::entities::{ClientScopeFilter, ScopeType};
    use crate::domain::common::pagination::DateRange;

    fn sql(filter: &ClientScopeFilter) -> String {
        listing_select(Uuid::nil(), filter)
            .build(DbBackend::Postgres)
            .to_string()
    }

    #[test]
    fn an_unbounded_created_range_adds_no_predicate() {
        let sql = sql(&ClientScopeFilter::default());
        assert!(!sql.contains(r#""client_scopes"."created_at" >"#), "{sql}");
        assert!(!sql.contains(r#""client_scopes"."created_at" <"#), "{sql}");
    }

    #[test]
    fn created_range_bounds_the_creation_date() {
        let sql = sql(&ClientScopeFilter {
            created: DateRange::new(
                Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).single(),
                Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).single(),
            ),
            ..ClientScopeFilter::default()
        });
        assert!(
            sql.contains(r#""client_scopes"."created_at" >= '2026-01-01 00:00:00.000000'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""client_scopes"."created_at" < '2026-02-01 00:00:00.000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn listing_is_always_bound_to_the_realm() {
        let sql = sql(&ClientScopeFilter::default());
        assert!(
            sql.contains(r#""client_scopes"."realm_id" = '00000000-0000-0000-0000-000000000000'"#),
            "{sql}"
        );
    }

    #[test]
    fn text_filters_are_escaped_contains_matches() {
        let sql = sql(&ClientScopeFilter {
            name: Some("a%".to_string()),
            description: Some("b".to_string()),
            ..ClientScopeFilter::default()
        });
        assert!(
            sql.contains(r#""client_scopes"."name" ILIKE E'%a\\%%'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""client_scopes"."description" ILIKE '%b%'"#),
            "{sql}"
        );
    }

    #[test]
    fn search_matches_the_name_or_the_description() {
        let sql = sql(&ClientScopeFilter {
            search: Some("pro".to_string()),
            ..ClientScopeFilter::default()
        });
        assert!(
            sql.contains(
                r#"(("client_scopes"."name" ILIKE '%pro%') OR ("client_scopes"."description" ILIKE '%pro%'))"#
            ),
            "{sql}"
        );
    }

    #[test]
    fn exact_filters_are_equalities() {
        let sql = sql(&ClientScopeFilter {
            protocol: Some("saml".to_string()),
            default_scope_type: Some(ScopeType::Optional),
            ..ClientScopeFilter::default()
        });
        assert!(
            sql.contains(r#""client_scopes"."protocol" = 'saml'"#),
            "{sql}"
        );
        assert!(
            sql.contains(r#""client_scopes"."default_scope_type" = 'OPTIONAL'"#),
            "{sql}"
        );
    }

    #[test]
    fn mapper_presence_is_a_subquery_on_protocol_mappers() {
        let with = sql(&ClientScopeFilter {
            has_protocol_mappers: Some(true),
            ..ClientScopeFilter::default()
        });
        assert!(
            with.contains(
                r#""client_scopes"."id" IN (SELECT "client_scope_protocol_mappers"."client_scope_id" FROM "client_scope_protocol_mappers")"#
            ),
            "{with}"
        );
        let without = sql(&ClientScopeFilter {
            has_protocol_mappers: Some(false),
            ..ClientScopeFilter::default()
        });
        assert!(
            without.contains(
                r#""client_scopes"."id" NOT IN (SELECT "client_scope_protocol_mappers"."client_scope_id" FROM "client_scope_protocol_mappers")"#
            ),
            "{without}"
        );
    }

    #[test]
    fn not_assigned_to_client_excludes_the_mapped_scopes_of_that_client() {
        let client_id = Uuid::from_u128(7);
        let sql = sql(&ClientScopeFilter {
            not_assigned_to_client: Some(client_id),
            ..ClientScopeFilter::default()
        });
        assert!(
            sql.contains(&format!(
                r#""client_scopes"."id" NOT IN (SELECT "client_scope_mappings"."client_scope_id" FROM "client_scope_mappings" WHERE "client_scope_mappings"."client_id" = '{client_id}')"#
            )),
            "{sql}"
        );
    }
}
