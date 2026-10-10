use std::net::IpAddr;
use std::sync::Arc;

use tracing::error;
use uuid::Uuid;

use crate::domain::aegis::ports::{ClientScopeMappingRepository, ClientScopeRepository};
use crate::domain::authentication::entities::AuthProtocol;
use crate::domain::client::entities::{ClientRegistrationSource, ClientType};
use crate::domain::client::ports::{ClientRepository, RedirectUriRepository};
use crate::domain::client::value_objects::CreateClientRequest;
use crate::domain::client_registration::entities::{
    ClientRegistrationRequest, ClientRegistrationResponse, MAX_DYNAMIC_CLIENTS_PER_REALM,
    RegistrationError,
};
use crate::domain::client_registration::ports::ClientRegistrationService;
use crate::domain::client_registration::provision::{assign_scopes, registrable_scopes};
use crate::domain::client_registration::validation::{effective_scope, validate_registration};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_random_token;
use crate::domain::common::rate_limit::{IpRateLimiter, RealmBudget};
use crate::domain::realm::entities::RealmScope;
use crate::domain::realm::ports::RealmRepository;

#[derive(Debug)]
pub struct ClientRegistrationServiceImpl<R, C, RU, CS, CSM>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    realm_repository: Arc<R>,
    client_repository: Arc<C>,
    redirect_uri_repository: Arc<RU>,
    scope_repository: Arc<CS>,
    scope_mapping_repository: Arc<CSM>,
    rate_limiter: IpRateLimiter,
    realm_budget: RealmBudget,
}

impl<R, C, RU, CS, CSM> ClientRegistrationServiceImpl<R, C, RU, CS, CSM>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        redirect_uri_repository: Arc<RU>,
        scope_repository: Arc<CS>,
        scope_mapping_repository: Arc<CSM>,
        rate_limiter: IpRateLimiter,
        realm_budget: RealmBudget,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            redirect_uri_repository,
            scope_repository,
            scope_mapping_repository,
            rate_limiter,
            realm_budget,
        }
    }

    async fn store(
        &self,
        scope: &RealmScope,
        request: ClientRegistrationRequest,
    ) -> Result<ClientRegistrationResponse, RegistrationError> {
        let registrable = registrable_scopes(self.scope_repository.as_ref(), scope.id())
            .await
            .map_err(internal)?;

        let validated = validate_registration(request, &registrable)?;

        let existing = self
            .client_repository
            .count_by_source(scope.id(), ClientRegistrationSource::Dynamic)
            .await
            .map_err(internal)?;
        if existing >= MAX_DYNAMIC_CLIENTS_PER_REALM {
            return Err(RegistrationError::InvalidClientMetadata(
                "this realm has reached its limit of dynamic clients",
            ));
        }

        let client_id = Uuid::new_v4().to_string();
        let secret = (!validated.public_client).then(generate_random_token);

        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id: scope.id(),
                name: validated.client_name.clone(),
                client_id: client_id.clone(),
                secret: secret.clone(),
                enabled: true,
                protocol: AuthProtocol::OpenIdConnect,
                public_client: validated.public_client,
                service_account_enabled: false,
                direct_access_grants_enabled: false,
                oauth_device_code_grant_enabled: false,
                token_exchange_enabled: false,
                client_type: if validated.public_client {
                    ClientType::Public
                } else {
                    ClientType::Confidential
                },
                require_pkce: validated.public_client,
                consent_required: true,
                registration_source: ClientRegistrationSource::Dynamic,
            })
            .await
            .map_err(internal)?;

        for uri in &validated.redirect_uris {
            self.redirect_uri_repository
                .create_redirect_uri(client.id, uri.clone(), true)
                .await
                .map_err(internal)?;
        }

        assign_scopes(
            self.scope_repository.as_ref(),
            self.scope_mapping_repository.as_ref(),
            scope.id(),
            client.id,
            validated.scopes.as_deref(),
        )
        .await
        .map_err(internal)?;

        Ok(ClientRegistrationResponse {
            client_id,
            client_id_issued_at: client.created_at.timestamp(),
            client_secret_expires_at: secret.as_ref().map(|_| 0),
            client_secret: secret,
            client_name: validated.client_name,
            redirect_uris: validated.redirect_uris,
            token_endpoint_auth_method: validated.token_endpoint_auth_method,
            grant_types: validated.grant_types,
            response_types: validated.response_types,
            scope: effective_scope(validated.scopes.as_deref().unwrap_or_default()),
        })
    }
}

impl<R, C, RU, CS, CSM> ClientRegistrationService
    for ClientRegistrationServiceImpl<R, C, RU, CS, CSM>
where
    R: RealmRepository,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn register(
        &self,
        realm_name: &str,
        source_ip: Option<IpAddr>,
        request: ClientRegistrationRequest,
    ) -> Result<ClientRegistrationResponse, RegistrationError> {
        let ip = source_ip.unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
        if !self.rate_limiter.allow(ip) {
            return Err(RegistrationError::RateLimited);
        }

        let scope = RealmScope::resolve(self.realm_repository.as_ref(), realm_name)
            .await
            .map_err(|_| RegistrationError::Disabled)?;

        let enabled = scope
            .realm()
            .settings
            .as_ref()
            .is_some_and(|settings| settings.dcr_enabled);
        if !enabled {
            return Err(RegistrationError::Disabled);
        }

        if !self.realm_budget.allow(Uuid::from(scope.id())) {
            return Err(RegistrationError::RateLimited);
        }

        self.store(&scope, request).await
    }
}

fn internal(failure: CoreError) -> RegistrationError {
    error!(error = ?failure, "dynamic client registration failed");
    RegistrationError::Internal
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aegis::mocks::{
        MockClientScopeMappingRepository, MockClientScopeRepository,
    };
    use crate::domain::client::entities::Client;
    use crate::domain::client::ports::{MockClientRepository, MockRedirectUriRepository};
    use crate::domain::common::services::tests::create_test_realm_with_name;
    use crate::domain::realm::entities::RealmSetting;
    use crate::domain::realm::ports::MockRealmRepository;

    type Service = ClientRegistrationServiceImpl<
        MockRealmRepository,
        MockClientRepository,
        MockRedirectUriRepository,
        MockClientScopeRepository,
        MockClientScopeMappingRepository,
    >;

    fn realms() -> MockRealmRepository {
        let mut realms = MockRealmRepository::new();
        let realm_id = create_test_realm_with_name("acme").id;
        realms.expect_get_by_name().returning(move |name| {
            let name = name.to_string();
            Box::pin(async move {
                let mut realm = create_test_realm_with_name(&name);
                realm.id = realm_id;
                let mut settings = RealmSetting::new(realm.id, None);
                settings.dcr_enabled = true;
                realm.settings = Some(settings);
                Ok(Some(realm))
            })
        });
        realms
    }

    fn scopes() -> MockClientScopeRepository {
        let mut scopes = MockClientScopeRepository::new();
        scopes
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        scopes
    }

    fn service(
        clients: MockClientRepository,
        redirects: MockRedirectUriRepository,
        limiter: IpRateLimiter,
        budget: RealmBudget,
    ) -> Service {
        ClientRegistrationServiceImpl::new(
            Arc::new(realms()),
            Arc::new(clients),
            Arc::new(redirects),
            Arc::new(scopes()),
            Arc::new(MockClientScopeMappingRepository::new()),
            limiter,
            budget,
        )
    }

    fn request(method: &str) -> ClientRegistrationRequest {
        ClientRegistrationRequest {
            redirect_uris: vec!["https://app.example/cb".to_string()],
            token_endpoint_auth_method: Some(method.to_string()),
            ..Default::default()
        }
    }

    fn ip(last: u8) -> Option<IpAddr> {
        Some(IpAddr::V4(std::net::Ipv4Addr::new(198, 51, 100, last)))
    }

    fn creating_clients(existing: u64) -> MockClientRepository {
        let mut clients = MockClientRepository::new();
        clients
            .expect_count_by_source()
            .returning(move |_, _| Box::pin(async move { Ok(existing) }));
        clients.expect_create_client().returning(|request| {
            Box::pin(async move {
                Ok(Client::from_realm_and_client_id(
                    request.realm_id,
                    request.client_id,
                ))
            })
        });
        clients
    }

    fn accepting_redirects() -> MockRedirectUriRepository {
        let mut redirects = MockRedirectUriRepository::new();
        redirects
            .expect_create_redirect_uri()
            .returning(|client_id, value, enabled| {
                Box::pin(async move {
                    Ok(crate::domain::client::entities::redirect_uri::RedirectUri {
                        id: Uuid::new_v4(),
                        client_id,
                        value,
                        enabled,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
                })
            });
        redirects
    }

    #[tokio::test]
    async fn a_confidential_client_gets_a_43_character_secret() {
        let service = service(
            creating_clients(0),
            accepting_redirects(),
            IpRateLimiter::per_minute(10),
            RealmBudget::per_hour(10),
        );

        let response = service
            .register("acme", ip(1), request("client_secret_basic"))
            .await
            .expect("the registration must succeed");

        assert_eq!(response.client_secret.map(|secret| secret.len()), Some(43));
    }

    #[tokio::test]
    async fn the_realm_budget_caps_registrations_from_many_addresses() {
        let service = service(
            creating_clients(0),
            accepting_redirects(),
            IpRateLimiter::per_minute(10),
            RealmBudget::per_hour(2),
        );

        for last in 1..=2 {
            assert!(
                service
                    .register("acme", ip(last), request("none"))
                    .await
                    .is_ok()
            );
        }

        assert_eq!(
            service.register("acme", ip(3), request("none")).await,
            Err(RegistrationError::RateLimited)
        );
    }

    #[tokio::test]
    async fn the_per_ip_bucket_still_applies() {
        let service = service(
            creating_clients(0),
            accepting_redirects(),
            IpRateLimiter::per_minute(1),
            RealmBudget::per_hour(10),
        );

        assert!(
            service
                .register("acme", ip(1), request("none"))
                .await
                .is_ok()
        );
        assert_eq!(
            service.register("acme", ip(1), request("none")).await,
            Err(RegistrationError::RateLimited)
        );
    }

    #[tokio::test]
    async fn a_realm_at_its_dynamic_client_cap_refuses_new_registrations() {
        let mut clients = MockClientRepository::new();
        clients.expect_count_by_source().returning(|_, source| {
            assert_eq!(source, ClientRegistrationSource::Dynamic);
            Box::pin(async { Ok(MAX_DYNAMIC_CLIENTS_PER_REALM) })
        });
        let service = service(
            clients,
            MockRedirectUriRepository::new(),
            IpRateLimiter::per_minute(10),
            RealmBudget::per_hour(10),
        );

        assert!(matches!(
            service.register("acme", ip(1), request("none")).await,
            Err(RegistrationError::InvalidClientMetadata(_))
        ));
    }

    #[tokio::test]
    async fn a_realm_just_under_the_cap_still_registers() {
        let service = service(
            creating_clients(MAX_DYNAMIC_CLIENTS_PER_REALM - 1),
            accepting_redirects(),
            IpRateLimiter::per_minute(10),
            RealmBudget::per_hour(10),
        );

        assert!(
            service
                .register("acme", ip(1), request("none"))
                .await
                .is_ok()
        );
    }
}
