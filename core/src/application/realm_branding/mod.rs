use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        realm_branding::{
            entities::{BrandingConfig, RealmBranding},
            ports::{GetBrandingInput, RealmBrandingService, UpdateBrandingInput},
        },
    },
};

impl RealmBrandingService for ApplicationService {
    async fn get_branding(
        &self,
        identity: Identity,
        input: GetBrandingInput,
    ) -> Result<BrandingConfig, CoreError> {
        self.realm_branding_service
            .get_branding(identity, input)
            .await
    }

    async fn update_branding(
        &self,
        identity: Identity,
        input: UpdateBrandingInput,
    ) -> Result<RealmBranding, CoreError> {
        self.realm_branding_service
            .update_branding(identity, input)
            .await
    }

    async fn get_public_branding(
        &self,
        input: GetBrandingInput,
    ) -> Result<BrandingConfig, CoreError> {
        self.realm_branding_service.get_public_branding(input).await
    }
}
