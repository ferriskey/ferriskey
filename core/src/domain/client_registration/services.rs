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
    ClientRegistrationRequest, ClientRegistrationResponse, RegistrationError,
};
use crate::domain::client_registration::ports::ClientRegistrationService;
use crate::domain::client_registration::provision::{assign_default_scopes, registrable_scopes};
use crate::domain::client_registration::rate_limit::IpRateLimiter;
use crate::domain::client_registration::validation::{effective_scope, validate_registration};
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::generate_random_string;
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
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            redirect_uri_repository,
            scope_repository,
            scope_mapping_repository,
            rate_limiter,
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

        let client_id = Uuid::new_v4().to_string();
        let secret = (!validated.public_client).then(generate_random_string);

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

        assign_default_scopes(
            self.scope_repository.as_ref(),
            self.scope_mapping_repository.as_ref(),
            scope.id(),
            client.id,
            &validated.scopes,
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
            scope: effective_scope(&validated.scopes),
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

        self.store(&scope, request).await
    }
}

fn internal(failure: CoreError) -> RegistrationError {
    error!(error = ?failure, "dynamic client registration failed");
    RegistrationError::Internal
}
