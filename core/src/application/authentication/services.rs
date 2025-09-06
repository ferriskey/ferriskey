use std::vec;

use uuid::Uuid;

use crate::{
    application::common::services::DefaultJwtService,
    domain::{
        authentication::{
            entities::{
                AuthSession, AuthenticateOutput, AuthenticationStepStatus, CredentialsAuthParams,
            },
            ports::{AuthSessionRepository, AuthenticatePort},
            value_objects::AuthenticationResult,
        },
        client::ports::ClientRepository,
        common::{entities::app_errors::CoreError, generate_random_string},
        credential::ports::CredentialRepository,
        crypto::ports::HasherRepository,
        jwt::{
            entities::{ClaimsTyp, JwtClaim},
            ports::JwtService,
        },
        realm::ports::RealmRepository,
        user::{entities::RequiredAction, ports::UserRepository},
    },
    infrastructure::{
        auth_session::AuthSessionRepoAny, client::repositories::ClientRepoAny,
        credential::CredentialRepoAny, hasher::HasherRepoAny, realm::repositories::RealmRepoAny,
        user::UserRepoAny,
    },
};

#[derive(Clone)]
pub struct AuthenticateFactory {
    auth_session_repository: AuthSessionRepoAny,
    user_repository: UserRepoAny,
    realm_repository: RealmRepoAny,
    client_repository: ClientRepoAny,
    credential_repository: CredentialRepoAny,
    hasher_repository: HasherRepoAny,
    jwt_service: DefaultJwtService,
}

impl AuthenticateFactory {
    pub fn new(
        auth_session_repository: AuthSessionRepoAny,
        user_repository: UserRepoAny,
        realm_repository: RealmRepoAny,
        client_repository: ClientRepoAny,
        credential_repository: CredentialRepoAny,
        hasher_repository: HasherRepoAny,
        jwt_service: DefaultJwtService,
    ) -> Self {
        Self {
            auth_session_repository,
            user_repository,
            realm_repository,
            client_repository,
            credential_repository,
            hasher_repository,
            jwt_service,
        }
    }
}

impl AuthenticatePort for AuthenticateFactory {
    async fn determine_next_step(
        &self,
        auth_result: AuthenticationResult,
        session_code: Uuid,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
        if !auth_result.required_actions.is_empty() {
            return Ok(AuthenticateOutput::requires_actions(
                auth_result.user_id,
                auth_result.required_actions,
                auth_result.token.ok_or(CoreError::InternalServerError)?,
            ));
        }

        let has_otp_credentials = auth_result.credentials.iter().any(|cred| cred == "otp");
        let needs_configure_otp = auth_result
            .required_actions
            .contains(&RequiredAction::ConfigureOtp);

        if has_otp_credentials && !needs_configure_otp {
            let token = auth_result.token.ok_or(CoreError::InternalServerError)?;
            return Ok(AuthenticateOutput::requires_otp_challenge(
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
    ) -> Result<AuthenticateOutput, CoreError> {
        let authorization_code = generate_random_string();

        self.auth_session_repository
            .update_code_and_user_id(session_code, authorization_code.clone(), user_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let redirect_uri = self.build_redirect_url(&auth_session, &authorization_code)?;

        Ok(AuthenticateOutput::complete_with_redirect(
            user_id,
            authorization_code,
            redirect_uri,
        ))
    }

    async fn handle_token_refresh(
        &self,
        token: String,
        realm_id: Uuid,
        auth_session: AuthSession,
        session_code: Uuid,
    ) -> Result<AuthenticateOutput, CoreError> {
        let claims = self
            .jwt_service
            .verify_refresh_token(token.clone(), realm_id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let user = self
            .user_repository
            .get_by_id(claims.sub)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        if !user.required_actions.is_empty() {
            let jwt_token = self
                .jwt_service
                .generate_token(claims, realm_id)
                .await
                .map_err(|_| CoreError::InternalServerError)?;

            return Ok(AuthenticateOutput {
                status: AuthenticationStepStatus::RequiresActions,
                user_id: user.id,
                authorization_code: None,
                redirect_url: None,
                required_actions: user.required_actions,
                session_state: None,
                temporary_token: Some(jwt_token.token),
            });
        }

        self.finalize_authentication(claims.sub, session_code, auth_session)
            .await
    }

    async fn handle_user_credentials_authentication(
        &self,
        params: CredentialsAuthParams,
        auth_session: AuthSession,
    ) -> Result<AuthenticateOutput, CoreError> {
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

    async fn using_session_code(
        &self,
        realm_name: String,
        client_id: String,
        session_code: Uuid,
        username: String,
        password: String,
        base_url: String,
    ) -> Result<AuthenticationResult, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(realm_name)
            .await
            .map_err(|_| CoreError::InvalidRealm)?
            .ok_or(CoreError::InvalidRealm)?;

        self.client_repository
            .get_by_client_id(client_id.clone(), realm.id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        let user = self
            .user_repository
            .get_by_username(username, realm.id)
            .await
            .map_err(|_| CoreError::InvalidUser)?;

        let user_credentials = self
            .credential_repository
            .get_credentials_by_user_id(user.id)
            .await
            .map_err(|_| CoreError::GetUserCredentialsError)?;

        let has_temporary_password = user_credentials.iter().any(|cred| cred.temporary);

        let credentials: Vec<String> = user_credentials
            .iter()
            .map(|cred| cred.credential_type.clone())
            .collect();

        let credential = self
            .credential_repository
            .get_password_credential(user.id)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let salt = credential.salt.ok_or(CoreError::InternalServerError)?;

        let has_valid_password = self
            .hasher_repository
            .verify_password(
                &password,
                &credential.secret_data,
                &credential.credential_data,
                &salt,
            )
            .await
            .map_err(|_| CoreError::InvalidPassword)?;

        if !has_valid_password {
            return Err(CoreError::InvalidPassword);
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
                .map_err(|e| CoreError::TokenGenerationError(e.to_string()))?;

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
                .map_err(|_| CoreError::InternalServerError)?;

            return Ok(AuthenticationResult {
                code: None,
                required_actions: user.required_actions.clone(),
                user_id: user.id,
                token: Some(jwt_token.token),
                credentials,
            });
        }

        self.auth_session_repository
            .get_by_session_code(session_code)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        Ok(AuthenticationResult {
            code: Some(generate_random_string()),
            required_actions: Vec::new(),
            user_id: user.id,
            token: None,
            credentials,
        })
    }

    fn build_redirect_url(
        &self,
        auth_session: &AuthSession,
        authorization_code: &str,
    ) -> Result<String, CoreError> {
        let state = auth_session
            .state
            .as_ref()
            .ok_or(CoreError::InternalServerError)?;

        Ok(format!(
            "{}?code={}&state={}",
            auth_session.redirect_uri, authorization_code, state
        ))
    }
}
