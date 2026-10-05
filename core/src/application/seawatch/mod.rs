use ferriskey_domain::common::pagination::{Page, PageRequest};

use crate::{
    application::services::ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        seawatch::{
            SecurityEvent, VerifyResult,
            ports::SecurityEventService,
            value_objects::{SecurityEventFilter, SecurityEventSortField},
        },
    },
};

impl SecurityEventService for ApplicationService {
    async fn list_events(
        &self,
        identity: Identity,
        realm_name: String,
        request: PageRequest<SecurityEventFilter, SecurityEventSortField>,
    ) -> Result<Page<SecurityEvent>, CoreError> {
        self.security_event_service
            .list_events(identity, realm_name, request)
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
