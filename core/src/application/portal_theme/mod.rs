use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        portal_theme::{
            entities::{PortalTheme, PortalThemeConfig},
            ports::{
                CreateThemeInput, GetThemeByIdInput, GetThemeInput, ListThemesInput,
                PortalThemeService, UpdateThemeInput, UpdateThemeMetadataInput,
                UpdateThemePageInput,
            },
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

    async fn list_themes(
        &self,
        identity: Identity,
        input: ListThemesInput,
    ) -> Result<Vec<PortalTheme>, CoreError> {
        self.portal_theme_service.list_themes(identity, input).await
    }

    async fn get_theme_by_id(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .get_theme_by_id(identity, input)
            .await
    }

    async fn create_theme(
        &self,
        identity: Identity,
        input: CreateThemeInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .create_theme(identity, input)
            .await
    }

    async fn update_theme_metadata(
        &self,
        identity: Identity,
        input: UpdateThemeMetadataInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme_metadata(identity, input)
            .await
    }

    async fn update_theme_page(
        &self,
        identity: Identity,
        input: UpdateThemePageInput,
    ) -> Result<PortalTheme, CoreError> {
        self.portal_theme_service
            .update_theme_page(identity, input)
            .await
    }

    async fn activate_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        self.portal_theme_service
            .activate_theme(identity, input)
            .await
    }

    async fn delete_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> Result<(), CoreError> {
        self.portal_theme_service
            .delete_theme(identity, input)
            .await
    }

    async fn get_active_theme(
        &self,
        input: ListThemesInput,
    ) -> Result<Option<PortalTheme>, CoreError> {
        self.portal_theme_service.get_active_theme(input).await
    }
}
