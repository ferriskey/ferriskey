use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        portal_theme::{
            entities::{PortalTheme, PortalThemeConfig},
            ports::{GetThemeInput, PortalThemeService, UpdateThemeInput},
        },
    },
};

impl PortalThemeService for ApplicationService {
    async fn get_theme(
        &self,
        identity: Identity,
        input: GetThemeInput,
    ) -> Result<PortalThemeConfig, CoreError> {
        self.portal_theme_service.get_theme(identity, input).await
    }

    async fn update_theme(
        &self,
        identity: Identity,
        input: UpdateThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme(identity, input)
            .await
    }

    async fn get_public_theme(&self, input: GetThemeInput) -> Result<PortalThemeConfig, CoreError> {
        self.portal_theme_service.get_public_theme(input).await
    }
}
