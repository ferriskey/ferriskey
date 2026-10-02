use crate::application::services::ApplicationService;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::consent::{
    ConsentDecisionOutcome, ConsentEvaluation, ConsentRequestView, ConsentService,
    DecideConsentInput, EvaluateConsentInput,
};

impl ApplicationService {
    pub async fn get_consent_request(
        &self,
        realm_name: &str,
        consent_token: &str,
    ) -> Result<ConsentRequestView, CoreError> {
        self.auth_service
            .get_consent_view(realm_name, consent_token)
            .await
    }

    pub async fn submit_consent_decision(
        &self,
        realm_name: &str,
        consent_token: &str,
        approved_scopes: Vec<String>,
    ) -> Result<String, CoreError> {
        self.auth_service
            .decide_consent(realm_name, consent_token, approved_scopes)
            .await
    }
}

impl ConsentService for ApplicationService {
    async fn evaluate(&self, input: EvaluateConsentInput) -> Result<ConsentEvaluation, CoreError> {
        self.consent_service.evaluate(input).await
    }

    async fn decide(&self, input: DecideConsentInput) -> Result<ConsentDecisionOutcome, CoreError> {
        self.consent_service.decide(input).await
    }

    async fn find_decision(
        &self,
        realm_id: ferriskey_domain::realm::RealmId,
        user_id: uuid::Uuid,
        client_id: uuid::Uuid,
    ) -> Result<Option<ferriskey_consent::ConsentDecision>, CoreError> {
        self.consent_service
            .find_decision(realm_id, user_id, client_id)
            .await
    }
}
