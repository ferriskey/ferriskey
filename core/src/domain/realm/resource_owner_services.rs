use std::collections::HashSet;
use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::client::entities::{Client, ClientRegistrationSource};
use crate::domain::client::ports::ClientRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::{FerriskeyPolicy, ensure_policy};
use crate::domain::realm::entities::{RealmScope, ResourceOwner};
use crate::domain::realm::ports::{
    ListResourceOwnersInput, RealmPolicy, RealmRepository, ReplaceResourceOwnersInput,
    ResourceOwnerRepository, ResourceOwnerService,
};
use crate::domain::user::ports::{UserRepository, UserRoleRepository};

fn invalid(detail: impl Into<String>) -> CoreError {
    CoreError::InvalidResourceOwner(detail.into())
}

pub(crate) fn ensure_owner_uris(
    allowed_resources: &[String],
    owners: &[ResourceOwner],
) -> Result<(), CoreError> {
    let mut seen = HashSet::new();

    for owner in owners {
        if !allowed_resources.contains(&owner.uri) {
            return Err(invalid(format!(
                "{} is not in the realm's allowed resources",
                owner.uri
            )));
        }
        if !seen.insert(owner.uri.as_str()) {
            return Err(invalid(format!("{} is listed more than once", owner.uri)));
        }
    }

    Ok(())
}

pub(crate) fn ensure_owner_client(client: &Client) -> Result<(), CoreError> {
    if client.public_client {
        return Err(invalid(format!(
            "client {} is public and cannot own a resource",
            client.client_id
        )));
    }

    if client.registration_source != ClientRegistrationSource::Admin {
        return Err(invalid(format!(
            "client {} was registered dynamically and cannot own a resource",
            client.client_id
        )));
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct ResourceOwnerServiceImpl<R, C, O, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    O: ResourceOwnerRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) resource_owner_repository: Arc<O>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, C, O, U, UR> ResourceOwnerServiceImpl<R, C, O, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    O: ResourceOwnerRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        resource_owner_repository: Arc<O>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            resource_owner_repository,
            policy,
        }
    }

    async fn authorized_scope(
        &self,
        identity: &Identity,
        realm_name: &str,
    ) -> Result<RealmScope, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), realm_name).await?;

        ensure_policy(
            self.policy.can_update_realm(identity, scope.realm()).await,
            "insufficient permissions",
        )?;

        Ok(scope)
    }

    async fn ensure_owner_clients(
        &self,
        scope: &RealmScope,
        owners: &[ResourceOwner],
    ) -> Result<(), CoreError> {
        let mut checked = HashSet::<Uuid>::new();

        for owner in owners {
            if !checked.insert(owner.client_id) {
                continue;
            }

            let client = self
                .client_repository
                .get_by_id(scope.id(), owner.client_id)
                .await
                .map_err(|err| match err {
                    CoreError::NotFound => invalid(format!(
                        "client {} does not exist in this realm",
                        owner.client_id
                    )),
                    other => other,
                })?
                .in_realm(scope)
                .map_err(|_| {
                    invalid(format!(
                        "client {} does not exist in this realm",
                        owner.client_id
                    ))
                })?
                .into_inner();

            ensure_owner_client(&client)?;
        }

        Ok(())
    }
}

impl<R, C, O, U, UR> ResourceOwnerService for ResourceOwnerServiceImpl<R, C, O, U, UR>
where
    R: RealmRepository,
    C: ClientRepository,
    O: ResourceOwnerRepository,
    U: UserRepository,
    UR: UserRoleRepository,
{
    #[instrument(
        skip(self, identity, input),
        fields(identity.id = %identity.id(), realm.name = %input.realm_name)
    )]
    async fn list_resource_owners(
        &self,
        identity: Identity,
        input: ListResourceOwnersInput,
    ) -> Result<Vec<ResourceOwner>, CoreError> {
        let scope = self.authorized_scope(&identity, &input.realm_name).await?;

        self.resource_owner_repository
            .list_by_realm(scope.id())
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(identity.id = %identity.id(), realm.name = %input.realm_name)
    )]
    async fn replace_resource_owners(
        &self,
        identity: Identity,
        input: ReplaceResourceOwnersInput,
    ) -> Result<Vec<ResourceOwner>, CoreError> {
        let scope = self.authorized_scope(&identity, &input.realm_name).await?;

        let allowed_resources = self
            .realm_repository
            .get_realm_settings(scope.id())
            .await?
            .ok_or(CoreError::NotFound)?
            .allowed_resources;

        ensure_owner_uris(&allowed_resources, &input.owners)?;
        self.ensure_owner_clients(&scope, &input.owners).await?;

        self.resource_owner_repository
            .replace_for_realm(scope.id(), &input.owners)
            .await?;

        self.resource_owner_repository
            .list_by_realm(scope.id())
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm::entities::RealmId;

    fn owner(uri: &str) -> ResourceOwner {
        ResourceOwner {
            uri: uri.to_string(),
            client_id: Uuid::from_u128(1),
        }
    }

    fn client(public: bool, source: ClientRegistrationSource) -> Client {
        let mut client = Client::from_realm_and_client_id(RealmId::default(), "owner".to_string());
        client.public_client = public;
        client.registration_source = source;
        client
    }

    #[test]
    fn owners_must_target_allowed_resources() {
        let allowed = vec!["https://mcp.example.com".to_string()];

        assert!(ensure_owner_uris(&allowed, &[owner("https://mcp.example.com")]).is_ok());
        assert!(matches!(
            ensure_owner_uris(&allowed, &[owner("https://other.example.com")]),
            Err(CoreError::InvalidResourceOwner(_))
        ));
    }

    #[test]
    fn a_uri_has_a_single_owner() {
        let allowed = vec!["https://mcp.example.com".to_string()];
        let owners = [
            owner("https://mcp.example.com"),
            owner("https://mcp.example.com"),
        ];

        assert!(matches!(
            ensure_owner_uris(&allowed, &owners),
            Err(CoreError::InvalidResourceOwner(_))
        ));
    }

    #[test]
    fn only_confidential_admin_clients_own_resources() {
        assert!(ensure_owner_client(&client(false, ClientRegistrationSource::Admin)).is_ok());
        assert!(ensure_owner_client(&client(true, ClientRegistrationSource::Admin)).is_err());
        assert!(ensure_owner_client(&client(false, ClientRegistrationSource::Dynamic)).is_err());
        assert!(
            ensure_owner_client(&client(false, ClientRegistrationSource::MetadataDocument))
                .is_err()
        );
    }
}
