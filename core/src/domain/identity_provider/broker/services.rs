use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::domain::authentication::entities::{AuthSession, AuthSessionParams};
use crate::domain::authentication::ports::AuthSessionRepository;
use crate::domain::client::ports::{ClientRepository, RedirectUriRepository};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::identity_provider::IdentityProvider;
use crate::domain::identity_provider::ports::IdentityProviderRepository;
use crate::domain::realm::entities::RealmId;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::user::entities::User;
use crate::domain::user::ports::UserRepository;
use crate::domain::user::value_objects::CreateUserRequest;

use super::entities::IdentityProviderLink;
use super::ports::{
    BrokerAuthSessionRepository, BrokerService, IdentityProviderLinkRepository, OAuthClient,
};
use super::value_objects::{
    BrokerCallbackInput, BrokerCallbackOutput, BrokerLoginInput, BrokerLoginOutput,
    BrokeredUserInfo, CreateBrokerAuthSessionRequest, CreateIdentityProviderLinkRequest,
    OAuthProviderConfig, OAuthTokenResponse,
};

/// Implementation of the BrokerService trait
#[derive(Clone, Debug)]
pub struct BrokerServiceImpl<RR, IR, BR, LR, CR, RUR, UR, ASR, OC>
where
    RR: RealmRepository,
    IR: IdentityProviderRepository,
    BR: BrokerAuthSessionRepository,
    LR: IdentityProviderLinkRepository,
    CR: ClientRepository,
    RUR: RedirectUriRepository,
    UR: UserRepository,
    ASR: AuthSessionRepository,
    OC: OAuthClient,
{
    realm_repository: Arc<RR>,
    identity_provider_repository: Arc<IR>,
    broker_session_repository: Arc<BR>,
    link_repository: Arc<LR>,
    client_repository: Arc<CR>,
    redirect_uri_repository: Arc<RUR>,
    user_repository: Arc<UR>,
    auth_session_repository: Arc<ASR>,
    oauth_client: Arc<OC>,
}

impl<RR, IR, BR, LR, CR, RUR, UR, ASR, OC> BrokerServiceImpl<RR, IR, BR, LR, CR, RUR, UR, ASR, OC>
where
    RR: RealmRepository,
    IR: IdentityProviderRepository,
    BR: BrokerAuthSessionRepository,
    LR: IdentityProviderLinkRepository,
    CR: ClientRepository,
    RUR: RedirectUriRepository,
    UR: UserRepository,
    ASR: AuthSessionRepository,
    OC: OAuthClient,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<RR>,
        identity_provider_repository: Arc<IR>,
        broker_session_repository: Arc<BR>,
        link_repository: Arc<LR>,
        client_repository: Arc<CR>,
        redirect_uri_repository: Arc<RUR>,
        user_repository: Arc<UR>,
        auth_session_repository: Arc<ASR>,
        oauth_client: Arc<OC>,
    ) -> Self {
        Self {
            realm_repository,
            identity_provider_repository,
            broker_session_repository,
            link_repository,
            client_repository,
            redirect_uri_repository,
            user_repository,
            auth_session_repository,
            oauth_client,
        }
    }

    /// Generates a cryptographically secure random string
    fn generate_random_string(length: usize) -> String {
        let mut bytes = vec![0u8; length];
        thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(&bytes)
    }

    /// Generates a PKCE code verifier
    fn generate_pkce_verifier() -> String {
        Self::generate_random_string(32)
    }

    /// Generates a PKCE code challenge from a verifier
    fn generate_pkce_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    /// Validates that the redirect URI is allowed for the client
    async fn validate_redirect_uri(
        &self,
        client_id: Uuid,
        redirect_uri: &str,
    ) -> Result<(), CoreError> {
        let redirect_uris = self
            .redirect_uri_repository
            .get_by_client_id(client_id)
            .await?;

        let is_valid = redirect_uris.iter().any(|uri| uri.value == redirect_uri);

        if !is_valid {
            return Err(CoreError::InvalidRedirectUri);
        }

        Ok(())
    }

    /// Builds the OAuth2 authorization URL for the IdP
    fn build_authorization_url(
        &self,
        config: &OAuthProviderConfig,
        callback_url: &str,
        broker_state: &str,
        code_challenge: Option<&str>,
        nonce: Option<&str>,
    ) -> String {
        let scopes = config.scopes.join(" ");

        let mut url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&state={}",
            config.authorization_url,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(callback_url),
            urlencoding::encode(broker_state),
        );

        // Add scopes
        if !scopes.is_empty() {
            url.push_str(&format!("&scope={}", urlencoding::encode(&scopes)));
        }

        // Add PKCE challenge if enabled
        if let Some(challenge) = code_challenge {
            url.push_str(&format!(
                "&code_challenge={}&code_challenge_method=S256",
                urlencoding::encode(challenge)
            ));
        }

        // Add nonce for OIDC
        if let Some(n) = nonce {
            url.push_str(&format!("&nonce={}", urlencoding::encode(n)));
        }

        url
    }

    /// Finds or creates a user based on the brokered user info
    async fn find_or_create_user(
        &self,
        realm_id: RealmId,
        idp: &IdentityProvider,
        user_info: &BrokeredUserInfo,
        access_token: Option<&str>,
    ) -> Result<(User, bool), CoreError> {
        // 1. Check if user is already linked to this IdP
        if let Some(link) = self
            .link_repository
            .get_by_provider_and_external_id(idp.id, &user_info.subject)
            .await?
        {
            // get_by_id returns Result<User>, not Result<Option<User>>
            // If user doesn't exist, it returns an error
            let user = self
                .user_repository
                .get_by_id(link.user_id)
                .await
                .map_err(|_| CoreError::UserNotFound)?;

            // Update token if store_token is enabled
            if idp.store_token
                && let Some(token) = access_token
            {
                self.link_repository
                    .update_token(link.id, Some(token.to_string()))
                    .await?;
            }

            return Ok((user, false));
        }

        // 2. If link_only mode, try to find user by email
        if idp.link_only {
            if let Some(email) = &user_info.email
                && let Some(user) = self.user_repository.get_by_email(email, realm_id).await?
            {
                // Link existing user
                self.create_idp_link(&user, idp, user_info, access_token)
                    .await?;
                return Ok((user, false));
            }
            // link_only mode and no matching user found
            return Err(CoreError::LinkOnlyUserNotFound);
        }

        // 3. Try to find by email if trust_email is enabled
        if idp.trust_email
            && let Some(email) = &user_info.email
            && user_info.email_verified.unwrap_or(false)
            && let Some(user) = self.user_repository.get_by_email(email, realm_id).await?
        {
            // Link existing user
            self.create_idp_link(&user, idp, user_info, access_token)
                .await?;
            return Ok((user, false));
        }

        // 4. Create new user
        let username = user_info.get_username(&idp.alias);

        let user = self
            .user_repository
            .create_user(CreateUserRequest {
                realm_id,
                client_id: None,
                username,
                firstname: user_info.given_name.clone().unwrap_or_default(),
                lastname: user_info.family_name.clone().unwrap_or_default(),
                email: user_info.email.clone().unwrap_or_default(),
                email_verified: user_info.email_verified.unwrap_or(false) && idp.trust_email,
                enabled: true,
            })
            .await?;

        // 5. Create IdP link
        self.create_idp_link(&user, idp, user_info, access_token)
            .await?;

        Ok((user, true))
    }

    /// Creates a link between a user and an identity provider
    async fn create_idp_link(
        &self,
        user: &User,
        idp: &IdentityProvider,
        user_info: &BrokeredUserInfo,
        access_token: Option<&str>,
    ) -> Result<IdentityProviderLink, CoreError> {
        let token = if idp.store_token {
            access_token.map(|t| t.to_string())
        } else {
            None
        };

        let request = CreateIdentityProviderLinkRequest {
            user_id: user.id,
            identity_provider_id: idp.id.into(),
            identity_provider_user_id: user_info.subject.clone(),
            identity_provider_username: user_info
                .preferred_username
                .clone()
                .or_else(|| user_info.email.clone())
                .unwrap_or_else(|| user_info.subject.clone()),
            token,
        };

        self.link_repository.create(request).await
    }

    /// Extracts user info from an ID token (JWT)
    fn extract_user_info_from_id_token(id_token: &str) -> Result<BrokeredUserInfo, CoreError> {
        // Split the JWT into parts
        let parts: Vec<&str> = id_token.split('.').collect();
        if parts.len() != 3 {
            return Err(CoreError::InvalidIdToken);
        }

        // Decode the payload (second part)
        let payload = URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|_| CoreError::InvalidIdToken)?;

        let claims: serde_json::Value =
            serde_json::from_slice(&payload).map_err(|_| CoreError::InvalidIdToken)?;

        Ok(BrokeredUserInfo {
            subject: claims["sub"]
                .as_str()
                .ok_or(CoreError::InvalidIdToken)?
                .to_string(),
            email: claims["email"].as_str().map(|s| s.to_string()),
            email_verified: claims["email_verified"].as_bool(),
            name: claims["name"].as_str().map(|s| s.to_string()),
            given_name: claims["given_name"].as_str().map(|s| s.to_string()),
            family_name: claims["family_name"].as_str().map(|s| s.to_string()),
            preferred_username: claims["preferred_username"].as_str().map(|s| s.to_string()),
            picture: claims["picture"].as_str().map(|s| s.to_string()),
        })
    }
}

impl<RR, IR, BR, LR, CR, RUR, UR, ASR, OC> BrokerService
    for BrokerServiceImpl<RR, IR, BR, LR, CR, RUR, UR, ASR, OC>
where
    RR: RealmRepository,
    IR: IdentityProviderRepository,
    BR: BrokerAuthSessionRepository,
    LR: IdentityProviderLinkRepository,
    CR: ClientRepository,
    RUR: RedirectUriRepository,
    UR: UserRepository,
    ASR: AuthSessionRepository,
    OC: OAuthClient,
{
    #[instrument(
        skip(self, input),
        fields(
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
            client.id = %input.client_id,
        )
    )]
    async fn initiate_login(
        &self,
        input: BrokerLoginInput,
    ) -> Result<BrokerLoginOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(input.realm_name.clone())
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let client = self
            .client_repository
            .get_by_client_id(input.client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::ClientNotFound)?;

        self.validate_redirect_uri(client.id, &input.redirect_uri)
            .await?;

        // 3. Get identity provider by alias
        let idp = self
            .identity_provider_repository
            .get_identity_provider_by_realm_and_alias(realm.id, &input.alias)
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        if !idp.enabled {
            return Err(CoreError::ProviderDisabled);
        }

        // 4. Parse OAuth config from idp.config
        let oauth_config: OAuthProviderConfig = idp.config.clone().try_into().map_err(|e| {
            error!("error: {e}");
            e
        })?;

        // 5. Generate secure random state for CSRF protection
        let broker_state = Self::generate_random_string(32);

        // 6. Generate PKCE if enabled
        let (code_verifier, code_challenge) = if oauth_config.use_pkce.unwrap_or(false) {
            let verifier = Self::generate_pkce_verifier();
            let challenge = Self::generate_pkce_challenge(&verifier);
            (Some(verifier), Some(challenge))
        } else {
            (None, None)
        };

        // 7. Create broker session
        let request = CreateBrokerAuthSessionRequest {
            realm_id: realm.id.into(),
            identity_provider_id: idp.id.into(),
            client_id: client.id,
            redirect_uri: input.redirect_uri.clone(),
            response_type: input.response_type.clone(),
            scope: input.scope.clone().unwrap_or_default(),
            state: input.state.clone(),
            nonce: input.nonce.clone(),
            broker_state: broker_state.clone(),
            code_verifier,
            auth_session_id: input.auth_session_id,
        };

        let broker_session = self.broker_session_repository.create(request).await?;

        // 8. Build IdP authorization URL
        let callback_url = format!(
            "{}/realms/{}/broker/{}/endpoint",
            input.base_url, input.realm_name, input.alias
        );

        let authorization_url = self.build_authorization_url(
            &oauth_config,
            &callback_url,
            &broker_state,
            code_challenge.as_deref(),
            input.nonce.as_deref(),
        );

        Ok(BrokerLoginOutput {
            authorization_url,
            broker_session_id: broker_session.id,
        })
    }

    #[instrument(
        skip(self, input),
        fields(
            realm.name = %input.realm_name,
            provider.alias = %input.alias,
        )
    )]
    async fn handle_callback(
        &self,
        input: BrokerCallbackInput,
    ) -> Result<BrokerCallbackOutput, CoreError> {
        // 1. Handle IdP errors - redirect to client with error
        if let Some(error) = &input.error {
            let broker_session = self
                .broker_session_repository
                .get_by_broker_state(&input.state)
                .await?
                .ok_or(CoreError::BrokerSessionNotFound)?;

            let error_desc = input.error_description.as_deref().unwrap_or("");
            let mut redirect_url = broker_session.redirect_uri.clone();
            redirect_url.push_str(&format!(
                "?error={}&error_description={}",
                urlencoding::encode(error),
                urlencoding::encode(error_desc)
            ));
            if let Some(state) = &broker_session.state {
                redirect_url.push_str(&format!("&state={}", urlencoding::encode(state)));
            }

            // Clean up the broker session
            self.broker_session_repository
                .delete(broker_session.id)
                .await?;

            return Err(CoreError::IdpAuthenticationFailed(format!(
                "{}: {}",
                error, error_desc
            )));
        }

        // 2. Validate code is present
        let code = input
            .code
            .as_ref()
            .ok_or(CoreError::MissingAuthorizationCode)?;

        // 3. Lookup broker session by state
        let broker_session = self
            .broker_session_repository
            .get_by_broker_state(&input.state)
            .await?
            .ok_or(CoreError::BrokerSessionNotFound)?;

        // 4. Check expiration
        if broker_session.is_expired() {
            self.broker_session_repository
                .delete(broker_session.id)
                .await?;
            return Err(CoreError::BrokerSessionExpired);
        }

        // 5. Resolve realm and IdP
        let realm = self
            .realm_repository
            .get_by_id(broker_session.realm_id)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let idp = self
            .identity_provider_repository
            .get_identity_provider_by_id(broker_session.identity_provider_id.into())
            .await?
            .ok_or(CoreError::ProviderNotFound)?;

        let oauth_config: OAuthProviderConfig = idp.config.clone().try_into()?;

        // 6. Exchange authorization code for tokens
        let callback_url = format!(
            "{}/realms/{}/broker/{}/endpoint",
            input.base_url, realm.name, idp.alias
        );

        let token_response = self
            .oauth_client
            .exchange_code(
                &oauth_config.token_url,
                code,
                &callback_url,
                &oauth_config.client_id,
                &oauth_config.client_secret,
                broker_session.code_verifier.as_deref(),
            )
            .await?;

        // 7. Extract user info from tokens
        let user_info = self
            .extract_user_info(&oauth_config, &token_response)
            .await?;

        // 8. Find or create user
        let (user, is_new_user) = self
            .find_or_create_user(
                realm.id,
                &idp,
                &user_info,
                Some(&token_response.access_token),
            )
            .await?;

        // 9. Create or update auth session with authorization code
        let authorization_code = Self::generate_random_string(32);

        // If we have an existing auth session, update it
        if let Some(auth_session_id) = broker_session.auth_session_id {
            self.auth_session_repository
                .update_user_id(auth_session_id, user.id)
                .await?;
            self.auth_session_repository
                .update_code(auth_session_id, authorization_code.clone())
                .await?;
        } else {
            // Create a new auth session
            let auth_session = AuthSession::new(AuthSessionParams {
                realm_id: realm.id,
                client_id: broker_session.client_id,
                redirect_uri: broker_session.redirect_uri.clone(),
                response_type: broker_session.response_type.clone(),
                scope: broker_session.scope.clone(),
                state: broker_session.state.clone(),
                nonce: broker_session.nonce.clone(),
                user_id: Some(user.id),
                code: Some(authorization_code.clone()),
                authenticated: true,
                webauthn_challenge: None,
                webauthn_challenge_issued_at: None,
            });
            self.auth_session_repository.create(&auth_session).await?;
        }

        // 10. Clean up broker session
        self.broker_session_repository
            .delete(broker_session.id)
            .await?;

        // 11. Build redirect URL back to client
        let mut redirect_url = broker_session.redirect_uri.clone();
        redirect_url.push_str(&format!(
            "?code={}",
            urlencoding::encode(&authorization_code)
        ));
        if let Some(state) = &broker_session.state {
            redirect_url.push_str(&format!("&state={}", urlencoding::encode(state)));
        }

        Ok(BrokerCallbackOutput {
            redirect_url,
            authorization_code,
            user_id: user.id,
            is_new_user,
        })
    }

    async fn extract_user_info(
        &self,
        config: &OAuthProviderConfig,
        token_response: &OAuthTokenResponse,
    ) -> Result<BrokeredUserInfo, CoreError> {
        // Try to extract from ID token first (more efficient)
        if let Some(id_token) = &token_response.id_token
            && let Ok(user_info) = Self::extract_user_info_from_id_token(id_token)
        {
            return Ok(user_info);
        }

        // Fall back to userinfo endpoint
        if let Some(userinfo_url) = &config.userinfo_url {
            return self
                .oauth_client
                .fetch_userinfo(userinfo_url, &token_response.access_token)
                .await;
        }

        Err(CoreError::IdpUserInfoFailed(
            "No ID token and no userinfo endpoint configured".to_string(),
        ))
    }
}
