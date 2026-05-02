use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        portal::{
            entities::PortalConfig,
            ports::{
                DeletePortalConfigInput, DisablePortalConfigInput, EnablePortalConfigInput,
                GetPortalConfigInput, PortalService, UpsertPortalConfigInput,
            },
        },
    },
};

impl PortalService for ApplicationService {
    async fn get_portal_config(
        &self,
        identity: Identity,
        input: GetPortalConfigInput,
    ) -> Result<Option<PortalConfig>, CoreError> {
        self.portal_service.get_portal_config(identity, input).await
    }

    async fn upsert_portal_config(
        &self,
        identity: Identity,
        input: UpsertPortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        self.portal_service
            .upsert_portal_config(identity, input)
            .await
    }

    async fn delete_portal_config(
        &self,
        identity: Identity,
        input: DeletePortalConfigInput,
    ) -> Result<(), CoreError> {
        self.portal_service
            .delete_portal_config(identity, input)
            .await
    }

    async fn enable_portal_config(
        &self,
        identity: Identity,
        input: EnablePortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        self.portal_service
            .enable_portal_config(identity, input)
            .await
    }

    async fn disable_portal_config(
        &self,
        identity: Identity,
        input: DisablePortalConfigInput,
    ) -> Result<PortalConfig, CoreError> {
        self.portal_service
            .disable_portal_config(identity, input)
            .await
    }

    async fn get_active_portal_config(
        &self,
        realm_name: &str,
    ) -> Result<Option<PortalConfig>, CoreError> {
        self.portal_service
            .get_active_portal_config(realm_name)
            .await
    }
}
