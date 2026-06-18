use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        seawatch::{
            SecurityEvent, VerifyResult, ports::SecurityEventService,
            value_objects::FetchEventsInput,
        },
    },
};

impl SecurityEventService for ApplicationService {
    async fn fetch_events(
        &self,
        identity: Identity,
        input: FetchEventsInput,
    ) -> Result<Vec<SecurityEvent>, CoreError> {
        self.security_event_service
            .fetch_events(identity, input)
            .await
    }

    async fn verify_realm_chain(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<VerifyResult, CoreError> {
        self.security_event_service
            .verify_realm_chain(identity, realm_name)
            .await
    }
}
