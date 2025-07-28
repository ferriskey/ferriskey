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
        use_cases::entities::{AuthenticateCommand, AuthenticateResult, AuthenticationMethod},
    },
    jwt::{ports::jwt_service::JwtService, services::jwt_service::DefaultJwtService},
    realm::{
        entities::realm::Realm, ports::realm_service::RealmService,
        services::realm_service::DefaultRealmService,
    },
    user::entities::required_action::RequiredAction,
    utils::generate_random_string,
};

#[derive(Clone)]
pub struct AuthenticateUseCase {
    realm_service: DefaultRealmService,
    auth_session_service: DefaultAuthSessionService,
    jwt_service: DefaultJwtService,
    authentication_service: DefaultAuthenticationService,
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

impl AuthenticateUseCase {
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

    pub async fn execute(
        &self,
        command: AuthenticateCommand,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        info!("starting authentication for realm: {}", command.realm_name);

        // 1. Valider la session et le realm
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

        // Finaliser l'authentification
        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        // Déléguer l'authentification au service existant
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

        // Analyser le résultat et déterminer la prochaine étape
        self.determine_next_step(auth_result, params.session_code, auth_session)
            .await
    }

    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateResult, AuthenticationError> {
        // 1. Vérifier s'il y a des actions requises
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

        // 3. Authentification complète
        self.finalize_authentication(auth_result.user_id, session_code, auth_session)
            .await
    }

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
