use std::collections::HashMap;
use std::sync::Arc;

use tracing::instrument;
use uuid::Uuid;

use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::pagination::{Page, PageRequest};
use crate::domain::common::policies::ensure_policy;
use crate::domain::realm::entities::{RealmScope, Scoped, Unscoped};
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
    GetIdentityProviderInput, IdentityProvider, IdentityProviderFilter, IdentityProviderLinkView,
    IdentityProviderSortField, ListIdentityProviderLinksInput, ListIdentityProvidersInput,
    UpdateIdentityProviderInput,
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
    ) -> Result<(RealmScope, Scoped<User>), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), realm_name).await?;

        ensure_policy(
            self.identity_provider_policy
                .can_update_user(identity, scope.realm())
                .await,
            "insufficient permissions to manage identity provider links",
        )?;

        let user = self
            .user_repository
            .get_by_id(user_id)
            .await?
            .in_realm(&scope)?;

        Ok((scope, user))
    }

    async fn load_links_in_realm(
        &self,
        scope: &RealmScope,
        user: &Scoped<User>,
    ) -> Result<Vec<(IdentityProviderLink, String)>, CoreError> {
        let aliases = self
            .identity_provider_repository
            .list_identity_providers_by_realm(scope.id(), None)
            .await?
            .into_iter()
            .map(|provider| (provider.id.as_uuid(), provider.alias))
            .collect::<HashMap<_, _>>();

        let links = self
            .identity_provider_link_repository
            .get_by_user_id(user)
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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_create_identity_provider(&identity, scope.realm())
                .await,
            "insufficient permissions to create identity provider",
        )?;

        // Check if alias already exists in realm
        let exists = self
            .identity_provider_repository
            .exists_identity_provider_by_realm_and_alias(scope.id(), &input.alias)
            .await?;

        if exists {
            return Err(CoreError::ProviderNameAlreadyExists);
        }

        // Create the identity provider
        let request = CreateIdentityProviderRequest {
            realm_id: scope.id(),
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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(scope.id(), &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_view_identity_provider(&identity, scope.realm())
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
        request: PageRequest<IdentityProviderFilter, IdentityProviderSortField>,
    ) -> Result<Page<IdentityProvider>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        ensure_policy(
            self.identity_provider_policy
                .can_view_identity_provider(&identity, scope.realm())
                .await,
            "insufficient permissions to view identity providers",
        )?;

        self.identity_provider_repository
            .list(&scope, &request)
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
    async fn update_identity_provider(
        &self,
        identity: Identity,
        input: UpdateIdentityProviderInput,
    ) -> Result<IdentityProvider, CoreError> {
        // Resolve realm by name
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(scope.id(), &input.alias)
            .await?
            .map(Unscoped::new)
            .ok_or(CoreError::ProviderNotFound)?
            .in_realm(&scope)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_update_identity_provider(&identity, scope.realm())
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
            patch: input.patch,
        };

        self.identity_provider_repository
            .update_identity_provider(&provider, request)
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
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &input.realm_name).await?;

        // Get the identity provider
        let provider = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(scope.id(), &input.alias)
            .await?
            .map(Unscoped::new)
            .ok_or(CoreError::ProviderNotFound)?
            .in_realm(&scope)?;

        // Check authorization
        ensure_policy(
            self.identity_provider_policy
                .can_delete_identity_provider(&identity, scope.realm())
                .await,
            "insufficient permissions to delete identity provider",
        )?;

        self.identity_provider_repository
            .delete_identity_provider(&provider)
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
        let (scope, user) = self
            .resolve_user_for_link_management(&identity, &input.realm_name, input.user_id)
            .await?;

        let links = self.load_links_in_realm(&scope, &user).await?;

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
        let (scope, user) = self
            .resolve_user_for_link_management(&identity, &input.realm_name, input.user_id)
            .await?;

        let (link, alias) = self
            .load_links_in_realm(&scope, &user)
            .await?
            .into_iter()
            .find(|(link, _)| link.id == input.link_id)
            .ok_or(CoreError::NotFound)?;

        self.security_event_repository
            .store_event(
                SecurityEvent::new(
                    scope.id(),
                    SecurityEventType::IdentityProviderLinkRemoved,
                    EventStatus::Success,
                    identity.id(),
                )
                .with_target("identity_provider_link".to_string(), link.id, Some(alias))
                .with_details(serde_json::json!({
                    "user_id": user.get().id,
                    "identity_provider_id": link.identity_provider_id.as_uuid(),
                })),
            )
            .await?;

        self.identity_provider_link_repository
            .delete(&user, link.id)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::abyss::identity_provider::broker::ports::MockIdentityProviderLinkRepository;
    use crate::domain::abyss::identity_provider::ports::MockIdentityProviderRepository;
    use crate::domain::client::ports::MockClientRepository;
    use crate::domain::common::policies::FerriskeyPolicy;
    use crate::domain::common::services::tests::{
        create_test_realm_with_name, create_test_user_identity_with_realm,
    };
    use crate::domain::realm::entities::Realm;
    use crate::domain::realm::ports::MockRealmRepository;
    use crate::domain::role::entities::Role;
    use crate::domain::role::entities::permission::Permissions;
    use crate::domain::seawatch::ports::MockSecurityEventRepository;
    use crate::domain::user::ports::{MockUserRepository, MockUserRoleRepository};

    type TestService = IdentityProviderServiceImpl<
        MockIdentityProviderRepository,
        FerriskeyPolicy<MockUserRepository, MockClientRepository, MockUserRoleRepository>,
        MockRealmRepository,
        MockUserRepository,
        MockIdentityProviderLinkRepository,
        MockSecurityEventRepository,
    >;

    fn role_with(realm: &Realm, permissions: Vec<String>) -> Role {
        Role {
            id: Uuid::new_v4(),
            name: "caller".to_string(),
            description: None,
            permissions,
            realm_id: realm.id,
            client_id: None,
            client: None,
            require_mfa: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn service(
        realm: Realm,
        caller_id: Uuid,
        roles: Vec<Role>,
        repository: MockIdentityProviderRepository,
    ) -> TestService {
        let mut realm_repository = MockRealmRepository::new();
        realm_repository
            .expect_get_by_name()
            .with(mockall::predicate::eq("test-realm".to_string()))
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(Some(realm)) }));
        let mut user_role_repository = MockUserRoleRepository::new();
        user_role_repository
            .expect_get_user_roles()
            .with(mockall::predicate::eq(caller_id))
            .times(1)
            .return_once(move |_| Box::pin(async move { Ok(roles) }));
        let user_repository = Arc::new(MockUserRepository::new());
        let policy = FerriskeyPolicy::new(
            user_repository.clone(),
            Arc::new(MockClientRepository::new()),
            Arc::new(user_role_repository),
        );

        IdentityProviderServiceImpl::new(
            Arc::new(repository),
            Arc::new(policy),
            Arc::new(realm_repository),
            user_repository,
            Arc::new(MockIdentityProviderLinkRepository::new()),
            Arc::new(MockSecurityEventRepository::new()),
        )
    }

    fn list_request() -> PageRequest<IdentityProviderFilter, IdentityProviderSortField> {
        PageRequest {
            filter: IdentityProviderFilter {
                alias: Some("git".to_string()),
                ..IdentityProviderFilter::default()
            },
            ..PageRequest::default()
        }
    }

    fn caller_of(identity: &Identity) -> Uuid {
        match identity {
            Identity::User(user) => user.id,
            _ => panic!("Expected user identity"),
        }
    }

    fn input() -> ListIdentityProvidersInput {
        ListIdentityProvidersInput {
            realm_name: "test-realm".to_string(),
        }
    }

    #[tokio::test]
    async fn list_identity_providers_refuses_a_caller_without_view_rights_before_listing() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let caller_id = caller_of(&identity);
        let mut repository = MockIdentityProviderRepository::new();
        repository.expect_list().never();
        let service = service(realm, caller_id, vec![], repository);

        let result = service
            .list_identity_providers(identity, input(), list_request())
            .await;

        assert!(
            matches!(result, Err(CoreError::Forbidden(_))),
            "got {result:?}"
        );
    }

    #[tokio::test]
    async fn list_identity_providers_pages_the_resolved_realm_with_the_request() {
        let realm = create_test_realm_with_name("test-realm");
        let identity = create_test_user_identity_with_realm(&realm);
        let caller_id = caller_of(&identity);
        let viewer = role_with(&realm, vec![Permissions::ViewIdentityProviders.name()]);
        let realm_id = realm.id;
        let mut repository = MockIdentityProviderRepository::new();
        repository
            .expect_list()
            .withf(move |scope, request| scope.id() == realm_id && *request == list_request())
            .times(1)
            .return_once(move |_, request| {
                let page = Page::new(vec![], 7, request.page, request.limit);
                Box::pin(async move { Ok(page) })
            });
        let service = service(realm, caller_id, vec![viewer], repository);

        let page = service
            .list_identity_providers(identity, input(), list_request())
            .await
            .expect("listing succeeds");

        assert_eq!(page.metadata().total, 7);
    }
}
