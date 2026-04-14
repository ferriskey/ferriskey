use uuid::Uuid;

use crate::{
    ApplicationService,
    domain::{
        authentication::value_objects::Identity,
        common::entities::app_errors::CoreError,
        maintenance::{
            entities::{MaintenanceWhitelistEntry, RealmMaintenanceWhitelistEntry},
            ports::MaintenanceService,
            value_objects::ToggleMaintenanceRequest,
        },
    },
};
use ferriskey_domain::realm::RealmId;

impl MaintenanceService for ApplicationService {
    async fn toggle_maintenance(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        request: ToggleMaintenanceRequest,
    ) -> Result<(), CoreError> {
        self.maintenance_service
            .toggle_maintenance(identity, realm_name, client_id, request)
            .await
    }

    async fn is_user_allowed(
        &self,
        client_id: Uuid,
        realm_id: RealmId,
        user_id: Uuid,
        user_role_ids: &[Uuid],
    ) -> Result<bool, CoreError> {
        self.maintenance_service
            .is_user_allowed(client_id, realm_id, user_id, user_role_ids)
            .await
    }

    async fn add_client_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        user_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        self.maintenance_service
            .add_client_whitelist_user(identity, realm_name, client_id, user_id)
            .await
    }

    async fn add_client_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        role_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        self.maintenance_service
            .add_client_whitelist_role(identity, realm_name, client_id, role_id)
            .await
    }

    async fn remove_client_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        entry_id: Uuid,
    ) -> Result<(), CoreError> {
        self.maintenance_service
            .remove_client_whitelist_entry(identity, realm_name, client_id, entry_id)
            .await
    }

    async fn get_client_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
    ) -> Result<Vec<MaintenanceWhitelistEntry>, CoreError> {
        self.maintenance_service
            .get_client_whitelist(identity, realm_name, client_id)
            .await
    }

    async fn add_realm_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        self.maintenance_service
            .add_realm_whitelist_user(identity, realm_name, user_id)
            .await
    }

    async fn add_realm_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        self.maintenance_service
            .add_realm_whitelist_role(identity, realm_name, role_id)
            .await
    }

    async fn remove_realm_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        entry_id: Uuid,
    ) -> Result<(), CoreError> {
        self.maintenance_service
            .remove_realm_whitelist_entry(identity, realm_name, entry_id)
            .await
    }

    async fn get_realm_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError> {
        self.maintenance_service
            .get_realm_whitelist(identity, realm_name)
            .await
    }
}
