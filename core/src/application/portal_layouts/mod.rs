use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        portal_layouts::{
            entities::PortalLayout,
            ports::{
                CreateLayoutInput, GetLayoutInput, ListLayoutsInput, PortalLayoutsService,
                UpdateLayoutInput,
            },
        },
    },
};

impl PortalLayoutsService for ApplicationService {
    async fn list_layouts(
        &self,
        identity: Identity,
        input: ListLayoutsInput,
    ) -> Result<Vec<PortalLayout>, CoreError> {
        self.portal_layouts_service
            .list_layouts(identity, input)
            .await
    }

    async fn get_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        self.portal_layouts_service
            .get_layout(identity, input)
            .await
    }

    async fn create_layout(
        &self,
        identity: Identity,
        input: CreateLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        self.portal_layouts_service
            .create_layout(identity, input)
            .await
    }

    async fn update_layout(
        &self,
        identity: Identity,
        input: UpdateLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        self.portal_layouts_service
            .update_layout(identity, input)
            .await
    }

    async fn set_default_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<PortalLayout, CoreError> {
        self.portal_layouts_service
            .set_default_layout(identity, input)
            .await
    }

    async fn delete_layout(
        &self,
        identity: Identity,
        input: GetLayoutInput,
    ) -> Result<(), CoreError> {
        self.portal_layouts_service
            .delete_layout(identity, input)
            .await
    }

    async fn get_public_default_layout(
        &self,
        input: ListLayoutsInput,
    ) -> Result<Option<PortalLayout>, CoreError> {
        self.portal_layouts_service
            .get_public_default_layout(input)
            .await
    }

    async fn get_public_layout(
        &self,
        input: GetLayoutInput,
    ) -> Result<Option<PortalLayout>, CoreError> {
        self.portal_layouts_service.get_public_layout(input).await
    }
}
