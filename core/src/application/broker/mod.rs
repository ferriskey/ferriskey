use crate::{
    ApplicationService,
    domain::{
        common::entities::app_errors::CoreError,
        identity_provider::broker::{
            self, BrokerCallbackInput, BrokerCallbackOutput, BrokerLoginOutput, BrokerService,
            BrokeredUserInfo, OAuthProviderConfig, OAuthTokenResponse,
        },
    },
};

impl BrokerService for ApplicationService {
    async fn extract_user_info(
        &self,
        config: &OAuthProviderConfig,
        token_response: &OAuthTokenResponse,
    ) -> Result<BrokeredUserInfo, CoreError> {
        self.broker_service
            .extract_user_info(config, token_response)
            .await
    }

    async fn handle_callback(
        &self,
        input: BrokerCallbackInput,
    ) -> Result<BrokerCallbackOutput, CoreError> {
        self.broker_service.handle_callback(input).await
    }

    async fn initiate_login(
        &self,
        input: broker::BrokerLoginInput,
    ) -> Result<BrokerLoginOutput, CoreError> {
        self.broker_service.initiate_login(input).await
    }
}
