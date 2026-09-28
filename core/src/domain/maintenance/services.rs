use std::sync::Arc;

use serde_json::json;
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::{
    authentication::value_objects::Identity,
    client::ports::ClientRepository,
    common::{
        entities::app_errors::CoreError,
        policies::{FerriskeyPolicy, ensure_policy},
    },
    maintenance::{
        entities::{MaintenanceWhitelistEntry, RealmMaintenanceWhitelistEntry},
        ports::{MaintenanceWhitelistRepository, RealmMaintenanceWhitelistRepository},
        value_objects::ToggleMaintenanceRequest,
    },
    realm::{
        entities::{RealmScope, Scoped},
        ports::RealmRepository,
    },
    seawatch::{EventStatus, SecurityEvent, SecurityEventRepository, SecurityEventType},
    user::ports::{UserRepository, UserRoleRepository},
    webhook::{
        entities::{webhook_payload::WebhookPayload, webhook_trigger::WebhookTrigger},
        ports::WebhookRepository,
    },
};
use ferriskey_domain::client::entities::Client;
use ferriskey_domain::client::ports::ClientPolicy;
use ferriskey_domain::client::value_objects::UpdateClientRequest;
use ferriskey_domain::maintenance::ports::MaintenanceService;
use ferriskey_domain::realm::RealmId;

#[derive(Clone, Debug)]
pub struct MaintenanceServiceImpl<R, U, C, UR, W, SE, MW, RMW>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) webhook_repository: Arc<W>,
    pub(crate) security_event_repository: Arc<SE>,
    pub(crate) maintenance_whitelist_repository: Arc<MW>,
    pub(crate) realm_maintenance_whitelist_repository: Arc<RMW>,
    pub(crate) policy: Arc<FerriskeyPolicy<U, C, UR>>,
}

impl<R, U, C, UR, W, SE, MW, RMW> MaintenanceServiceImpl<R, U, C, UR, W, SE, MW, RMW>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        webhook_repository: Arc<W>,
        security_event_repository: Arc<SE>,
        maintenance_whitelist_repository: Arc<MW>,
        realm_maintenance_whitelist_repository: Arc<RMW>,
        policy: Arc<FerriskeyPolicy<U, C, UR>>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            webhook_repository,
            security_event_repository,
            maintenance_whitelist_repository,
            realm_maintenance_whitelist_repository,
            policy,
        }
    }

    async fn load_entry_of_client(
        &self,
        entry_id: Uuid,
        client: &Scoped<Client>,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        let entry = self
            .maintenance_whitelist_repository
            .get_by_id(entry_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        if entry.client_id != client.get().id {
            warn!(
                entry_id = %entry_id,
                entry_client_id = %entry.client_id,
                request_client_id = %client.get().id,
                "Refused access to a maintenance whitelist entry of another client"
            );
            return Err(CoreError::NotFound);
        }

        Ok(entry)
    }
}

impl<R, U, C, UR, W, SE, MW, RMW> MaintenanceService
    for MaintenanceServiceImpl<R, U, C, UR, W, SE, MW, RMW>
where
    R: RealmRepository,
    U: UserRepository,
    C: ClientRepository,
    UR: UserRoleRepository,
    W: WebhookRepository,
    SE: SecurityEventRepository,
    MW: MaintenanceWhitelistRepository,
    RMW: RealmMaintenanceWhitelistRepository,
{
    async fn toggle_maintenance(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        request: ToggleMaintenanceRequest,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        let realm_id = scope.id();

        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to toggle maintenance",
        )?;

        let client = self
            .client_repository
            .get_by_id(realm_id, client_id)
            .await
            .map_err(|_| CoreError::ClientNotFound)?
            .in_realm(&scope)?;

        let update = UpdateClientRequest {
            name: None,
            client_id: None,
            enabled: None,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            require_pkce: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: Some(request.enabled),
            maintenance_reason: Some(request.reason.clone()),
            maintenance_session_strategy: request.session_strategy.clone(),
        };

        self.client_repository
            .update_client(&client, update)
            .await
            .map_err(|_| CoreError::InternalServerError)?;

        let (event_type, trigger) = if request.enabled {
            (
                SecurityEventType::ClientMaintenanceEnabled,
                WebhookTrigger::ClientMaintenanceEnabled,
            )
        } else {
            (
                SecurityEventType::ClientMaintenanceDisabled,
                WebhookTrigger::ClientMaintenanceDisabled,
            )
        };

        info!(
            "Maintenance mode {} for client {} ({})",
            if request.enabled {
                "enabled"
            } else {
                "disabled"
            },
            client.get().name,
            client_id
        );

        self.security_event_repository
            .store_event(
                SecurityEvent::new(realm_id, event_type, EventStatus::Success, identity.id())
                    .with_target("client".to_string(), client_id, None)
                    .with_details(json!({
                        "enabled": request.enabled,
                        "reason": request.reason,
                        "session_strategy": request.session_strategy,
                    })),
            )
            .await?;

        self.webhook_repository
            .notify(
                realm_id,
                WebhookPayload::new(trigger, realm_id.into(), Some(client.into_inner())),
            )
            .await?;

        Ok(())
    }

    async fn is_user_allowed(
        &self,
        client_id: Uuid,
        realm_id: RealmId,
        user_id: Uuid,
        user_role_ids: &[Uuid],
    ) -> Result<bool, CoreError> {
        let client_user_ids = self
            .maintenance_whitelist_repository
            .get_whitelisted_user_ids(client_id)
            .await?;
        if client_user_ids.contains(&user_id) {
            return Ok(true);
        }

        let client_role_ids = self
            .maintenance_whitelist_repository
            .get_whitelisted_role_ids(client_id)
            .await?;
        if user_role_ids.iter().any(|r| client_role_ids.contains(r)) {
            return Ok(true);
        }

        let realm_user_ids = self
            .realm_maintenance_whitelist_repository
            .get_whitelisted_user_ids(realm_id)
            .await?;
        if realm_user_ids.contains(&user_id) {
            return Ok(true);
        }

        let realm_role_ids = self
            .realm_maintenance_whitelist_repository
            .get_whitelisted_role_ids(realm_id)
            .await?;
        if user_role_ids.iter().any(|r| realm_role_ids.contains(r)) {
            return Ok(true);
        }

        Ok(false)
    }

    async fn add_client_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        user_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage maintenance whitelist",
        )?;
        let client = self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;
        self.maintenance_whitelist_repository
            .add_user(client.get().id, user_id)
            .await
    }

    async fn add_client_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        role_id: Uuid,
    ) -> Result<MaintenanceWhitelistEntry, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage maintenance whitelist",
        )?;
        let client = self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;
        self.maintenance_whitelist_repository
            .add_role(client.get().id, role_id)
            .await
    }

    async fn remove_client_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
        entry_id: Uuid,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage maintenance whitelist",
        )?;
        let client = self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;
        let entry = self.load_entry_of_client(entry_id, &client).await?;
        self.maintenance_whitelist_repository
            .remove(client.get().id, entry.id)
            .await
    }

    async fn get_client_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
        client_id: Uuid,
    ) -> Result<Vec<MaintenanceWhitelistEntry>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy.can_view_client(&identity, scope.realm()).await,
            "insufficient permissions to view maintenance whitelist",
        )?;
        let client = self
            .client_repository
            .get_by_id(scope.id(), client_id)
            .await
            .map_err(|_| CoreError::NotFound)?
            .in_realm(&scope)?;
        self.maintenance_whitelist_repository
            .get_by_client_id(client.get().id)
            .await
    }

    async fn add_realm_whitelist_user(
        &self,
        identity: Identity,
        realm_name: String,
        user_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage realm maintenance whitelist",
        )?;
        self.realm_maintenance_whitelist_repository
            .add_user(scope.id(), user_id)
            .await
    }

    async fn add_realm_whitelist_role(
        &self,
        identity: Identity,
        realm_name: String,
        role_id: Uuid,
    ) -> Result<RealmMaintenanceWhitelistEntry, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage realm maintenance whitelist",
        )?;
        self.realm_maintenance_whitelist_repository
            .add_role(scope.id(), role_id)
            .await
    }

    async fn remove_realm_whitelist_entry(
        &self,
        identity: Identity,
        realm_name: String,
        entry_id: Uuid,
    ) -> Result<(), CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy
                .can_update_client(&identity, scope.realm())
                .await,
            "insufficient permissions to manage realm maintenance whitelist",
        )?;
        let entry = self
            .realm_maintenance_whitelist_repository
            .get_by_id(entry_id)
            .await?
            .ok_or(CoreError::NotFound)?
            .in_realm(&scope)?;
        self.realm_maintenance_whitelist_repository
            .remove(&entry)
            .await
    }

    async fn get_realm_whitelist(
        &self,
        identity: Identity,
        realm_name: String,
    ) -> Result<Vec<RealmMaintenanceWhitelistEntry>, CoreError> {
        let scope = RealmScope::resolve(self.realm_repository.as_ref(), &realm_name).await?;
        ensure_policy(
            self.policy.can_view_client(&identity, scope.realm()).await,
            "insufficient permissions to view realm maintenance whitelist",
        )?;
        self.realm_maintenance_whitelist_repository
            .get_by_realm_id(scope.id())
            .await
    }
}
