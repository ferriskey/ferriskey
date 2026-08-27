use std::collections::HashMap;
use std::sync::Arc;

use tracing::{instrument, warn};
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::ensure_policy;
use crate::domain::realm::entities::Realm;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::seawatch::{
    EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType,
};
use crate::domain::user::entities::User;
use crate::domain::user::ports::{UserPolicy, UserRepository};

use crate::domain::abyss::identity_provider::broker::{
    IdentityProviderLink, IdentityProviderLinkRepository,
};
use crate::domain::abyss::identity_provider::value_objects::{
    CreateIdentityProviderRequest, UpdateIdentityProviderRequest,
};
use crate::domain::abyss::identity_provider::{
    CreateIdentityProviderInput, DeleteIdentityProviderInput, DeleteIdentityProviderLinkInput,
    GetIdentityProviderInput, IdentityProvider, IdentityProviderLinkView,
    ListIdentityProviderLinksInput, ListIdentityProvidersInput, UpdateIdentityProviderInput,
};
use crate::domain::abyss::identity_provider::{
    IdentityProviderPolicy, IdentityProviderRepository, IdentityProviderService,
};

/// Implementation of the IdentityProviderService trait
///
/// Provides business logic for managing identity providers,
/// including authorization checks and validation.
#[derive(Clone, Debug)]
pub struct IdentityProviderServiceImpl<R, P, RR, U, L, SE>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy + UserPolicy,
    RR: RealmRepository,
    U: UserRepository,
    L: IdentityProviderLinkRepository,
    SE: SecurityEventRepository,
{
    identity_provider_repository: Arc<R>,
    identity_provider_policy: Arc<P>,
    realm_repository: Arc<RR>,
    user_repository: Arc<U>,
    identity_provider_link_repository: Arc<L>,
    security_event_repository: Arc<SE>,
}

impl<R, P, RR, U, L, SE> IdentityProviderServiceImpl<R, P, RR, U, L, SE>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy + UserPolicy,
    RR: RealmRepository,
    U: UserRepository,
    L: IdentityProviderLinkRepository,
    SE: SecurityEventRepository,
{
    /// Creates a new IdentityProviderServiceImpl
    ///
    /// # Arguments
    /// * `identity_provider_repository` - The identity provider repository for data access
    /// * `identity_provider_policy` - The authorization policy for access control
    /// * `realm_repository` - The realm repository to resolve realm names
    pub fn new(
        identity_provider_repository: Arc<R>,
        identity_provider_policy: Arc<P>,
        realm_repository: Arc<RR>,
        user_repository: Arc<U>,
        identity_provider_link_repository: Arc<L>,
        security_event_repository: Arc<SE>,
    ) -> Self {
        Self {
            identity_provider_repository,
            identity_provider_policy,
            realm_repository,
            user_repository,
            identity_provider_link_repository,
            security_event_repository,
        }
    }

    async fn resolve_user_for_link_management(
        &self,
        identity: &Identity,
        realm_name: &str,
        user_id: Uuid,
    ) -> Result<(Realm, User), CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        ensure_policy(
            self.identity_provider_policy
                .can_update_user(identity, &realm)
                .await,
            "insufficient permissions to manage identity provider links",
        )?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if user.realm_id != realm.id {
            warn!(
                user_id = %user_id,
                user_realm_id = %Uuid::from(user.realm_id),
                request_realm_id = %Uuid::from(realm.id),
                "Refused cross-realm access to identity provider links"
            );
            return Err(CoreError::NotFound);
        }

        Ok((realm, user))
    }

    async fn load_links_in_realm(
        &self,
        realm: &Realm,
        user_id: Uuid,
    ) -> Result<Vec<(IdentityProviderLink, String)>, CoreError> {
        let aliases = self
            .identity_provider_repository
            .list_identity_providers_by_realm(realm.id, None)
            .await?
            .into_iter()
            .map(|provider| (provider.id.as_uuid(), provider.alias))
            .collect::<HashMap<_, _>>();

        let links = self
            .identity_provider_link_repository
            .get_by_user_id(user_id)
            .await?
            .into_iter()
            .filter_map(|link| {
                aliases
                    .get(&link.identity_provider_id.as_uuid())
                    .cloned()
                    .map(|alias| (link, alias))
            })
            .collect();

        Ok(links)
    }
}

impl<R, P, RR, U, L, SE> IdentityProviderService for IdentityProviderServiceImpl<R, P, RR, U, L, SE>
where
    R: IdentityProviderRepository,
    P: IdentityProviderPolicy + UserPolicy,
    RR: RealmRepository,
    U: UserRepository,
    L: IdentityProviderLinkRepository,
    SE: SecurityEventRepository,
{
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn create_identity_provider(
        &self,
        identity: Identity,
        input: CreateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_create_identity_provider(&identity, &realm)
                .await,
            "insufficient permissions to create identity provider",
        )?;

        // Check if alias already exists in realm
        let exists = self
            .identity_provider_repository
            .exists_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?;

        if exists {
            return Err(CoreError::ProviderNameAlreadyExists);
        }

        // Create the identity provider
        let request = CreateIdentityProviderRequest {
            realm_id: realm.id,
            alias: input.alias,
            provider_id: input.provider_id,
            enabled: input.enabled,
            display_name: input.display_name,
            first_broker_login_flow_alias: input.first_broker_login_flow_alias,
            post_broker_login_flow_alias: input.post_broker_login_flow_alias,
            store_token: input.store_token,
            add_read_token_role_on_create: input.add_read_token_role_on_create,
            trust_email: input.trust_email,
            link_only: input.link_only,
            config: input.config,
        };

        self.identity_provider_repository
            .create_identity_provider(request)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn get_identity_provider(
        &self,
        identity: Identity,
        input: GetIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_view_identity_provider(&identity, &realm)
                .await,
            "insufficient permissions to view identity provider",
        )?;

        Ok(provider)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
        )
    )]
    async fn list_identity_providers(
        &self,
        identity: Identity,
        input: ListIdentityProvidersInput,
    ) -> Result<Vec<IdentityProvider>, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get all identity providers for the realm
        let providers = self
            .identity_provider_repository
            .list_identity_providers_by_realm(realm.id, None)
            .await?;

        // Filter based on view permission
        let mut accessible_providers = Vec::new();
        for provider in providers {
            if self
                .identity_provider_policy
                .can_view_identity_provider(&identity, &realm)
                .await
                .unwrap_or(false)
            {
                accessible_providers.push(provider);
            }
        }

        Ok(accessible_providers)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn update_identity_provider(
        &self,
        identity: Identity,
        input: UpdateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_update_identity_provider(&identity, &realm)
                .await,
            "insufficient permissions to update identity provider",
        )?;

        // Update the identity provider
        let request = UpdateIdentityProviderRequest {
            enabled: input.enabled,
            display_name: input.display_name,
            first_broker_login_flow_alias: input.first_broker_login_flow_alias,
            post_broker_login_flow_alias: input.post_broker_login_flow_alias,
            store_token: input.store_token,
            add_read_token_role_on_create: input.add_read_token_role_on_create,
            trust_email: input.trust_email,
            link_only: input.link_only,
            config: input.config,
        };

        self.identity_provider_repository
            .update_identity_provider(provider.id.into(), request)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn delete_identity_provider(
        &self,
        identity: Identity,
        input: DeleteIdentityProviderInput,
    ) -> Result<(), CoreError> {
        // Resolve realm by name
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_delete_identity_provider(&identity, &realm)
                .await,
            "insufficient permissions to delete identity provider",
        )?;

        self.identity_provider_repository
            .delete_identity_provider(provider.id.into())
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            user.id = %input.user_id,
        )
    )]
    async fn list_identity_provider_links(
        &self,
        identity: Identity,
        input: ListIdentityProviderLinksInput,
    ) -> Result<Vec<IdentityProviderLinkView>, CoreError> {
        let (realm, user) = self
            .resolve_user_for_link_management(&identity, &input.realm_name, input.user_id)
            .await?;

        let links = self.load_links_in_realm(&realm, user.id).await?;

        Ok(links
            .into_iter()
            .map(|(link, alias)| IdentityProviderLinkView {
                id: link.id,
                identity_provider_id: link.identity_provider_id,
                identity_provider_alias: alias,
                identity_provider_user_id: link.identity_provider_user_id,
                created_at: link.created_at,
                updated_at: link.updated_at,
            })
            .collect())
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.name = %input.realm_name,
            user.id = %input.user_id,
            link.id = %input.link_id,
        )
    )]
    async fn delete_identity_provider_link(
        &self,
        identity: Identity,
        input: DeleteIdentityProviderLinkInput,
    ) -> Result<(), CoreError> {
        let (realm, user) = self
            .resolve_user_for_link_management(&identity, &input.realm_name, input.user_id)
            .await?;

        let (link, alias) = self
            .load_links_in_realm(&realm, user.id)
            .await?
            .into_iter()
            .find(|(link, _)| link.id == input.link_id)
            .ok_or(CoreError::NotFound)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    realm.id,
                    SecurityEventType::IdentityProviderLinkRemoved,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("identity_provider_link".to_string(), link.id, Some(alias))
                .with_details(serde_json::json!({
                    "user_id": user.id,
                    "identity_provider_id": link.identity_provider_id.as_uuid(),
                })),
            )
            .await?;

        self.identity_provider_link_repository
            .delete(link.id)
            .await?;

        Ok(())
    }
}
