use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    realm::entities::Realm,
    realm_branding::entities::{BrandingConfig, RealmBranding},
};

pub trait RealmBrandingService: Send + Sync {
    fn get_branding(
        &self,
        identity: Identity,
        input: GetBrandingInput,
    ) -> impl Future<Output = Result<BrandingConfig, CoreError>> + Send;

    fn update_branding(
        &self,
        identity: Identity,
        input: UpdateBrandingInput,
    ) -> impl Future<Output = Result<RealmBranding, CoreError>> + Send;

    fn get_public_branding(
        &self,
        input: GetBrandingInput,
    ) -> impl Future<Output = Result<BrandingConfig, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait RealmBrandingRepository: Send + Sync {
    fn get_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<RealmBranding>, CoreError>> + Send;

    fn upsert(
        &self,
        realm_id: Uuid,
        config: BrandingConfig,
    ) -> impl Future<Output = Result<RealmBranding, CoreError>> + Send;
}

pub trait RealmBrandingPolicy: Send + Sync {
    fn can_view_branding(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_branding(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetBrandingInput {
    pub realm_name: String,
}

pub struct UpdateBrandingInput {
    pub realm_name: String,
    pub config: BrandingConfig,
}
