use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        seawatch::{SecurityEvent, ports::SecurityEventService, value_objects::FetchEventsInput},
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
}
