use tracing::info;
use uuid::Uuid;

use crate::domain::{
    authentication::{
        entities::{auth_session::AuthSession, error::AuthenticationError},
        ports::{
            auth_session::AuthSessionService,
            authentication::{AuthenticationResult, AuthenticationService},
        },
        service::{
            auth_session::DefaultAuthSessionService, authentication::DefaultAuthenticationService,
        },
    },
    jwt::{ports::jwt_service::JwtService, services::jwt_service::DefaultJwtService},
    realm::{
        entities::realm::Realm, ports::realm_service::RealmService,
        services::realm_service::DefaultRealmService,
    },
    user::entities::required_action::RequiredAction,
    utils::generate_random_string,
};

#[derive(Debug, Clone)]
pub struct AuthenticateResult {
    pub user_id: Uuid,
    pub status: AuthenticationStepStatus,
    pub authorization_code: Option<String>,
    pub temporary_token: Option<String>,
    pub required_actions: Vec<RequiredAction>,
    pub redirect_url: Option<String>,
    pub session_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthenticationStepStatus {
    Success,
    RequiresActions,
    RequiresOtpChallenge,
    Failed,
}

impl AuthenticateResult {
    pub fn complete_with_redirect(
        user_id: Uuid,
        authorization_code: String,
        redirect_url: String,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::Success,
            authorization_code: Some(authorization_code),
            temporary_token: None,
            required_actions: Vec::new(),
            redirect_url: Some(redirect_url),
            session_state: None,
        }
    }

    pub fn requires_actions(
        user_id: Uuid,
        required_actions: Vec<RequiredAction>,
        temporary_token: String,
    ) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresActions,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions,
            redirect_url: None,
            session_state: None,
        }
    }

    pub fn requires_otp_challenge(user_id: Uuid, temporary_token: String) -> Self {
        Self {
            user_id,
            status: AuthenticationStepStatus::RequiresOtpChallenge,
            authorization_code: None,
            temporary_token: Some(temporary_token),
            required_actions: Vec::new(),
            redirect_url: None,
            session_state: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthenticateCommand {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub auth_method: AuthenticationMethod,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    UserCredentials { username: String, password: String },
    ExistingToken { token: String },
}

impl AuthenticateCommand {
    pub fn with_user_credentials(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        username: String,
        password: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::UserCredentials { username, password },
        }
    }

    pub fn with_existing_token(
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        base_url: String,
        token: String,
    ) -> Self {
        Self {
            realm_name,
            client_id,
            session_code,
            base_url,
            auth_method: AuthenticationMethod::ExistingToken { token },
        }
    }

    pub fn is_token_refresh(&self) -> bool {
        matches!(self.auth_method, AuthenticationMethod::ExistingToken { .. })
    }

    pub fn is_credential_auth(&self) -> bool {
        matches!(
            self.auth_method,
            AuthenticationMethod::UserCredentials { .. }
        )
    }
}

/// Use case responsible for handling authentication flows in a hexagonal architecture.
///
/// This use case orchestrates the complete authentication process, including:
/// - Username/password authentication
/// - Token refresh authentication
/// - Multi-factor authentication (MFA) flow determination
/// - Session management and authorization code generation
///
/// The use case follows the Single Responsibility Principle (SRP) by encapsulating
/// all authentication business logic in one place, while delegating infrastructure
/// concerns to the injected services.
///
/// # Architecture
///
/// This use case sits in the domain layer and uses ports (interfaces) to communicate
/// with external services, ensuring loose coupling and testability.
///
/// # Example Usage
///
/// ```rust
/// let command = AuthenticateCommand::with_user_credentials(
///     "master".to_string(),
///     "client-id".to_string(),
///     session_uuid,
///     "https://myapp.com".to_string(),
///     "username".to_string(),
///     "password".to_string(),
/// );
///
/// let result = use_case.execute(command).await?;
/// ```
#[derive(Clone)]
pub struct AuthenticateUseCase {
    realm_service: DefaultRealmService,
    auth_session_service: DefaultAuthSessionService,
    jwt_service: DefaultJwtService,
    authentication_service: DefaultAuthenticationService,
}

/// Internal parameters for user credentials authentication.
///
/// This struct encapsulates all the necessary parameters for username/password
/// authentication to avoid having too many function parameters, which improves
/// code readability and maintainability.
///
/// # Fields
///
/// * `realm_name` - The name of the authentication realm
/// * `client_id` - OAuth client identifier
/// * `session_code` - Unique session identifier for this authentication attempt
/// * `base_url` - Base URL for token issuer claims
/// * `username` - User's username/email for authentication
/// * `password` - User's password for authentication
#[derive(Debug)]
struct CredentialsAuthParams {
    realm_name: String,
    client_id: String,
    session_code: Uuid,
    base_url: String,
    username: String,
    password: String,
}

impl AuthenticateUseCase {
    /// Creates a new instance of the AuthenticateUseCase.
    ///
    /// # Parameters
    ///
    /// * `realm_service` - Service for realm operations
    /// * `auth_session_service` - Service for session management
    /// * `jwt_service` - Service for JWT operations
    /// * `authentication_service` - Core authentication service
    ///
    /// # Returns
    ///
    /// A new configured instance of `AuthenticateUseCase`
    ///
    /// # Example
    ///
    /// ```rust
    /// let use_case = AuthenticateUseCase::new(
    ///     realm_service,
    ///     auth_session_service,
    ///     jwt_service,
    ///     authentication_service,
    /// );
    /// ```
    pub fn new(
        realm_service: DefaultRealmService,
        auth_session_service: DefaultAuthSessionService,
        jwt_service: DefaultJwtService,
        authentication_service: DefaultAuthenticationService,
    ) -> Self {
        Self {
            realm_service,
            auth_session_service,
            jwt_service,
            authentication_service,
        }
    }

    /// Executes the authentication use case.
    ///
    /// This is the main entry point for authentication operations. It handles both
    /// username/password authentication and token refresh scenarios, determining
    /// the appropriate authentication flow based on the command type.
    ///
    /// # Parameters
    ///
    /// * `command` - The authentication command containing all necessary data
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateResult)` - Successful authentication with next steps
    /// * `Err(AuthenticationError)` - Authentication failure with error details
    ///
    /// # Authentication Flow
    ///
    /// 1. Validates the session and realm
    /// 2. Routes to appropriate authentication method:
    ///    - Token refresh for existing valid tokens
    ///    - Credential authentication for username/password
    /// 3. Determines next steps (complete, requires MFA, requires actions)
    /// 4. Generates authorization codes and redirect URLs as needed
    ///
    /// # Errors
    ///
    /// Returns `AuthenticationError` for various failure scenarios:
    /// - Invalid realm or session
    /// - Invalid credentials
    /// - Token verification failures
    /// - Internal service errors
    pub async fn execute(
        &self,
        command: AuthenticateCommand,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        info!("starting authentication for realm: {}", command.realm_name);

        let (realm, auth_session) = self.validate_session_and_realm(&command).await?;

        match command.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.handle_token_refresh(token, realm.id, auth_session, command.session_code)
                    .await
            }
            AuthenticationMethod::UserCredentials { username, password } => {
                let params = CredentialsAuthParams {
                    realm_name: command.realm_name,
                    client_id: command.client_id,
                    session_code: command.session_code,
                    base_url: command.base_url,
                    username,
                    password,
                };

                self.handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }

    /// Validates that the session and realm exist and are valid.
    ///
    /// This method performs early validation to ensure that the authentication
    /// request is being made in the context of a valid session and realm.
    ///
    /// # Parameters
    ///
    /// * `command` - The authentication command containing session and realm info
    ///
    /// # Returns
    ///
    /// * `Ok((Realm, AuthSession))` - Valid realm and session objects
    /// * `Err(AuthenticationError)` - Invalid session or realm
    ///
    /// # Errors
    ///
    /// - `InvalidRealm` - If the realm doesn't exist
    /// - `InternalServerError` - If session lookup fails
    async fn validate_session_and_realm(
        &self,
        command: &AuthenticateCommand,
    ) -> Result<(Realm, AuthSession), AuthenticationError> {
        let auth_session = self
            .auth_session_service
            .get_by_session_code(command.session_code)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let realm = self
            .realm_service
            .get_by_name(command.realm_name.clone())
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        Ok((realm, auth_session))
    }

    /// Handles authentication using an existing JWT token (token refresh flow).
    ///
    /// This method is used when a client already has a valid JWT token and wants
    /// to complete the authentication flow without re-entering credentials.
    /// Typically used in scenarios like:
    /// - Session restoration
    /// - Silent authentication
    /// - Token-based re-authentication
    ///
    /// # Parameters
    ///
    /// * `token` - The existing JWT token to verify
    /// * `realm_id` - ID of the realm for token verification
    /// * `auth_session` - Current authentication session
    /// * `session_code` - Session identifier
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateResult)` - Successful token verification and authentication
    /// * `Err(AuthenticationError)` - Token verification failure
    ///
    /// # Process
    ///
    /// 1. Verifies the JWT token against the realm
    /// 2. Extracts user information from token claims
    /// 3. Finalizes authentication by generating authorization code
    async fn handle_token_refresh(
        &self,
        token: String,
        realm_id: Uuid,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        let claims = self
            .jwt_service
            .verify_token(token.clone(), realm_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        // Finalize authentication
        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    /// Handles authentication using username and password credentials.
    ///
    /// This method orchestrates the traditional username/password authentication
    /// flow and determines what additional steps may be required (MFA, account
    /// setup actions, etc.).
    ///
    /// # Parameters
    ///
    /// * `params` - Encapsulated authentication parameters
    /// * `auth_session` - Current authentication session
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateResult)` - Authentication result with next steps
    /// * `Err(AuthenticationError)` - Authentication failure
    ///
    /// # Process
    ///
    /// 1. Delegates primary authentication to the authentication service
    /// 2. Analyzes the result to determine next steps
    /// 3. Routes to appropriate flow (complete, MFA, required actions)
    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        // Delegate authentication to the existing service
        let auth_result = self
            .authentication_service
            .using_session_code(
                params.realm_name,
                params.client_id,
                params.session_code,
                params.username,
                params.password,
                params.base_url,
            )
            .await?;

        self.determine_next_step(auth_result, params.session_code, auth_session)
            .await
    }

    /// Determines the next step in the authentication flow based on user state.
    ///
    /// This method implements the authentication decision logic, determining whether:
    /// - The user needs to complete required actions (email verification, password reset)
    /// - The user needs to complete MFA/OTP challenge
    /// - The authentication can be finalized immediately
    ///
    /// # Parameters
    ///
    /// * `auth_result` - Result from the primary authentication service
    /// * `session_code` - Session identifier
    /// * `auth_session` - Current authentication session
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateResult)` - Next step in authentication flow
    /// * `Err(AuthenticationError)` - Processing error
    ///
    /// # Decision Logic
    ///
    /// 1. **Required Actions**: If user has pending required actions, return them
    /// 2. **OTP Challenge**: If user has OTP configured and no setup needed, require OTP
    /// 3. **Complete**: If no additional steps needed, finalize authentication
    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        if !auth_result.required_actions.is_empty() {
            return Ok(AuthenticateResult::requires_actions(
                auth_result.user_id,
                auth_result.required_actions,
                auth_result
                    .token
                    .ok_or(AuthenticationError::InternalServerError)?,
            ));
        }

        let has_otp_credentials = auth_result.credentials.iter().any(|cred| cred == "otp");
        let needs_configure_otp = auth_result
            .required_actions
            .contains(&RequiredAction::ConfigureOtp);

        if has_otp_credentials && !needs_configure_otp {
            let token = auth_result
                .token
                .ok_or(AuthenticationError::InternalServerError)?;
            return Ok(AuthenticateResult::requires_otp_challenge(
                auth_result.user_id,
                token,
            ));
        }

        self.finalize_authentication(auth_result.user_id, session_code, auth_session)
            .await
    }

    /// Finalizes the authentication process by generating authorization codes.
    ///
    /// This method completes the OAuth/OIDC authentication flow by:
    /// - Generating a unique authorization code
    /// - Updating the session with the code and user information
    /// - Building the redirect URL for the client application
    ///
    /// # Parameters
    ///
    /// * `user_id` - ID of the authenticated user
    /// * `session_code` - Session identifier
    /// * `auth_session` - Current authentication session
    ///
    /// # Returns
    ///
    /// * `Ok(AuthenticateResult)` - Complete authentication with redirect URL
    /// * `Err(AuthenticationError)` - Session update or URL building failure
    ///
    /// # OAuth Flow
    ///
    /// This method implements the final step of the OAuth authorization code flow,
    /// preparing the authorization code that will be exchanged for access tokens.
    async fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        let authorization_code = generate_random_string();

        self.auth_session_service
            .update_code(session_code, authorization_code.clone(), user_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let redirect_url = self.build_redirect_url(&auth_session, &authorization_code)?;

        Ok(AuthenticateResult::complete_with_redirect(
            user_id,
            authorization_code,
            redirect_url,
        ))
    }

    /// Builds the OAuth redirect URL with authorization code and state.
    ///
    /// Constructs the final redirect URL that will be sent back to the client
    /// application, including the authorization code and state parameter for
    /// security and flow tracking.
    ///
    /// # Parameters
    ///
    /// * `auth_session` - Authentication session containing redirect URI and state
    /// * `authorization_code` - Generated authorization code
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Complete redirect URL
    /// * `Err(AuthenticationError)` - Missing state parameter
    ///
    /// # URL Format
    ///
    /// The resulting URL follows the OAuth 2.0 specification:
    /// `{redirect_uri}?code={authorization_code}&state={state}`
    ///
    /// # Security
    ///
    /// The state parameter is required for CSRF protection in OAuth flows.
    fn build_redirect_url(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<String, AuthenticationError> {
        let state = auth_session
            .state
            .as_ref()
            .ok_or(AuthenticationError::InternalServerError)?;

        Ok(format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, state
        ))
    }
}
