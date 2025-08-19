use crate::application::common::services::{
    DefaultAuthSessionService, DefaultClientService, DefaultCredentialService, DefaultJwtService,
    DefaultRealmService, DefaultUserService,
};
use crate::domain::authentication::entities::{AuthSession, AuthenticationError};
use crate::domain::authentication::ports::AuthSessionService;
use crate::domain::authentication::value_objects::AuthenticationResult;
use crate::domain::client::ports::ClientService;
use crate::domain::common::generate_random_string;
use crate::domain::credential::ports::CredentialService;
use crate::domain::jwt::entities::{ClaimsTyp, JwtClaim};
use crate::domain::jwt::ports::JwtService;
use crate::domain::realm::entities::Realm;
use crate::domain::realm::ports::RealmService;
use crate::domain::user::entities::RequiredAction;
use crate::domain::user::ports::UserService;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthenticateUseCase {
    realm_service: DefaultRealmService,
    auth_session_service: DefaultAuthSessionService,
    jwt_service: DefaultJwtService,
    client_service: DefaultClientService,
    credential_service: DefaultCredentialService,
    user_service: DefaultUserService,
}

pub struct AuthenticateUseCaseResponse {
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

impl AuthenticateUseCaseResponse {
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

pub struct AuthenticateUseCaseParams {
    pub realm_name: String,
    pub client_id: String,
    pub session_code: Uuid,
    pub base_url: String,
    pub auth_method: AuthenticationMethod,
}

#[derive(Debug)]
struct CredentialsAuthParams {
    realm_name: String,
    client_id: String,
    session_code: Uuid,
    base_url: String,
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    UserCredentials { username: String, password: String },
    ExistingToken { token: String },
}

impl AuthenticateUseCaseParams {
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

impl AuthenticateUseCase {
    pub fn new(
        realm_service: DefaultRealmService,
        jwt_service: DefaultJwtService,
        auth_session_service: DefaultAuthSessionService,
        client_service: DefaultClientService,
        credential_service: DefaultCredentialService,
        user_service: DefaultUserService,
    ) -> Self {
        Self {
            realm_service,
            jwt_service,
            auth_session_service,
            client_service,
            credential_service,
            user_service,
        }
    }

    pub async fn execute(
        &self,
        params: AuthenticateUseCaseParams,
    ) -> Result<AuthenticateUseCaseResponse, AuthenticationError> {
        let (realm, auth_session) = self.validate_session_and_realm(&params).await?;

        match params.auth_method {
            AuthenticationMethod::ExistingToken { token } => {
                self.handle_token_refresh(token, realm.id, auth_session, params.session_code)
                    .await
            }
            AuthenticationMethod::UserCredentials { username, password } => {
                let params = CredentialsAuthParams {
                    realm_name: params.realm_name,
                    client_id: params.client_id,
                    session_code: params.session_code,
                    base_url: params.base_url,
                    username,
                    password,
                };

                self.handle_user_credentials_authentication(params, auth_session)
                    .await
            }
        }
    }

    async fn validate_session_and_realm(
        &self,
        params: &AuthenticateUseCaseParams,
    ) -> Result<(Realm, AuthSession), AuthenticationError> {
        let auth_session = self
            .auth_session_service
            .get_by_session_code(params.session_code)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let realm = self
            .realm_service
            .get_by_name(params.realm_name.clone())
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        Ok((realm, auth_session))
    }

    async fn handle_token_refresh(
        &self,
        token: String,
        realm_id: Uuid,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> Result<AuthenticateUseCaseResponse, AuthenticationError> {
        let claims = self
            .jwt_service
            .verify_token(token.clone(), realm_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        // Finalize authentication
        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateUseCaseResponse, AuthenticationError> {
        // Delegate authentication to the existing service
        let auth_result = self
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

    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateUseCaseResponse, AuthenticationError> {
        if !auth_result.required_actions.is_empty() {
            return Ok(AuthenticateUseCaseResponse::requires_actions(
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
            return Ok(AuthenticateUseCaseResponse::requires_otp_challenge(
                auth_result.user_id,
                token,
            ));
        }

        self.finalize_authentication(auth_result.user_id, session_code, auth_session)
            .await
    }

    async fn finalize_authentication(
        &self,
        user_id: Uuid,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateUseCaseResponse, AuthenticationError> {
        let authorization_code = generate_random_string();

        self.auth_session_service
            .update_code(session_code, authorization_code.clone(), user_id)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        let redirect_url = self.build_redirect_url(&auth_session, &authorization_code)?;

        Ok(AuthenticateUseCaseResponse::complete_with_redirect(
            user_id,
            authorization_code,
            redirect_url,
        ))
    }

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

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> Result<AuthenticationResult, AuthenticationError> {
        let realm = self
            .realm_service
            .get_by_name(realm_name)
            .await
            .map_err(|_| AuthenticationError::InvalidRealm)?;

        let _ = self
            .client_service
            .get_by_client_id(client_id.clone(), realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidClient);

        let user = self
            .user_service
            .get_by_username(username, realm.id)
            .await
            .map_err(|_| AuthenticationError::InvalidUser)?;

        let user_credentials = self
            .credential_service
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| AuthenticationError::InvalidUser)?;

        let has_temporary_password = user_credentials.iter().any(|cred| cred.temporary);

        let credentials: Vec<String> = user_credentials
            .iter()
            .map(|cred| cred.credential_type.clone())
            .collect();

        let has_valid_password = self
            .credential_service
            .verify_password(user.id, password)
            .await
            .map_err(|_| AuthenticationError::InvalidPassword)?;

        if !has_valid_password {
            return Err(AuthenticationError::InvalidPassword);
        }

        let iss = format!("{}/realms/{}", base_url, realm.name);
        let jwt_claim = JwtClaim::new(
            user.id,
            user.username.clone(),
            iss,
            vec![format!("{}-realm", realm.name), "account".to_string()],
            ClaimsTyp::Bearer,
            client_id.clone(),
            Some(user.email.clone()),
        );
        if !user.required_actions.is_empty() || has_temporary_password {
            let jwt_token = self
                .jwt_service
                .generate_token(jwt_claim, realm.id)
                .await
                .map_err(|_| AuthenticationError::InternalServerError)?;

            let required_actions = if has_temporary_password {
                vec![RequiredAction::UpdatePassword]
            } else {
                user.required_actions.clone()
            };

            return Ok(AuthenticationResult {
                code: None,
                required_actions,
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        let has_otp_credentials = credentials.iter().any(|cred| cred == "otp");
        if has_otp_credentials {
            let jwt_token = self
                .jwt_service
                .generate_token(jwt_claim, realm.id)
                .await
                .map_err(|_| AuthenticationError::InternalServerError)?;

            return Ok(AuthenticationResult {
                code: None,
                required_actions: user.required_actions.clone(),
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        self.auth_session_service
            .get_by_session_code(session_code)
            .await
            .map_err(|_| AuthenticationError::InternalServerError)?;

        Ok(AuthenticationResult {
            code: Some(generate_random_string()),
            required_actions: Vec::new(),
            user_id: user.id,
            token: None,
            credentials,
        })
    }
}
