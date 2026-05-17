use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    common::entities::app_errors::CoreError,
    portal_theme::entities::{PortalPageType, PortalTheme, PortalThemeConfig},
    realm::entities::Realm,
};

pub trait PortalThemeService: Send + Sync {
    // ---------- Legacy single-theme-per-realm API (still wired to the old
    // /portal/theme endpoint; will be removed once PR4 lands). ----------

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

    // ---------- New collection API ----------

    fn list_themes(
        &self,
        identity: Identity,
        input: ListThemesInput,
    ) -> impl Future<Output = Result<Vec<PortalTheme>, CoreError>> + Send;

    fn get_theme_by_id(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn create_theme(
        &self,
        identity: Identity,
        input: CreateThemeInput,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn update_theme_metadata(
        &self,
        identity: Identity,
        input: UpdateThemeMetadataInput,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn update_theme_page(
        &self,
        identity: Identity,
        input: UpdateThemePageInput,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn activate_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete_theme(
        &self,
        identity: Identity,
        input: GetThemeByIdInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_active_theme(
        &self,
        input: ListThemesInput,
    ) -> impl Future<Output = Result<Option<PortalTheme>, CoreError>> + Send;
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

    fn list_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Vec<PortalTheme>, CoreError>> + Send;

    fn get_by_id(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalTheme>, CoreError>> + Send;

    fn create(
        &self,
        realm_id: Uuid,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn update_metadata(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        name: String,
        layout_id: Option<Uuid>,
        config: PortalThemeConfig,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn update_page(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
        page_type: PortalPageType,
        tree: serde_json::Value,
    ) -> impl Future<Output = Result<PortalTheme, CoreError>> + Send;

    fn activate(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn delete(
        &self,
        realm_id: Uuid,
        theme_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_active(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalTheme>, CoreError>> + Send;
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

pub struct ListThemesInput {
    pub realm_name: String,
}

pub struct GetThemeByIdInput {
    pub realm_name: String,
    pub theme_id: Uuid,
}

pub struct CreateThemeInput {
    pub realm_name: String,
    pub name: String,
    pub layout_id: Option<Uuid>,
    pub config: PortalThemeConfig,
}

pub struct UpdateThemeMetadataInput {
    pub realm_name: String,
    pub theme_id: Uuid,
    pub name: String,
    pub layout_id: Option<Uuid>,
    pub config: PortalThemeConfig,
}

pub struct UpdateThemePageInput {
    pub realm_name: String,
    pub theme_id: Uuid,
    pub page_type: PortalPageType,
    pub tree: serde_json::Value,
}
