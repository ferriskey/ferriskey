use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    portal_theme::entities::{PortalTheme, PortalThemeConfig},
    realm::entities::Realm,
};

pub trait PortalThemeService: Send + Sync {
    fn get_theme(
        &self,
        identity: Identity,
        input: GetThemeInput,
    ) -> impl Future<Output = Result<PortalThemeConfig, CoreError>> + Send;

    fn update_theme(
        &self,
        identity: Identity,
        input: UpdateThemeInput,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn get_public_theme(
        &self,
        input: GetThemeInput,
    ) -> impl Future<Output = Result<PortalThemeConfig, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PortalThemeRepository: Send + Sync {
    fn get_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalTheme>, CoreError>> + Send;

    fn upsert(
        &self,
        realm_id: Uuid,
        config: PortalThemeConfig,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;
}

pub trait PortalThemePolicy: Send + Sync {
    fn can_view_theme(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_theme(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetThemeInput {
    pub realm_name: String,
}

pub struct UpdateThemeInput {
    pub realm_name: String,
    pub config: PortalThemeConfig,
}
