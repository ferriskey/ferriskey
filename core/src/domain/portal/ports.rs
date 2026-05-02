use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity, common::entities::app_errors::CoreError,
    portal::entities::PortalConfig, realm::entities::Realm,
};

pub trait PortalService: Send + Sync {
    fn get_portal_config(
        &self,
        identity: Identity,
        input: GetPortalConfigInput,
    ) -> impl Future<Output = Result<Option<PortalConfig>, CoreError>> + Send;

    fn upsert_portal_config(
        &self,
        identity: Identity,
        input: UpsertPortalConfigInput,
    ) -> impl Future<Output = Result<PortalConfig, CoreError>> + Send;

    fn delete_portal_config(
        &self,
        identity: Identity,
        input: DeletePortalConfigInput,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn enable_portal_config(
        &self,
        identity: Identity,
        input: EnablePortalConfigInput,
    ) -> impl Future<Output = Result<PortalConfig, CoreError>> + Send;

    fn disable_portal_config(
        &self,
        identity: Identity,
        input: DisablePortalConfigInput,
    ) -> impl Future<Output = Result<PortalConfig, CoreError>> + Send;

    fn get_active_portal_config(
        &self,
        realm_name: &str,
    ) -> impl Future<Output = Result<Option<PortalConfig>, CoreError>> + Send;
}

#[cfg_attr(test, mockall::automock)]
pub trait PortalRepository: Send + Sync {
    fn get_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalConfig>, CoreError>> + Send;

    fn get_active_by_realm(
        &self,
        realm_id: Uuid,
    ) -> impl Future<Output = Result<Option<PortalConfig>, CoreError>> + Send;

    fn upsert(
        &self,
        realm_id: Uuid,
        layout: serde_json::Value,
    ) -> impl Future<Output = Result<PortalConfig, CoreError>> + Send;

    fn set_active(
        &self,
        realm_id: Uuid,
        is_active: bool,
    ) -> impl Future<Output = Result<PortalConfig, CoreError>> + Send;

    fn delete(&self, realm_id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;
}

pub trait PortalPolicy: Send + Sync {
    fn can_view_portal(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn can_manage_portal(
        &self,
        identity: &Identity,
        target_realm: &Realm,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}

pub struct GetPortalConfigInput {
    pub realm_name: String,
}

pub struct UpsertPortalConfigInput {
    pub realm_name: String,
    pub layout: serde_json::Value,
}

pub struct DeletePortalConfigInput {
    pub realm_name: String,
}

pub struct EnablePortalConfigInput {
    pub realm_name: String,
}

pub struct DisablePortalConfigInput {
    pub realm_name: String,
}
