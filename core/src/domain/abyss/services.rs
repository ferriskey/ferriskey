use std::sync::Arc;

use tracing::instrument;

use crate::domain::abyss::entities::{
    Provider, ProviderConfig, ProviderId, ProviderMapping, ProviderMappingConfig, ProviderMappingId,
};
use crate::domain::abyss::ports::{ProviderPolicy, ProviderRepository, ProviderService};
use crate::domain::abyss::value_objects::{
    CreateProviderInput, CreateProviderMappingInput, DeleteProviderInput,
    DeleteProviderMappingInput, GetProviderInput, GetProviderMappingsByProviderInput,
    GetProvidersByRealmInput, ToggleProviderInput, UpdateProviderInput,
};
use crate::domain::authentication::value_objects::Identity;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::policies::ensure_policy;
use crate::domain::realm::entities::RealmId;

/// Implementation of the ProviderService trait
///
/// Provides business logic for managing external identity providers,
/// including authorization checks and validation.
#[derive(Clone, Debug)]
pub struct ProviderServiceImpl<R, P>
where
    R: ProviderRepository,
    P: ProviderPolicy,
{
    provider_repository: Arc<R>,
    provider_policy: Arc<P>,
}

impl<R, P> ProviderServiceImpl<R, P>
where
    R: ProviderRepository,
    P: ProviderPolicy,
{
    /// Creates a new ProviderServiceImpl
    ///
    /// # Arguments
    /// * `provider_repository` - The provider repository for data access
    /// * `provider_policy` - The authorization policy for access control
    pub fn new(provider_repository: Arc<R>, provider_policy: Arc<P>) -> Self {
        Self {
            provider_repository,
            provider_policy,
        }
    }

    /// Validates that a URL is well-formed
    fn validate_url(url: &str) -> Result<(), CoreError> {
        if url.is_empty() {
            return Err(CoreError::InvalidProviderUrl);
        }

        // Basic URL validation - must start with http:// or https://
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(CoreError::InvalidProviderUrl);
        }

        Ok(())
    }

    /// Validates the provider input
    fn validate_provider_input(input: &CreateProviderInput) -> Result<(), CoreError> {
        // Validate name is not empty
        if input.name.trim().is_empty() {
            return Err(CoreError::InvalidProviderConfiguration(
                "name must not be empty".to_string(),
            ));
        }

        // Validate client_id is not empty
        if input.client_id.trim().is_empty() {
            return Err(CoreError::InvalidProviderConfiguration(
                "client_id must not be empty".to_string(),
            ));
        }

        // Validate client_secret is not empty
        if input.client_secret.trim().is_empty() {
            return Err(CoreError::InvalidProviderConfiguration(
                "client_secret must not be empty".to_string(),
            ));
        }

        // Validate URLs
        Self::validate_url(&input.authorization_url)?;
        Self::validate_url(&input.token_url)?;

        if let Some(ref userinfo_url) = input.userinfo_url {
            Self::validate_url(userinfo_url)?;
        }

        Ok(())
    }
}

impl<R, P> ProviderService for ProviderServiceImpl<R, P>
where
    R: ProviderRepository,
    P: ProviderPolicy,
{
    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.id = ?input.realm_id,
            provider.name = %input.name,
        )
    )]
    async fn create_provider(
        &self,
        identity: Identity,
        input: CreateProviderInput,
    ) -> Result<Provider, CoreError> {
        // Check authorization
        ensure_policy(
            self.provider_policy
                .can_create_provider(&identity, input.realm_id)
                .await,
            "insufficient permissions to create provider",
        )?;

        // Validate input
        Self::validate_provider_input(&input)?;

        // Check if provider name already exists in realm
        let existing = self
            .provider_repository
            .get_provider_by_realm_and_name(input.realm_id, input.name.clone())
            .await?;

        if existing.is_some() {
            return Err(CoreError::ProviderNameAlreadyExists);
        }

        // Create the provider
        let provider = Provider::new(ProviderConfig {
            realm_id: input.realm_id,
            name: input.name,
            provider_type: input.provider_type,
            client_id: input.client_id,
            client_secret: input.client_secret,
            authorization_url: input.authorization_url,
            token_url: input.token_url,
            userinfo_url: input.userinfo_url,
            scopes: input.scopes,
            configuration: input.configuration,
        });

        self.provider_repository.create_provider(&provider).await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.id,
        )
    )]
    async fn get_provider(
        &self,
        identity: Identity,
        input: GetProviderInput,
    ) -> Result<Provider, CoreError> {
        let provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.provider_policy
                .can_view_provider(&identity, &provider)
                .await,
            "insufficient permissions to view provider",
        )?;

        Ok(provider)
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            realm.id = ?input.realm_id,
        )
    )]
    async fn list_providers_by_realm(
        &self,
        identity: Identity,
        input: GetProvidersByRealmInput,
    ) -> Result<Vec<Provider>, CoreError> {
        let providers = self
            .provider_repository
            .list_providers_by_realm(input.realm_id)
            .await?;

        // Filter providers based on view permission
        let mut accessible_providers = Vec::new();
        for provider in providers {
            if self
                .provider_policy
                .can_view_provider(&identity, &provider)
                .await
                .unwrap_or(false)
            {
                accessible_providers.push(provider);
            }
        }

        Ok(accessible_providers)
    }

    #[instrument(
        skip(self),
        fields(
            realm.id = ?realm_id,
        )
    )]
    async fn list_enabled_providers(&self, realm_id: RealmId) -> Result<Vec<Provider>, CoreError> {
        self.provider_repository
            .list_enabled_providers_by_realm(realm_id)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.id,
        )
    )]
    async fn update_provider(
        &self,
        identity: Identity,
        input: UpdateProviderInput,
    ) -> Result<Provider, CoreError> {
        let mut provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.provider_policy
                .can_update_provider(&identity, &provider)
                .await,
            "insufficient permissions to update provider",
        )?;

        // Validate URLs if provided
        if let Some(ref url) = input.authorization_url {
            Self::validate_url(url)?;
        }
        if let Some(ref url) = input.token_url {
            Self::validate_url(url)?;
        }
        if let Some(ref userinfo_url) = input.userinfo_url {
            Self::validate_url(userinfo_url)?;
        }

        // Check if new name conflicts with existing provider
        if let Some(ref new_name) = input.name
            && new_name != &provider.name
        {
            let existing = self
                .provider_repository
                .get_provider_by_realm_and_name(provider.realm_id, new_name.clone())
                .await?;

            if existing.is_some() {
                return Err(CoreError::ProviderNameAlreadyExists);
            }
        }

        // Apply updates
        if let Some(name) = input.name {
            provider.set_name(name);
        }

        if let Some(client_id) = input.client_id {
            let client_secret = input
                .client_secret
                .unwrap_or_else(|| provider.client_secret.clone());
            provider.update_credentials(client_id, client_secret);
        } else if let Some(client_secret) = input.client_secret {
            provider.update_credentials(provider.client_id.clone(), client_secret);
        }

        if input.authorization_url.is_some()
            || input.token_url.is_some()
            || input.userinfo_url.is_some()
        {
            provider.update_urls(
                input
                    .authorization_url
                    .unwrap_or_else(|| provider.authorization_url.clone()),
                input
                    .token_url
                    .unwrap_or_else(|| provider.token_url.clone()),
                input.userinfo_url.or_else(|| provider.userinfo_url.clone()),
            );
        }

        if let Some(scopes) = input.scopes {
            provider.set_scopes(scopes);
        }

        if let Some(configuration) = input.configuration {
            provider.set_configuration(configuration);
        }

        self.provider_repository.update_provider(&provider).await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.id,
        )
    )]
    async fn delete_provider(
        &self,
        identity: Identity,
        input: DeleteProviderInput,
    ) -> Result<(), CoreError> {
        let provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.provider_policy
                .can_delete_provider(&identity, &provider)
                .await,
            "insufficient permissions to delete provider",
        )?;

        self.provider_repository
            .delete_provider(ProviderId::from(input.id))
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.id,
            enabled = %input.enabled,
        )
    )]
    async fn toggle_provider(
        &self,
        identity: Identity,
        input: ToggleProviderInput,
    ) -> Result<Provider, CoreError> {
        let mut provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Check authorization
        ensure_policy(
            self.provider_policy
                .can_update_provider(&identity, &provider)
                .await,
            "insufficient permissions to toggle provider",
        )?;

        provider.set_enabled(input.enabled);

        self.provider_repository.update_provider(&provider).await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.provider_id,
        )
    )]
    async fn create_provider_mapping(
        &self,
        identity: Identity,
        input: CreateProviderMappingInput,
    ) -> Result<ProviderMapping, CoreError> {
        // Verify provider exists and check authorization
        let provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.provider_id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        ensure_policy(
            self.provider_policy
                .can_update_provider(&identity, &provider)
                .await,
            "insufficient permissions to create provider mapping",
        )?;

        // Validate mapping fields
        if input.external_field.trim().is_empty() || input.internal_field.trim().is_empty() {
            return Err(CoreError::InvalidProviderConfiguration(
                "external_field and internal_field must not be empty".to_string(),
            ));
        }

        let mapping = ProviderMapping::new(ProviderMappingConfig {
            provider_id: ProviderId::from(input.provider_id),
            external_field: input.external_field,
            internal_field: input.internal_field,
            is_required: input.is_required,
        });

        self.provider_repository
            .create_provider_mapping(&mapping)
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            provider.id = ?input.provider_id,
        )
    )]
    async fn list_provider_mappings(
        &self,
        identity: Identity,
        input: GetProviderMappingsByProviderInput,
    ) -> Result<Vec<ProviderMapping>, CoreError> {
        // Verify provider exists and check authorization
        let provider = self
            .provider_repository
            .get_provider_by_id(ProviderId::from(input.provider_id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        ensure_policy(
            self.provider_policy
                .can_view_provider(&identity, &provider)
                .await,
            "insufficient permissions to view provider mappings",
        )?;

        self.provider_repository
            .list_provider_mappings_by_provider(ProviderId::from(input.provider_id))
            .await
    }

    #[instrument(
        skip(self, identity, input),
        fields(
            identity.id = %identity.id(),
            identity.kind = %identity.kind(),
            mapping.id = ?input.id,
        )
    )]
    async fn delete_provider_mapping(
        &self,
        identity: Identity,
        input: DeleteProviderMappingInput,
    ) -> Result<(), CoreError> {
        // Get the mapping first to find its provider
        let mapping = self
            .provider_repository
            .get_provider_mapping_by_id(ProviderMappingId::from(input.id))
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        // Get the provider and check authorization
        let provider = self
            .provider_repository
            .get_provider_by_id(mapping.provider_id)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        ensure_policy(
            self.provider_policy
                .can_update_provider(&identity, &provider)
                .await,
            "insufficient permissions to delete provider mapping",
        )?;

        self.provider_repository
            .delete_provider_mapping(ProviderMappingId::from(input.id))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::abyss::entities::ProviderType;
    use crate::domain::abyss::ports::MockProviderRepository;
    use crate::domain::realm::entities::{Realm, RealmId};
    use crate::domain::user::entities::User;
    use mockall::predicate;

    /// Mock implementation of ProviderPolicy for testing
    struct MockProviderPolicy {
        allow_all: bool,
    }

    impl MockProviderPolicy {
        fn allow_all() -> Self {
            Self { allow_all: true }
        }

        fn deny_all() -> Self {
            Self { allow_all: false }
        }
    }

    impl ProviderPolicy for MockProviderPolicy {
        fn can_create_provider(
            &self,
            _identity: &Identity,
            _realm_id: RealmId,
        ) -> impl Future<Output = Result<bool, CoreError>> + Send {
            let result = self.allow_all;
            async move { Ok(result) }
        }

        fn can_view_provider(
            &self,
            _identity: &Identity,
            _provider: &Provider,
        ) -> impl Future<Output = Result<bool, CoreError>> + Send {
            let result = self.allow_all;
            async move { Ok(result) }
        }

        fn can_update_provider(
            &self,
            _identity: &Identity,
            _provider: &Provider,
        ) -> impl Future<Output = Result<bool, CoreError>> + Send {
            let result = self.allow_all;
            async move { Ok(result) }
        }

        fn can_delete_provider(
            &self,
            _identity: &Identity,
            _provider: &Provider,
        ) -> impl Future<Output = Result<bool, CoreError>> + Send {
            let result = self.allow_all;
            async move { Ok(result) }
        }
    }

    fn create_test_user() -> User {
        let realm_id = RealmId::default();
        User {
            id: uuid::Uuid::new_v4(),
            realm_id,
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            firstname: "Test".to_string(),
            lastname: "User".to_string(),
            enabled: true,
            roles: vec![],
            realm: Some(Realm::new("test".to_string())),
            client_id: None,
            required_actions: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn create_test_identity() -> Identity {
        Identity::User(create_test_user())
    }

    fn create_valid_provider_input(realm_id: RealmId) -> CreateProviderInput {
        CreateProviderInput {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client123".to_string(),
            client_secret: "secret456".to_string(),
            authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            userinfo_url: Some("https://openidconnect.googleapis.com/v1/userinfo".to_string()),
            scopes: vec!["openid".to_string(), "email".to_string()],
            configuration: serde_json::json!({}),
        }
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(
            ProviderServiceImpl::<MockProviderRepository, MockProviderPolicy>::validate_url(
                "https://example.com"
            )
            .is_ok()
        );
        assert!(
            ProviderServiceImpl::<MockProviderRepository, MockProviderPolicy>::validate_url(
                "http://localhost:8080"
            )
            .is_ok()
        );
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(
            ProviderServiceImpl::<MockProviderRepository, MockProviderPolicy>::validate_url("")
                .is_err()
        );
        assert!(
            ProviderServiceImpl::<MockProviderRepository, MockProviderPolicy>::validate_url(
                "ftp://example.com"
            )
            .is_err()
        );
        assert!(
            ProviderServiceImpl::<MockProviderRepository, MockProviderPolicy>::validate_url(
                "not-a-url"
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn test_create_provider_success() {
        let realm_id = RealmId::default();
        let input = create_valid_provider_input(realm_id);
        let identity = create_test_identity();

        let mut mock_repo = MockProviderRepository::new();

        // Expect check for existing provider
        mock_repo
            .expect_get_provider_by_realm_and_name()
            .with(predicate::eq(realm_id), predicate::eq("Google".to_string()))
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));

        // Expect create
        mock_repo
            .expect_create_provider()
            .times(1)
            .returning(|provider| {
                let p = provider.clone();
                Box::pin(async move { Ok(p) })
            });

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service.create_provider(identity, input).await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.name, "Google");
        assert!(provider.enabled);
    }

    #[tokio::test]
    async fn test_create_provider_duplicate_name() {
        let realm_id = RealmId::default();
        let input = create_valid_provider_input(realm_id);
        let identity = create_test_identity();

        let mut mock_repo = MockProviderRepository::new();

        // Return existing provider
        let existing_provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "existing".to_string(),
            client_secret: "secret".to_string(),
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: None,
            scopes: vec![],
            configuration: serde_json::json!({}),
        });

        mock_repo
            .expect_get_provider_by_realm_and_name()
            .times(1)
            .returning(move |_, _| {
                let provider = existing_provider.clone();
                Box::pin(async move { Ok(Some(provider)) })
            });

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service.create_provider(identity, input).await;
        assert!(matches!(result, Err(CoreError::ProviderNameAlreadyExists)));
    }

    #[tokio::test]
    async fn test_create_provider_unauthorized() {
        let realm_id = RealmId::default();
        let input = create_valid_provider_input(realm_id);
        let identity = create_test_identity();

        let mock_repo = MockProviderRepository::new();
        let policy = MockProviderPolicy::deny_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service.create_provider(identity, input).await;
        assert!(matches!(result, Err(CoreError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_create_provider_invalid_url() {
        let realm_id = RealmId::default();
        let mut input = create_valid_provider_input(realm_id);
        input.authorization_url = "invalid-url".to_string();

        let identity = create_test_identity();
        let mock_repo = MockProviderRepository::new();
        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service.create_provider(identity, input).await;
        assert!(matches!(result, Err(CoreError::InvalidProviderUrl)));
    }

    #[tokio::test]
    async fn test_get_provider_success() {
        let realm_id = RealmId::default();
        let provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: None,
            scopes: vec![],
            configuration: serde_json::json!({}),
        });
        let provider_id: uuid::Uuid = provider.id.into();

        let mut mock_repo = MockProviderRepository::new();
        let provider_clone = provider.clone();
        mock_repo
            .expect_get_provider_by_id()
            .times(1)
            .returning(move |_| {
                let p = provider_clone.clone();
                Box::pin(async move { Ok(Some(p)) })
            });

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service
            .get_provider(create_test_identity(), GetProviderInput { id: provider_id })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Google");
    }

    #[tokio::test]
    async fn test_get_provider_not_found() {
        let mut mock_repo = MockProviderRepository::new();
        mock_repo
            .expect_get_provider_by_id()
            .times(1)
            .returning(|_| Box::pin(async { Ok(None) }));

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service
            .get_provider(
                create_test_identity(),
                GetProviderInput {
                    id: uuid::Uuid::new_v4(),
                },
            )
            .await;

        assert!(matches!(result, Err(CoreError::ProviderNotFound)));
    }

    #[tokio::test]
    async fn test_toggle_provider() {
        let realm_id = RealmId::default();
        let provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: None,
            scopes: vec![],
            configuration: serde_json::json!({}),
        });
        let provider_id: uuid::Uuid = provider.id.into();

        let mut mock_repo = MockProviderRepository::new();
        let provider_clone = provider.clone();

        mock_repo
            .expect_get_provider_by_id()
            .times(1)
            .returning(move |_| {
                let p = provider_clone.clone();
                Box::pin(async move { Ok(Some(p)) })
            });

        mock_repo
            .expect_update_provider()
            .times(1)
            .returning(|provider| {
                let p = provider.clone();
                Box::pin(async move { Ok(p) })
            });

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service
            .toggle_provider(
                create_test_identity(),
                ToggleProviderInput {
                    id: provider_id,
                    enabled: false,
                },
            )
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap().enabled);
    }

    #[tokio::test]
    async fn test_list_enabled_providers() {
        let realm_id = RealmId::default();
        let provider = Provider::new(ProviderConfig {
            realm_id,
            name: "Google".to_string(),
            provider_type: ProviderType::OAuth2,
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            authorization_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            userinfo_url: None,
            scopes: vec![],
            configuration: serde_json::json!({}),
        });

        let mut mock_repo = MockProviderRepository::new();
        let provider_clone = provider.clone();

        mock_repo
            .expect_list_enabled_providers_by_realm()
            .times(1)
            .returning(move |_| {
                let p = provider_clone.clone();
                Box::pin(async move { Ok(vec![p]) })
            });

        let policy = MockProviderPolicy::allow_all();
        let service = ProviderServiceImpl::new(Arc::new(mock_repo), Arc::new(policy));

        let result = service.list_enabled_providers(realm_id).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
