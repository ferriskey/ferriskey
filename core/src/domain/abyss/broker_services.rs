use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use ferriskey_compass::{
    entities::{FlowStatus, FlowStepName, StepStatus},
    recorder::FlowRecorder,
};
use rand::{RngCore, thread_rng};
use sha2::{Digest, Sha256};
use tracing::{error, instrument, warn};
use uuid::Uuid;

use crate::domain::abyss::identity_provider::broker::{
    BrokerAuthSessionRepository, BrokerCallbackInput, BrokerCallbackOutput, BrokerLoginInput,
    BrokerLoginOutput, BrokerService, BrokeredUserInfo, CreateBrokerAuthSessionRequest,
    CreateIdentityProviderLinkRequest, IdentityProviderLink, IdentityProviderLinkRepository,
    OAuthClient, OAuthProviderConfig, OAuthTokenResponse,
};
use crate::domain::abyss::identity_provider::{IdentityProvider, IdentityProviderRepository};
use crate::domain::authentication::entities::{AuthProtocol, AuthSession, AuthSessionParams};
use crate::domain::authentication::ports::AuthSessionRepository;
use crate::domain::authentication::value_objects::CodeChallengeMethod;
use crate::domain::client::ports::{ClientRepository, RedirectUriRepository};
use crate::domain::client::redirect_uri_matching::redirect_uri_matches_any;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::entities::RealmId;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::user::entities::User;
use crate::domain::user::ports::UserRepository;
use crate::domain::user::value_objects::CreateUserRequest;

const ID_TOKEN_ALGORITHMS: &[jsonwebtoken::Algorithm] = &[
    jsonwebtoken::Algorithm::RS256,
    jsonwebtoken::Algorithm::RS384,
    jsonwebtoken::Algorithm::RS512,
    jsonwebtoken::Algorithm::PS256,
    jsonwebtoken::Algorithm::PS384,
    jsonwebtoken::Algorithm::PS512,
    jsonwebtoken::Algorithm::ES256,
    jsonwebtoken::Algorithm::ES384,
];

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
    flow_recorder: FlowRecorder,
}

fn evaluate_redirect_uri(allowed: &[String], redirect_uri: &str) -> Result<(), CoreError> {
    if allowed.is_empty() {
        return Err(CoreError::RedirectUriNotFound);
    }

    if redirect_uri_matches_any(allowed.iter().map(String::as_str), redirect_uri) {
        Ok(())
    } else {
        Err(CoreError::InvalidRedirectUri)
    }
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
        flow_recorder: FlowRecorder,
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
            flow_recorder,
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
            .get_enabled_by_client_id(client_id)
            .await?;

        let values = redirect_uris
            .into_iter()
            .map(|uri| uri.value)
            .collect::<Vec<_>>();

        if let Err(error) = evaluate_redirect_uri(&values, redirect_uri) {
            warn!(
                %client_id,
                redirect_uri = %redirect_uri,
                error = %error,
                "Broker redirect URI validation failed"
            );
            return Err(error);
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
                && user_info.email_verified.unwrap_or(false)
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
                firstname: user_info.given_name.clone(),
                lastname: user_info.family_name.clone(),
                email: user_info.email.clone(),
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

    async fn verify_id_token(
        &self,
        id_token: &str,
        config: &OAuthProviderConfig,
        expected_nonce: Option<&str>,
    ) -> Result<serde_json::Value, CoreError> {
        let jwks_url = config
            .jwks_url
            .as_deref()
            .ok_or(CoreError::InvalidIdToken)?;
        let issuer = config.issuer.as_deref().ok_or(CoreError::InvalidIdToken)?;

        let jwks = self.oauth_client.fetch_jwks(jwks_url).await?;

        verify_id_token_against_jwks(id_token, &jwks, issuer, &config.client_id, expected_nonce)
    }
}

fn verify_id_token_against_jwks(
    id_token: &str,
    jwks: &serde_json::Value,
    issuer: &str,
    audience: &str,
    expected_nonce: Option<&str>,
) -> Result<serde_json::Value, CoreError> {
    let header = jsonwebtoken::decode_header(id_token).map_err(|e| {
        warn!("id_token header is not decodable: {e}");
        CoreError::InvalidIdToken
    })?;

    let jwk_set: jsonwebtoken::jwk::JwkSet = serde_json::from_value(jwks.clone()).map_err(|e| {
        warn!("JWKS document is not a valid key set: {e}");
        CoreError::InvalidIdToken
    })?;

    if !ID_TOKEN_ALGORITHMS.contains(&header.alg) {
        warn!(
            "id_token declares algorithm {:?}, which is not accepted",
            header.alg
        );
        return Err(CoreError::InvalidIdToken);
    }

    let jwk = match header.kid.as_deref() {
        Some(kid) => jwk_set.find(kid),
        None => jwk_set.keys.first(),
    }
    .ok_or_else(|| {
        warn!("no JWKS key matches the id_token header");
        CoreError::InvalidIdToken
    })?;

    if let Some(key_alg) = jwk.common.key_algorithm
        && key_alg.to_string() != format!("{:?}", header.alg)
    {
        warn!("id_token algorithm does not match the JWKS key it selected");
        return Err(CoreError::InvalidIdToken);
    }

    let key = jsonwebtoken::DecodingKey::from_jwk(jwk).map_err(|e| {
        warn!("JWKS key is unusable: {e}");
        CoreError::InvalidIdToken
    })?;

    let mut validation = jsonwebtoken::Validation::new(header.alg);
    validation.set_issuer(&[issuer]);
    validation.set_audience(&[audience]);
    validation.validate_exp = true;

    let data =
        jsonwebtoken::decode::<serde_json::Value>(id_token, &key, &validation).map_err(|e| {
            warn!("id_token verification failed: {e}");
            CoreError::InvalidIdToken
        })?;

    let claims = data.claims;

    if let Some(expected) = expected_nonce {
        let presented = claims["nonce"].as_str();
        if presented != Some(expected) {
            warn!("id_token nonce does not match the one sent to the provider");
            return Err(CoreError::InvalidIdToken);
        }
    }

    Ok(claims)
}

fn user_info_from_claims(claims: serde_json::Value) -> Result<BrokeredUserInfo, CoreError> {
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
            .get_by_name(&input.realm_name)
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

        let challenge_method = input
            .code_challenge_method
            .as_deref()
            .map(|method| {
                method
                    .parse::<CodeChallengeMethod>()
                    .map_err(|_| CoreError::InvalidRequest)
            })
            .transpose()?;

        if client.require_pkce
            && !(input.code_challenge.is_some()
                && challenge_method == Some(CodeChallengeMethod::S256))
        {
            return Err(CoreError::PkceRequired);
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
            code_challenge: input.code_challenge.clone(),
            code_challenge_method: input.code_challenge_method.clone(),
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

        let client = self
            .client_repository
            .get_by_id(broker_session.realm_id, broker_session.client_id)
            .await?;

        let oauth_config: OAuthProviderConfig = idp.config.clone().try_into()?;

        // Start compass flow for broker authentication
        let flow_id = self
            .flow_recorder
            .start_flow(
                realm.id,
                Some(client.client_id.clone()),
                format!("broker_{}", idp.alias),
                None,
                None,
            )
            .await;

        // Step 1: IdP Redirect (the user was redirected to the external IdP)
        self.flow_recorder.record_step(
            flow_id.clone(),
            FlowStepName::IdpRedirect,
            StepStatus::Success,
            None,
            None,
            None,
        );

        // 6. Exchange authorization code for tokens (IdP callback)
        let callback_url = format!(
            "{}/realms/{}/broker/{}/endpoint",
            input.base_url, realm.name, idp.alias
        );

        let idp_params = {
            let mut parts = Vec::new();
            if let Some(c) = &input.code {
                parts.push(format!("code={}", c));
            }
            parts.push(format!("state={}", input.state));
            if let Some(e) = &input.error {
                parts.push(format!("error={}", e));
            }
            if let Some(ed) = &input.error_description {
                parts.push(format!("error_description={}", ed));
            }
            parts.join("&")
        };

        let cred_start = Utc::now();
        let token_response = match self
            .oauth_client
            .exchange_code(
                &oauth_config.token_url,
                code,
                &callback_url,
                &oauth_config.client_id,
                &oauth_config.client_secret,
                broker_session.code_verifier.as_deref(),
            )
            .await
        {
            Ok(response) => {
                let duration = (Utc::now() - cred_start).num_milliseconds();
                // Step 2: IdP Callback (received tokens from external IdP)
                self.flow_recorder.record_step(
                    flow_id.clone(),
                    FlowStepName::IdpCallback,
                    StepStatus::Success,
                    Some(duration),
                    None,
                    Some(idp_params.clone()),
                );
                response
            }
            Err(e) => {
                let duration = (Utc::now() - cred_start).num_milliseconds();
                self.flow_recorder.record_step(
                    flow_id.clone(),
                    FlowStepName::IdpCallback,
                    StepStatus::Failure,
                    Some(duration),
                    Some(format!("{:?}", e)),
                    Some(idp_params),
                );
                self.flow_recorder
                    .complete_flow(flow_id, FlowStatus::Failure, duration, None);
                return Err(e);
            }
        };

        // 7. Extract user info from tokens
        let user_info = self
            .extract_user_info(
                &oauth_config,
                &token_response,
                broker_session.nonce.as_deref(),
            )
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

        if !user.enabled {
            return Err(CoreError::UserDisabled);
        }

        // 9. Create or update auth session with authorization code
        // Set compass_flow_id so authorization_code() records TokenExchange + complete_flow
        let authorization_code = Self::generate_random_string(32);

        if let Some(auth_session_id) = broker_session.auth_session_id {
            self.auth_session_repository
                .update_user_id(auth_session_id, user.id)
                .await?;
            self.auth_session_repository
                .update_code(auth_session_id, authorization_code.clone())
                .await?;
            self.auth_session_repository
                .update_compass_flow_id(auth_session_id, flow_id.0)
                .await?;
        } else {
            let challenge_method = broker_session
                .code_challenge_method
                .as_deref()
                .map(|method| {
                    method.parse::<CodeChallengeMethod>().map_err(|_| {
                        warn!("broker session carries an unusable code_challenge_method");
                        CoreError::InvalidRequest
                    })
                })
                .transpose()?;

            let auth_session = AuthSession::new(AuthSessionParams {
                realm_id: realm.id,
                client_id: broker_session.client_id,
                protocol: AuthProtocol::OpenIdConnect,
                redirect_uri: broker_session.redirect_uri.clone(),
                response_type: Some(broker_session.response_type.clone()),
                scope: Some(broker_session.scope.clone()),
                state: broker_session.state.clone(),
                nonce: broker_session.nonce.clone(),
                user_id: Some(user.id),
                code: Some(authorization_code.clone()),
                authenticated: false,
                webauthn_challenge: None,
                webauthn_challenge_issued_at: None,
                compass_flow_id: Some(flow_id.0),
                code_challenge: broker_session.code_challenge.clone(),
                code_challenge_method: challenge_method,
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
            client_id: client.client_id,
        })
    }

    async fn extract_user_info(
        &self,
        config: &OAuthProviderConfig,
        token_response: &OAuthTokenResponse,
        expected_nonce: Option<&str>,
    ) -> Result<BrokeredUserInfo, CoreError> {
        if let Some(id_token) = &token_response.id_token
            && config.jwks_url.is_some()
            && config.issuer.is_some()
        {
            let claims = self
                .verify_id_token(id_token, config, expected_nonce)
                .await?;

            return user_info_from_claims(claims);
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

#[cfg(test)]
mod tests {
    use super::*;

    use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
    use chrono::Duration;
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs8::EncodePublicKey;
    use rsa::traits::PublicKeyParts;
    use rsa::{RsaPrivateKey, RsaPublicKey};

    const ISSUER: &str = "https://idp.example.com";
    const AUDIENCE: &str = "ferriskey-client";

    struct IdpKey {
        encoding: jsonwebtoken::EncodingKey,
        jwks: serde_json::Value,
    }

    fn idp_key(kid: &str) -> IdpKey {
        let mut rng = rand::thread_rng();
        let private = RsaPrivateKey::new(&mut rng, 2048).expect("generate rsa key");
        let public = RsaPublicKey::from(&private);

        let pem = private
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .expect("pem");
        let encoding =
            jsonwebtoken::EncodingKey::from_rsa_pem(pem.as_bytes()).expect("encoding key");

        let _ = public.to_public_key_pem(rsa::pkcs8::LineEnding::LF);
        let jwks = serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "kid": kid,
                "alg": "RS256",
                "n": B64.encode(public.n().to_bytes_be()),
                "e": B64.encode(public.e().to_bytes_be()),
            }]
        });

        IdpKey { encoding, jwks }
    }

    fn sign(key: &IdpKey, kid: &str, claims: serde_json::Value) -> String {
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(kid.to_string());
        jsonwebtoken::encode(&header, &claims, &key.encoding).expect("sign id_token")
    }

    fn claims(overrides: serde_json::Value) -> serde_json::Value {
        let mut base = serde_json::json!({
            "sub": "idp-user-1",
            "iss": ISSUER,
            "aud": AUDIENCE,
            "exp": (Utc::now() + Duration::minutes(5)).timestamp(),
            "iat": Utc::now().timestamp(),
            "nonce": "the-nonce",
        });

        for (k, v) in overrides.as_object().expect("object").iter() {
            base[k] = v.clone();
        }

        base
    }

    #[test]
    fn a_well_formed_id_token_is_accepted() {
        let key = idp_key("kid-1");
        let token = sign(&key, "kid-1", claims(serde_json::json!({})));

        let verified =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"))
                .expect("a token signed by the advertised key must verify");

        assert_eq!(verified["sub"], "idp-user-1");
    }

    #[test]
    fn a_token_signed_by_another_key_is_refused() {
        let key = idp_key("kid-1");
        let impostor = idp_key("kid-1");
        let token = sign(&impostor, "kid-1", claims(serde_json::json!({})));

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn a_token_from_another_issuer_is_refused() {
        let key = idp_key("kid-1");
        let token = sign(
            &key,
            "kid-1",
            claims(serde_json::json!({ "iss": "https://evil.example.com" })),
        );

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn a_token_for_another_audience_is_refused() {
        let key = idp_key("kid-1");
        let token = sign(
            &key,
            "kid-1",
            claims(serde_json::json!({ "aud": "someone-else" })),
        );

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn an_expired_token_is_refused() {
        let key = idp_key("kid-1");
        let expired = (Utc::now() - Duration::hours(1)).timestamp();
        let token = sign(&key, "kid-1", claims(serde_json::json!({ "exp": expired })));

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn a_replayed_nonce_is_refused() {
        let key = idp_key("kid-1");
        let token = sign(
            &key,
            "kid-1",
            claims(serde_json::json!({ "nonce": "a-different-login" })),
        );

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn a_token_naming_an_unknown_key_is_refused() {
        let key = idp_key("kid-1");
        let token = sign(&key, "kid-unknown", claims(serde_json::json!({})));

        let result =
            verify_id_token_against_jwks(&token, &key.jwks, ISSUER, AUDIENCE, Some("the-nonce"));

        assert!(matches!(result, Err(CoreError::InvalidIdToken)));
    }

    #[test]
    fn evaluate_redirect_uri_returns_not_found_when_allowed_list_is_empty() {
        let result = evaluate_redirect_uri(&[], "https://app.example/cb");

        assert!(matches!(result, Err(CoreError::RedirectUriNotFound)));
    }

    #[test]
    fn evaluate_redirect_uri_returns_invalid_when_no_entries_match() {
        let allowed = vec!["https://other.example/cb".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://app.example/cb");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_accepts_exact_match() {
        let allowed = vec!["https://app.example/cb".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://app.example/cb");

        assert!(result.is_ok());
    }

    #[test]
    fn evaluate_redirect_uri_accepts_anchored_regex_match() {
        let allowed = vec!["^https://app\\.example/.*$".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://app.example/cb");

        assert!(result.is_ok());
    }

    #[test]
    fn evaluate_redirect_uri_rejects_catch_all_pattern() {
        let allowed = vec!["^/*".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://attacker.example/steal");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_rejects_uri_merely_containing_a_registered_one() {
        let allowed = vec!["https://app.example/cb".to_string()];

        let result = evaluate_redirect_uri(
            &allowed,
            "https://attacker.example/?next=https://app.example/cb",
        );

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_does_not_treat_dots_in_a_literal_uri_as_wildcards() {
        let allowed = vec!["https://app.example/cb".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://appXexample/cb");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_ignores_an_unanchored_pattern() {
        let allowed = vec!["https://app\\.example/.*".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://app.example/cb");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_rejects_an_anchored_pattern_matching_the_empty_string() {
        let allowed = vec!["^.*$".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://attacker.example/steal");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }

    #[test]
    fn evaluate_redirect_uri_anchors_each_branch_of_an_alternation() {
        let allowed = vec!["^https://app\\.example/cb|attacker$".to_string()];

        let result = evaluate_redirect_uri(&allowed, "https://evil.example/attacker");

        assert!(matches!(result, Err(CoreError::InvalidRedirectUri)));
    }
}
