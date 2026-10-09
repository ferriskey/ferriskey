use std::collections::HashSet;

use uuid::Uuid;

use crate::domain::aegis::entities::ScopeType;
use crate::domain::aegis::ports::{ClientScopeMappingRepository, ClientScopeRepository};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;

pub const STANDARD_SCOPES: [&str; 4] = ["openid", "profile", "email", "roles"];

pub async fn registrable_scopes<CS>(
    scope_repository: &CS,
    realm_id: RealmId,
) -> Result<HashSet<String>, CoreError>
where
    CS: ClientScopeRepository,
{
    let realm_scopes = scope_repository.find_by_realm_id(realm_id).await?;

    Ok(realm_scopes
        .into_iter()
        .filter(|scope| {
            scope.default_scope_type == ScopeType::Default
                || STANDARD_SCOPES.contains(&scope.name.as_str())
        })
        .map(|scope| scope.name)
        .collect())
}

pub async fn assign_default_scopes<CS, CSM>(
    scope_repository: &CS,
    mapping_repository: &CSM,
    realm_id: RealmId,
    client_id: Uuid,
    extra: &[String],
) -> Result<(), CoreError>
where
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    let realm_scopes = scope_repository.find_by_realm_id(realm_id).await?;

    for scope in realm_scopes.into_iter().filter(|scope| {
        STANDARD_SCOPES.contains(&scope.name.as_str()) || extra.contains(&scope.name)
    }) {
        mapping_repository
            .assign_scope_to_client(client_id, scope.id, true, false)
            .await?;
    }

    Ok(())
}
