use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        client::{
            entities::{
                CreateTokenExchangePolicyInput, DeleteTokenExchangePolicyInput,
                GetTokenExchangePoliciesInput, token_exchange_policy::TokenExchangePolicy,
            },
            ports::TokenExchangePolicyService,
        },
        common::entities::app_errors::CoreError,
    },
};

impl TokenExchangePolicyService for ApplicationService {
    async fn create_token_exchange_policy(
        &self,
        identity: Identity,
        input: CreateTokenExchangePolicyInput,
    ) -> Result<TokenExchangePolicy, CoreError> {
        self.token_exchange_policy_service
            .create_token_exchange_policy(identity, input)
            .await
    }

    async fn get_token_exchange_policies(
        &self,
        identity: Identity,
        input: GetTokenExchangePoliciesInput,
    ) -> Result<Vec<TokenExchangePolicy>, CoreError> {
        self.token_exchange_policy_service
            .get_token_exchange_policies(identity, input)
            .await
    }

    async fn delete_token_exchange_policy(
        &self,
        identity: Identity,
        input: DeleteTokenExchangePolicyInput,
    ) -> Result<(), CoreError> {
        self.token_exchange_policy_service
            .delete_token_exchange_policy(identity, input)
            .await
    }
}
