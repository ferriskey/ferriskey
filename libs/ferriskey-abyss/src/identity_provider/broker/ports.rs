use std::future::Future;

use uuid::Uuid;

use crate::identity_provider::IdentityProviderId;
use ferriskey_domain::common::app_errors::CoreError;

use super::entities::{BrokerAuthSession, IdentityProviderLink};
use super::value_objects::{
    BrokerCallbackInput, BrokerCallbackOutput, BrokerLoginInput, BrokerLoginOutput,
    BrokeredUserInfo, CreateBrokerAuthSessionRequest, CreateIdentityProviderLinkRequest,
    OAuthProviderConfig, OAuthTokenResponse,
};

/// Repository trait for BrokerAuthSession persistence
pub trait BrokerAuthSessionRepository: Send + Sync {
    /// Creates a new broker auth session
    fn create(
        &self,
        request: CreateBrokerAuthSessionRequest,
    ) -> impl Future<Output = Result<BrokerAuthSession, CoreError>> + Send;

    /// Retrieves a broker auth session by its broker state (used for callback validation)
    fn get_by_broker_state(
        &self,
        broker_state: &str,
    ) -> impl Future<Output = Result<Option<BrokerAuthSession>, CoreError>> + Send;

    /// Retrieves a broker auth session by ID
    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<BrokerAuthSession>, CoreError>> + Send;

    /// Deletes a broker auth session by ID
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Deletes expired broker auth sessions (cleanup)
    fn delete_expired(&self) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

/// Repository trait for IdentityProviderLink persistence
pub trait IdentityProviderLinkRepository: Send + Sync {
    /// Creates a new identity provider link
    fn create(
        &self,
        request: CreateIdentityProviderLinkRequest,
    ) -> impl Future<Output = Result<IdentityProviderLink, CoreError>> + Send;

    /// Retrieves a link by identity provider and external user ID
    fn get_by_provider_and_external_id(
        &self,
        identity_provider_id: IdentityProviderId,
        external_user_id: &str,
    ) -> impl Future<Output = Result<Option<IdentityProviderLink>, CoreError>> + Send;

    /// Retrieves all links for a user
    fn get_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<Vec<IdentityProviderLink>, CoreError>> + Send;

    /// Retrieves a link by user ID and identity provider ID
    fn get_by_user_and_provider(
        &self,
        user_id: Uuid,
        identity_provider_id: IdentityProviderId,
    ) -> impl Future<Output = Result<Option<IdentityProviderLink>, CoreError>> + Send;

    /// Updates the stored token for a link
    fn update_token(
        &self,
        id: Uuid,
        token: Option<String>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Deletes a link by ID
    fn delete(&self, id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Deletes all links for a user
    fn delete_by_user_id(
        &self,
        user_id: Uuid,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

/// Trait for OAuth HTTP client operations (external IdP communication)
pub trait OAuthClient: Send + Sync {
    /// Exchange an authorization code for tokens at the IdP
    fn exchange_code(
        &self,
        token_url: &str,
        code: &str,
        redirect_uri: &str,
        client_id: &str,
        client_secret: &str,
        code_verifier: Option<&str>,
    ) -> impl Future<Output = Result<OAuthTokenResponse, CoreError>> + Send;

    /// Fetch user info from the IdP's userinfo endpoint
    fn fetch_userinfo(
        &self,
        userinfo_url: &str,
        access_token: &str,
    ) -> impl Future<Output = Result<BrokeredUserInfo, CoreError>> + Send;
}

/// Service trait for broker authentication business logic
pub trait BrokerService: Send + Sync {
    /// Initiates the SSO login flow
    fn initiate_login(
        &self,
        input: BrokerLoginInput,
    ) -> impl Future<Output = Result<BrokerLoginOutput, CoreError>> + Send;

    /// Handles the callback from the IdP
    fn handle_callback(
        &self,
        input: BrokerCallbackInput,
    ) -> impl Future<Output = Result<BrokerCallbackOutput, CoreError>> + Send;

    /// Extracts user info from OAuth tokens
    fn extract_user_info(
        &self,
        config: &OAuthProviderConfig,
        token_response: &OAuthTokenResponse,
    ) -> impl Future<Output = Result<BrokeredUserInfo, CoreError>> + Send;
}
