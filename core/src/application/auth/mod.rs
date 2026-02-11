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
                GetUserInfoInput, Identity, IntrospectTokenInput, RegisterUserInput,
                UserInfoResponse,
            },
        },
        common::entities::app_errors::CoreError,
        jwt::entities::JwkKey,
    },
};

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
    ) -> Result<JwtToken, CoreError> {
        self.auth_service.register_user(url, input).await
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
}
