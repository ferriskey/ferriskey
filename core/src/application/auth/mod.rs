use crate::{
    ApplicationService,
    domain::{
        authentication::{
            entities::{
                AuthInput, AuthOutput, AuthenticateInput, AuthenticateOutput,
                AuthorizeRequestInput, AuthorizeRequestOutput, ExchangeTokenInput, JwtToken,
                TokenIntrospectionResponse,
            },
            ports::AuthService,
            value_objects::{
                EndSessionInput, EndSessionOutput, GenerateTokensForUserInput, GetUserInfoInput,
                Identity, IntrospectTokenInput, RegisterUserInput, RegisterUserOutput,
                RevokeTokenInput, UserInfoResponse,
            },
        },
        common::entities::app_errors::CoreError,
        email_verification::ports::EmailVerificationService,
        jwt::entities::JwkKey,
        user::ports::UserRepository,
    },
};
use tracing::warn;

impl AuthService for ApplicationService {
    async fn auth(&self, input: AuthInput) -> Result<AuthOutput, CoreError> {
        self.auth_service.auth(input).await
    }

    async fn authenticate(
        &self,
        input: AuthenticateInput,
    ) -> Result<AuthenticateOutput, CoreError> {
        self.auth_service.authenticate(input).await
    }

    async fn authorize_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        self.auth_service.authorize_request(input).await
    }

    async fn authorize_login_action_request(
        &self,
        input: AuthorizeRequestInput,
    ) -> Result<AuthorizeRequestOutput, CoreError> {
        self.auth_service
            .authorize_login_action_request(input)
            .await
    }

    async fn exchange_token(&self, input: ExchangeTokenInput) -> Result<JwtToken, CoreError> {
        self.auth_service.exchange_token(input).await
    }

    async fn get_certs(&self, realm_name: String) -> Result<Vec<JwkKey>, CoreError> {
        self.auth_service.get_certs(realm_name).await
    }

    async fn register_user(
        &self,
        url: String,
        input: RegisterUserInput,
    ) -> Result<RegisterUserOutput, CoreError> {
        let realm_name = input.realm_name.clone();
        let output = self.auth_service.register_user(url.clone(), input).await?;

        if let RegisterUserOutput::PendingVerification { user_id, .. } = &output
            && let Err(err) = self
                .email_verification_service
                .send_verification_email(*user_id, realm_name, url)
                .await
        {
            // Avoid leaving behind an unverified user that can no longer re-register.
            if let Err(cleanup_err) = self
                .auth_service
                .user_repository
                .delete_user(*user_id)
                .await
            {
                warn!(
                    user_id = %user_id,
                    error = %cleanup_err,
                    "Failed to roll back user after verification email delivery error"
                );
            }

            return Err(err);
        }

        Ok(output)
    }

    async fn get_userinfo(
        &self,
        identity: Identity,
        input: GetUserInfoInput,
    ) -> Result<UserInfoResponse, CoreError> {
        self.auth_service.get_userinfo(identity, input).await
    }

    async fn introspect_token(
        &self,
        input: IntrospectTokenInput,
    ) -> Result<TokenIntrospectionResponse, CoreError> {
        self.auth_service.introspect_token(input).await
    }

    async fn revoke_token(&self, input: RevokeTokenInput) -> Result<(), CoreError> {
        self.auth_service.revoke_token(input).await
    }

    async fn end_session(&self, input: EndSessionInput) -> Result<EndSessionOutput, CoreError> {
        self.auth_service.end_session(input).await
    }

    async fn generate_tokens_for_user(
        &self,
        input: GenerateTokensForUserInput,
    ) -> Result<JwtToken, CoreError> {
        self.auth_service.generate_tokens_for_user(input).await
    }
}
