use std::sync::Arc;

use ferriskey_domain::client::entities::{
    Client, ClientFilter, ClientRegistrationSource, ClientSortField,
};
use ferriskey_domain::client::ports::ClientRepository;
use ferriskey_domain::client::value_objects::{CreateClientRequest, UpdateClientRequest};
use ferriskey_domain::common::app_errors::CoreError;
use ferriskey_domain::common::pagination::{Page, PageRequest};
use ferriskey_domain::realm::RealmId;
use ferriskey_domain::realm::scope::{RealmScope, Scoped, Unscoped};
use ferriskey_domain::role::entities::{Role, RoleFilter, RoleSortField};
use ferriskey_domain::role::ports::RoleRepository;
use ferriskey_domain::role::value_objects::{
    CreateRoleRequest, UpdateRolePermissionsRequest, UpdateRoleRequest,
};
use uuid::Uuid;

use super::state::TokenState;

#[derive(Debug, Clone)]
pub struct InvalidatingRoleRepository<R> {
    inner: Arc<R>,
    state: Arc<TokenState>,
}

impl<R> InvalidatingRoleRepository<R> {
    pub fn new(inner: Arc<R>, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: RoleRepository> RoleRepository for InvalidatingRoleRepository<R> {
    async fn create(&self, payload: CreateRoleRequest) -> Result<Role, CoreError> {
        self.inner.create(payload).await
    }

    async fn get_by_client_id(&self, client_id: Uuid) -> Result<Vec<Role>, CoreError> {
        self.inner.get_by_client_id(client_id).await
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Unscoped<Role>>, CoreError> {
        self.inner.get_by_id(id).await
    }

    async fn delete_by_id(&self, role: &Scoped<Role>) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_id(role).await;
        self.state.changed();
        deleted
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<RoleFilter, RoleSortField>,
    ) -> Result<Page<Role>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn find_by_name(
        &self,
        name: String,
        realm_id: Uuid,
    ) -> Result<Option<Unscoped<Role>>, CoreError> {
        self.inner.find_by_name(name, realm_id).await
    }

    async fn update_by_id(
        &self,
        role: &Scoped<Role>,
        payload: UpdateRoleRequest,
    ) -> Result<Role, CoreError> {
        let updated = self.inner.update_by_id(role, payload).await;
        self.state.changed();
        updated
    }

    async fn update_permissions_by_id(
        &self,
        role: &Scoped<Role>,
        payload: UpdateRolePermissionsRequest,
    ) -> Result<Role, CoreError> {
        let updated = self.inner.update_permissions_by_id(role, payload).await;
        self.state.changed();
        updated
    }
}

#[derive(Debug, Clone)]
pub struct InvalidatingClientRepository<R> {
    inner: R,
    state: Arc<TokenState>,
}

impl<R> InvalidatingClientRepository<R> {
    pub fn new(inner: R, state: Arc<TokenState>) -> Self {
        Self { inner, state }
    }
}

impl<R: ClientRepository> ClientRepository for InvalidatingClientRepository<R> {
    async fn create_client(&self, data: CreateClientRequest) -> Result<Client, CoreError> {
        self.inner.create_client(data).await
    }

    async fn get_by_client_id(
        &self,
        client_id: String,
        realm_id: RealmId,
    ) -> Result<Unscoped<Client>, CoreError> {
        self.inner.get_by_client_id(client_id, realm_id).await
    }

    async fn get_by_id(&self, realm_id: RealmId, id: Uuid) -> Result<Unscoped<Client>, CoreError> {
        self.inner.get_by_id(realm_id, id).await
    }

    async fn get_by_realm_id(&self, realm_id: RealmId) -> Result<Vec<Client>, CoreError> {
        self.inner.get_by_realm_id(realm_id).await
    }

    async fn list(
        &self,
        scope: &RealmScope,
        request: &PageRequest<ClientFilter, ClientSortField>,
    ) -> Result<Page<Client>, CoreError> {
        self.inner.list(scope, request).await
    }

    async fn update_client(
        &self,
        client: &Scoped<Client>,
        data: UpdateClientRequest,
    ) -> Result<Client, CoreError> {
        let updated = self.inner.update_client(client, data).await;
        self.state.changed();
        updated
    }

    async fn delete_by_id(&self, client: &Scoped<Client>) -> Result<(), CoreError> {
        let deleted = self.inner.delete_by_id(client).await;
        self.state.changed();
        deleted
    }

    async fn count_by_source(
        &self,
        realm_id: RealmId,
        source: ClientRegistrationSource,
    ) -> Result<u64, CoreError> {
        self.inner.count_by_source(realm_id, source).await
    }
}

#[cfg(test)]
mod tests {
    use ferriskey_domain::authentication::entities::AuthProtocol;
    use ferriskey_domain::client::entities::{ClientConfig, ClientType};
    use ferriskey_domain::client::ports::MockClientRepository;
    use ferriskey_domain::role::ports::MockRoleRepository;

    use super::*;
    use crate::infrastructure::token_state::test_support::{role, scoped, synced_state};

    fn client() -> Client {
        Client::new(ClientConfig {
            realm_id: RealmId::default(),
            name: "console".to_string(),
            client_id: "console".to_string(),
            secret: None,
            enabled: true,
            protocol: AuthProtocol::OpenIdConnect,
            public_client: true,
            service_account_enabled: false,
            client_type: ClientType::Public,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
        })
    }

    fn rename_client() -> UpdateClientRequest {
        UpdateClientRequest {
            name: None,
            client_id: Some("renamed".to_string()),
            enabled: None,
            direct_access_grants_enabled: None,
            oauth_device_code_grant_enabled: None,
            token_exchange_enabled: None,
            require_pkce: None,
            backchannel_logout_uri: None,
            backchannel_logout_session_required: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: None,
            maintenance_reason: None,
            maintenance_session_strategy: None,
            consent_required: None,
            token_endpoint_auth_method: None,
            jwks: None,
            jwks_uri: None,
        }
    }

    #[tokio::test]
    async fn every_role_write_announces_a_change() {
        let state = synced_state();
        let stored = role();
        let mut inner = MockRoleRepository::new();
        let returned = stored.clone();
        inner
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));
        inner.expect_update_by_id().returning(move |_, _| {
            let role = returned.clone();
            Box::pin(async move { Ok(role) })
        });
        let returned = stored.clone();
        inner
            .expect_update_permissions_by_id()
            .returning(move |_, _| {
                let role = returned.clone();
                Box::pin(async move { Ok(role) })
            });
        let repository = InvalidatingRoleRepository::new(Arc::new(inner), state.clone());
        let target = scoped(stored);

        repository
            .update_by_id(
                &target,
                UpdateRoleRequest {
                    name: Some("renamed".to_string()),
                    description: None,
                    require_mfa: None,
                },
            )
            .await
            .expect("rename");
        assert_eq!(state.generation(), 1);
        repository
            .update_permissions_by_id(
                &target,
                UpdateRolePermissionsRequest {
                    permissions: Vec::new(),
                },
            )
            .await
            .expect("permissions");
        assert_eq!(state.generation(), 2);
        repository.delete_by_id(&target).await.expect("delete");
        assert_eq!(state.generation(), 3);
    }

    #[tokio::test]
    async fn a_role_read_does_not_announce_a_change() {
        let state = synced_state();
        let mut inner = MockRoleRepository::new();
        inner
            .expect_get_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let repository = InvalidatingRoleRepository::new(Arc::new(inner), state.clone());

        repository.get_by_id(Uuid::new_v4()).await.expect("read");

        assert_eq!(state.generation(), 0);
    }

    #[tokio::test]
    async fn every_client_write_announces_a_change() {
        let state = synced_state();
        let stored = client();
        let mut inner = MockClientRepository::new();
        let returned = stored.clone();
        inner.expect_update_client().returning(move |_, _| {
            let client = returned.clone();
            Box::pin(async move { Ok(client) })
        });
        inner
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Ok(()) }));
        let repository = InvalidatingClientRepository::new(inner, state.clone());
        let target = scoped(stored);

        repository
            .update_client(&target, rename_client())
            .await
            .expect("update");
        assert_eq!(state.generation(), 1);
        repository.delete_by_id(&target).await.expect("delete");
        assert_eq!(state.generation(), 2);
    }

    #[tokio::test]
    async fn a_failed_client_write_still_announces_a_change() {
        let state = synced_state();
        let mut inner = MockClientRepository::new();
        inner
            .expect_delete_by_id()
            .returning(|_| Box::pin(async { Err(CoreError::InternalServerError) }));
        let repository = InvalidatingClientRepository::new(inner, state.clone());

        let deleted = repository.delete_by_id(&scoped(client())).await;

        assert!(deleted.is_err());
        assert_eq!(state.generation(), 1);
    }
}
