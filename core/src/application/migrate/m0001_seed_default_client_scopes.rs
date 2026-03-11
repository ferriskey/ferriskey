use std::collections::HashSet;

use ferriskey_aegis::{
    ports::{ClientScopeMappingRepository, ClientScopeRepository, ProtocolMapperRepository},
    value_objects::{CreateClientScopeRequest, CreateProtocolMapperRequest},
};
use ferriskey_migrate::ports::{Migration, MigrationFuture};
use serde_json::json;
use uuid::Uuid;

use crate::domain::{client::ports::ClientRepository, realm::ports::RealmRepository};

use super::context::MigrationContext;

/// V0_4_0 — Seed default OIDC client scopes for all existing realms.
///
/// For every realm:
///   1. Creates each standard scope with its protocol mappers if they don't
///      already exist (checked by name per realm — idempotent).
///   2. Assigns every scope as default or optional to all clients in the realm
///      that don't already have the mapping.
///
/// The scope/mapper definitions mirror `RealmServiceImpl::seed_default_scopes_for_client`.
pub struct SeedDefaultClientScopes;

type ScopeTuple = (
    &'static str,                                         // scope name
    bool,                                                 // is_default
    Vec<(&'static str, &'static str, serde_json::Value)>, // (mapper_name, mapper_type, config)
);

fn default_scopes() -> Vec<ScopeTuple> {
    vec![
        ("openid", true, vec![]),
        (
            "profile",
            true,
            vec![
                (
                    "given_name",
                    "oidc-usermodel-property-mapper",
                    json!({
                        "user.attribute": "firstName",
                        "claim.name": "given_name",
                        "access.token.claim": "true",
                        "id.token.claim": "true"
                    }),
                ),
                (
                    "family_name",
                    "oidc-usermodel-property-mapper",
                    json!({
                        "user.attribute": "lastName",
                        "claim.name": "family_name",
                        "access.token.claim": "true",
                        "id.token.claim": "true"
                    }),
                ),
                (
                    "preferred_username",
                    "oidc-usermodel-property-mapper",
                    json!({
                        "user.attribute": "username",
                        "claim.name": "preferred_username",
                        "access.token.claim": "true",
                        "id.token.claim": "true"
                    }),
                ),
            ],
        ),
        (
            "email",
            true,
            vec![
                (
                    "email",
                    "oidc-usermodel-property-mapper",
                    json!({
                        "user.attribute": "email",
                        "claim.name": "email",
                        "access.token.claim": "true",
                        "id.token.claim": "true"
                    }),
                ),
                (
                    "email_verified",
                    "oidc-usermodel-property-mapper",
                    json!({
                        "user.attribute": "emailVerified",
                        "claim.name": "email_verified",
                        "jsonType.label": "boolean",
                        "access.token.claim": "true",
                        "id.token.claim": "true"
                    }),
                ),
            ],
        ),
        (
            "roles",
            true,
            vec![(
                "realm_access",
                "oidc-usermodel-realm-role-mapper",
                json!({
                    "claim.name": "realm_access.roles",
                    "access.token.claim": "true",
                    "id.token.claim": "true"
                }),
            )],
        ),
        ("offline_access", false, vec![]),
        (
            "phone",
            false,
            vec![(
                "phone_number",
                "oidc-usermodel-attribute-mapper",
                json!({
                    "user.attribute": "phone_number",
                    "claim.name": "phone_number",
                    "access.token.claim": "true",
                    "id.token.claim": "true"
                }),
            )],
        ),
        (
            "address",
            false,
            vec![(
                "address",
                "oidc-usermodel-attribute-mapper",
                json!({
                    "user.attribute": "address",
                    "claim.name": "address",
                    "access.token.claim": "true",
                    "id.token.claim": "true"
                }),
            )],
        ),
    ]
}

impl<R, C, CS, PM, CSM> Migration<MigrationContext<R, C, CS, PM, CSM>> for SeedDefaultClientScopes
where
    R: RealmRepository + 'static,
    C: ClientRepository + 'static,
    CS: ClientScopeRepository + 'static,
    PM: ProtocolMapperRepository + 'static,
    CSM: ClientScopeMappingRepository + 'static,
{
    fn version(&self) -> u64 {
        1
    }

    fn name(&self) -> &str {
        "v0_4_0_seed_default_client_scopes"
    }

    fn up<'a>(&'a self, ctx: &'a MigrationContext<R, C, CS, PM, CSM>) -> MigrationFuture<'a> {
        Box::pin(async move {
            let realms = ctx.realm_repository.fetch_realm().await?;

            for realm in realms {
                tracing::info!(realm.id = ?realm.id, realm.name = %realm.name, "seeding default scopes");

                // 1. Ensure each scope and its mappers exist, collect their ids.
                let mut scope_ids: Vec<(Uuid, bool)> = Vec::new();

                for (name, is_default, mappers) in default_scopes() {
                    let scope = match ctx
                        .client_scope_repository
                        .find_by_name(name.to_string(), realm.id)
                        .await?
                    {
                        Some(existing) => {
                            tracing::debug!(
                                scope = name,
                                "scope already exists, skipping creation"
                            );
                            existing
                        }
                        None => {
                            let created = ctx
                                .client_scope_repository
                                .create(CreateClientScopeRequest {
                                    realm_id: realm.id,
                                    name: name.to_string(),
                                    description: None,
                                    protocol: "openid-connect".to_string(),
                                    is_default,
                                })
                                .await?;
                            tracing::info!(scope = name, "created scope");
                            created
                        }
                    };

                    // Ensure every mapper exists on the scope.
                    let existing_mappers = ctx
                        .protocol_mapper_repository
                        .get_by_scope_id(scope.id)
                        .await?;

                    let existing_names: HashSet<&str> =
                        existing_mappers.iter().map(|m| m.name.as_str()).collect();

                    for (mapper_name, mapper_type, config) in mappers {
                        if existing_names.contains(mapper_name) {
                            tracing::debug!(
                                mapper = mapper_name,
                                "mapper already exists, skipping"
                            );
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

                        tracing::info!(scope = name, mapper = mapper_name, "created mapper");
                    }

                    scope_ids.push((scope.id, is_default));
                }

                // 2. Assign all scopes to every client in the realm.
                let clients = ctx.client_repository.get_by_realm_id(realm.id).await?;

                for client in clients {
                    let existing_mappings = ctx
                        .scope_mapping_repository
                        .get_client_scopes(client.id)
                        .await?;

                    let already_assigned: HashSet<Uuid> =
                        existing_mappings.iter().map(|m| m.scope_id).collect();

                    for (scope_id, is_default) in &scope_ids {
                        if already_assigned.contains(scope_id) {
                            continue;
                        }

                        ctx.scope_mapping_repository
                            .assign_scope_to_client(client.id, *scope_id, *is_default, !is_default)
                            .await?;

                        tracing::info!(
                            client.id = %client.id,
                            scope.id = %scope_id,
                            is_default,
                            "assigned scope to client"
                        );
                    }
                }
            }

            Ok(())
        })
    }
}
