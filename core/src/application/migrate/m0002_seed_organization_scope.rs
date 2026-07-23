use std::collections::HashSet;

use ferriskey_aegis::{
    ports::{ClientScopeMappingRepository, ClientScopeRepository, ProtocolMapperRepository},
    value_objects::{CreateClientScopeRequest, CreateProtocolMapperRequest},
};
use ferriskey_migrate::ports::{Migration, MigrationFuture};
use serde_json::json;

use crate::domain::{client::ports::ClientRepository, realm::ports::RealmRepository};

use super::context::MigrationContext;

/// Seed the default `organization` client scope for all existing realms.
///
/// The scope carries two protocol mappers that both write into the `organizations` claim
/// (they deep-merge): the organization-membership mapper (stable `id`/`name`/`alias`) and the
/// organization-role mapper (`roles` + `clients.<id>.roles` scoped to each org). It is a *default*
/// scope, so it applies to every client without being explicitly requested.
///
/// Idempotent: scope/mappers are checked by name per realm, assignments by scope id per client.
/// Mirrors the `organization` entry in `RealmServiceImpl::seed_default_scopes_for_client`.
pub struct SeedOrganizationScope;

/// `(mapper_name, mapper_type, config)` for the two org mappers on the `organization` scope.
/// Shared with the realm-creation seed to keep the two provisioning paths in sync.
pub fn organization_scope_mappers() -> Vec<(&'static str, &'static str, serde_json::Value)> {
    vec![
        (
            "organizations",
            "oidc-organization-membership-mapper",
            json!({
                "claim.name": "organizations",
                "access.token.claim": "true",
                "id.token.claim": "true"
            }),
        ),
        (
            "organization_roles",
            "oidc-organization-role-mapper",
            json!({
                "claim.name": "organizations",
                "access.token.claim": "true",
                "id.token.claim": "true"
            }),
        ),
    ]
}

impl<R, C, CS, PM, CSM> Migration<MigrationContext<R, C, CS, PM, CSM>> for SeedOrganizationScope
where
    R: RealmRepository + 'static,
    C: ClientRepository + 'static,
    CS: ClientScopeRepository + 'static,
    PM: ProtocolMapperRepository + 'static,
    CSM: ClientScopeMappingRepository + 'static,
{
    fn version(&self) -> u64 {
        2
    }

    fn name(&self) -> &str {
        "v0_8_0_seed_organization_scope"
    }

    fn up<'a>(&'a self, ctx: &'a MigrationContext<R, C, CS, PM, CSM>) -> MigrationFuture<'a> {
        Box::pin(async move {
            let realms = ctx.realm_repository.fetch_realm().await?;

            for realm in realms {
                tracing::info!(realm.id = ?realm.id, realm.name = %realm.name, "seeding organization scope");

                // 1. Ensure the scope and its mappers exist.
                let scope = match ctx
                    .client_scope_repository
                    .find_by_name("organization".to_string(), realm.id)
                    .await?
                {
                    Some(existing) => existing,
                    None => {
                        let created = ctx
                            .client_scope_repository
                            .create(CreateClientScopeRequest {
                                realm_id: realm.id,
                                name: "organization".to_string(),
                                description: None,
                                protocol: "openid-connect".to_string(),
                                is_default: true,
                            })
                            .await?;
                        tracing::info!(scope = "organization", "created scope");
                        created
                    }
                };

                let existing_mappers = ctx
                    .protocol_mapper_repository
                    .get_by_scope_id(scope.id)
                    .await?;
                let existing_names: HashSet<&str> =
                    existing_mappers.iter().map(|m| m.name.as_str()).collect();

                for (mapper_name, mapper_type, config) in organization_scope_mappers() {
                    if existing_names.contains(mapper_name) {
                        continue;
                    }
                    ctx.protocol_mapper_repository
                        .create(CreateProtocolMapperRequest {
                            client_scope_id: scope.id,
                            name: mapper_name.to_string(),
                            mapper_type: mapper_type.to_string(),
                            config,
                        })
                        .await?;
                    tracing::info!(
                        scope = "organization",
                        mapper = mapper_name,
                        "created mapper"
                    );
                }

                // 2. Assign the scope (as default) to every client that lacks it.
                let clients = ctx.client_repository.get_by_realm_id(realm.id).await?;
                for client in clients {
                    let existing_mappings = ctx
                        .scope_mapping_repository
                        .get_client_scopes(client.id)
                        .await?;
                    let already_assigned = existing_mappings.iter().any(|m| m.scope_id == scope.id);
                    if already_assigned {
                        continue;
                    }

                    ctx.scope_mapping_repository
                        .assign_scope_to_client(client.id, scope.id, true, false)
                        .await?;
                    tracing::info!(
                        client.id = %client.id,
                        scope.id = %scope.id,
                        "assigned organization scope to client"
                    );
                }
            }

            Ok(())
        })
    }
}
