use std::collections::HashSet;

use uuid::Uuid;

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
            scope.dynamic_registration_allowed || STANDARD_SCOPES.contains(&scope.name.as_str())
        })
        .map(|scope| scope.name)
        .collect())
}

pub async fn assign_scopes<CS, CSM>(
    scope_repository: &CS,
    mapping_repository: &CSM,
    realm_id: RealmId,
    client_id: Uuid,
    requested: Option<&[String]>,
) -> Result<(), CoreError>
where
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    let realm_scopes = scope_repository.find_by_realm_id(realm_id).await?;

    for scope in realm_scopes {
        let is_default = STANDARD_SCOPES.contains(&scope.name.as_str())
            || requested.is_some_and(|requested| requested.contains(&scope.name));
        let is_optional = !is_default && requested.is_none() && scope.dynamic_registration_allowed;

        if is_default || is_optional {
            mapping_repository
                .assign_scope_to_client(client_id, scope.id, is_default, is_optional)
                .await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aegis::entities::ClientScope;
    use crate::domain::aegis::ports::{
        MockClientScopeMappingRepository, MockClientScopeRepository,
    };

    fn scope(realm_id: RealmId, name: &str, dynamic: bool) -> ClientScope {
        let mut scope = ClientScope::new(
            realm_id,
            name.to_string(),
            None,
            "openid-connect".to_string(),
        );
        scope.dynamic_registration_allowed = dynamic;
        scope
    }

    fn realm_scopes(realm_id: RealmId) -> Vec<ClientScope> {
        vec![
            scope(realm_id, "openid", false),
            scope(realm_id, "profile", false),
            scope(realm_id, "mcp:tools", true),
            scope(realm_id, "internal", false),
        ]
    }

    fn scope_repository(scopes: Vec<ClientScope>) -> MockClientScopeRepository {
        let mut repository = MockClientScopeRepository::new();
        repository.expect_find_by_realm_id().returning(move |_| {
            let scopes = scopes.clone();
            Box::pin(async move { Ok(scopes) })
        });
        repository
    }

    #[tokio::test]
    async fn only_standard_and_flagged_scopes_are_registrable() {
        let realm_id = RealmId::default();
        let registrable = registrable_scopes(&scope_repository(realm_scopes(realm_id)), realm_id)
            .await
            .expect("scopes");

        let mut names: Vec<_> = registrable.into_iter().collect();
        names.sort();
        assert_eq!(names, vec!["mcp:tools", "openid", "profile"]);
    }

    async fn assigned(requested: Option<&[String]>) -> Vec<(String, bool, bool)> {
        let realm_id = RealmId::default();
        let fixtures = realm_scopes(realm_id);
        let names: std::collections::HashMap<Uuid, String> = fixtures
            .iter()
            .map(|scope| (scope.id, scope.name.clone()))
            .collect();
        let scopes = scope_repository(fixtures);
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = seen.clone();
        let mut mappings = MockClientScopeMappingRepository::new();
        mappings.expect_assign_scope_to_client().returning(
            move |client_id, scope_id, is_default, is_optional| {
                recorded
                    .lock()
                    .expect("lock")
                    .push((scope_id, is_default, is_optional));
                Box::pin(async move {
                    Ok(crate::domain::aegis::entities::ClientScopeMapping {
                        client_id,
                        scope_id,
                        default_scope_type: if is_default {
                            crate::domain::aegis::entities::ScopeType::Default
                        } else {
                            crate::domain::aegis::entities::ScopeType::Optional
                        },
                    })
                })
            },
        );

        assign_scopes(&scopes, &mappings, realm_id, Uuid::new_v4(), requested)
            .await
            .expect("assigned");

        let mut out: Vec<_> = seen
            .lock()
            .expect("lock")
            .iter()
            .map(|(id, d, o)| (names[id].clone(), *d, *o))
            .collect();
        out.sort();
        out
    }

    #[tokio::test]
    async fn without_a_request_flagged_scopes_are_optional() {
        assert_eq!(
            assigned(None).await,
            vec![
                ("mcp:tools".to_string(), false, true),
                ("openid".to_string(), true, false),
                ("profile".to_string(), true, false),
            ]
        );
    }

    #[tokio::test]
    async fn requested_scopes_become_default_and_the_rest_is_not_offered() {
        let requested = vec!["mcp:tools".to_string()];

        assert_eq!(
            assigned(Some(&requested)).await,
            vec![
                ("mcp:tools".to_string(), true, false),
                ("openid".to_string(), true, false),
                ("profile".to_string(), true, false),
            ]
        );
    }
}
