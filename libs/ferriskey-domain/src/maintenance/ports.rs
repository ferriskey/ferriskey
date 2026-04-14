use uuid::Uuid;

use crate::auth::Identity;
use crate::common::app_errors::CoreError;
use crate::maintenance::entities::{MaintenanceWhitelistEntry, RealmMaintenanceWhitelistEntry};
use crate::maintenance::value_objects::ToggleMaintenanceRequest;
use crate::realm::RealmId;

pub trait MaintenanceWhitelistRepository: Send + Sync {
    fn add_user(
        &self,
        client_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<MaintenanceWhitelistEntry, CoreError>> + Send;

    fn add_role(
        &self,
        client_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<MaintenanceWhitelistEntry, CoreError>> + Send;

    fn remove(&self, entry_id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_by_client_id(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<MaintenanceWhitelistEntry>, CoreError>> + Send;

    fn get_whitelisted_user_ids(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;

    fn get_whitelisted_role_ids(
        &self,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;
}

pub trait RealmMaintenanceWhitelistRepository: Send + Sync {
    fn add_user(
        &self,
        realm_id: RealmId,
        user_id: Uuid,
    ) -> impl Future<Output = Result<RealmMaintenanceWhitelistEntry, CoreError>> + Send;

    fn add_role(
        &self,
        realm_id: RealmId,
        role_id: Uuid,
    ) -> impl Future<Output = Result<RealmMaintenanceWhitelistEntry, CoreError>> + Send;

    fn remove(&self, entry_id: Uuid) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_by_realm_id(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError>> + Send;

    fn get_whitelisted_user_ids(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;

    fn get_whitelisted_role_ids(
        &self,
        realm_id: RealmId,
    ) -> impl Future<Output = Result<Vec<Uuid>, CoreError>> + Send;
}

pub trait MaintenanceService: Send + Sync {
    fn toggle_maintenance(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        request: ToggleMaintenanceRequest,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn is_user_allowed(
        &self,
        client_id: Uuid,
        realm_id: RealmId,
        user_id: Uuid,
        user_role_ids: &[Uuid],
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn add_client_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        user_id: Uuid,
    ) -> impl Future<Output = Result<MaintenanceWhitelistEntry, CoreError>> + Send;

    fn add_client_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        role_id: Uuid,
    ) -> impl Future<Output = Result<MaintenanceWhitelistEntry, CoreError>> + Send;

    fn remove_client_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        entry_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_client_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
    ) -> impl Future<Output = Result<Vec<MaintenanceWhitelistEntry>, CoreError>> + Send;

    fn add_realm_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> impl Future<Output = Result<RealmMaintenanceWhitelistEntry, CoreError>> + Send;

    fn add_realm_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> impl Future<Output = Result<RealmMaintenanceWhitelistEntry, CoreError>> + Send;

    fn remove_realm_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        entry_id: Uuid,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn get_realm_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> impl Future<Output = Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError>> + Send;
}
