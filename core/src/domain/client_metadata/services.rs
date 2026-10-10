use std::net::IpAddr;
use std::sync::Arc;

use tracing::warn;

use crate::domain::aegis::ports::{ClientScopeMappingRepository, ClientScopeRepository};
use crate::domain::authentication::entities::AuthProtocol;
use crate::domain::client::entities::{
    Client, ClientRegistrationSource, ClientType, redirect_uri::RedirectUri,
};
use crate::domain::client::ports::{ClientRepository, RedirectUriRepository};
use crate::domain::client::value_objects::{CreateClientRequest, UpdateClientRequest};
use crate::domain::client_metadata::entities::{
    ClientMetadataDocument, ClientMetadataError, MAX_CLIENT_NAME_CHARS,
    MAX_METADATA_DOCUMENT_CLIENTS_PER_REALM,
};
use crate::domain::client_metadata::ports::{
    ClientMetadataDocumentFetcher, ClientMetadataResolver,
};
use crate::domain::client_metadata::validation::{
    ensure_host_allowed, is_metadata_client_id, parse_client_id_url, validate_document,
};
use crate::domain::client_registration::provision::assign_scopes;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::common::rate_limit::IpRateLimiter;
use crate::domain::realm::entities::RealmScope;

#[derive(Debug)]
pub struct ClientMetadataResolverImpl<F, C, RU, CS, CSM>
where
    F: ClientMetadataDocumentFetcher,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    fetcher: Arc<F>,
    client_repository: Arc<C>,
    redirect_uri_repository: Arc<RU>,
    scope_repository: Arc<CS>,
    scope_mapping_repository: Arc<CSM>,
    rate_limiter: Arc<IpRateLimiter>,
    allow_cleartext: bool,
}

impl<F, C, RU, CS, CSM> Clone for ClientMetadataResolverImpl<F, C, RU, CS, CSM>
where
    F: ClientMetadataDocumentFetcher,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    fn clone(&self) -> Self {
        Self {
            fetcher: Arc::clone(&self.fetcher),
            client_repository: Arc::clone(&self.client_repository),
            redirect_uri_repository: Arc::clone(&self.redirect_uri_repository),
            scope_repository: Arc::clone(&self.scope_repository),
            scope_mapping_repository: Arc::clone(&self.scope_mapping_repository),
            rate_limiter: Arc::clone(&self.rate_limiter),
            allow_cleartext: self.allow_cleartext,
        }
    }
}

impl<F, C, RU, CS, CSM> ClientMetadataResolverImpl<F, C, RU, CS, CSM>
where
    F: ClientMetadataDocumentFetcher,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    pub fn new(
        fetcher: Arc<F>,
        client_repository: Arc<C>,
        redirect_uri_repository: Arc<RU>,
        scope_repository: Arc<CS>,
        scope_mapping_repository: Arc<CSM>,
        rate_limiter: Arc<IpRateLimiter>,
        allow_cleartext: bool,
    ) -> Self {
        Self {
            fetcher,
            client_repository,
            redirect_uri_repository,
            scope_repository,
            scope_mapping_repository,
            rate_limiter,
            allow_cleartext,
        }
    }

    async fn find_existing(
        &self,
        scope: &RealmScope,
        client_id: &str,
    ) -> Result<Option<Client>, CoreError> {
        match self
            .client_repository
            .get_by_client_id(client_id.to_string(), scope.id())
            .await
        {
            Ok(client) => Ok(Some(client.in_realm(scope)?.into_inner())),
            Err(CoreError::NotFound) => Ok(None),
            Err(error) => Err(error),
        }
    }

    async fn upsert(
        &self,
        scope: &RealmScope,
        host: &str,
        document: &ClientMetadataDocument,
        existing: Option<Client>,
    ) -> Result<(), CoreError> {
        let name = display_name(document, host);

        let client = match existing {
            Some(client) => client,
            None => match self.create(scope, &document.client_id, name.clone()).await {
                Ok(client) => client,
                Err(CoreError::ClientIdAlreadyExists(_)) => self
                    .find_existing(scope, &document.client_id)
                    .await?
                    .ok_or(CoreError::InternalServerError)?,
                Err(error) => return Err(error),
            },
        };

        if client.registration_source != ClientRegistrationSource::MetadataDocument {
            return Err(ClientMetadataError::ClientIdTaken.into());
        }

        if client.name != name {
            let scoped = self
                .client_repository
                .get_by_id(scope.id(), client.id)
                .await?
                .in_realm(scope)?;
            self.client_repository
                .update_client(&scoped, rename(name))
                .await?;
        }

        self.sync_redirect_uris(client.id, &document.redirect_uris)
            .await
    }

    async fn create(
        &self,
        scope: &RealmScope,
        client_id: &str,
        name: String,
    ) -> Result<Client, CoreError> {
        let client = self
            .client_repository
            .create_client(CreateClientRequest {
                realm_id: scope.id(),
                name,
                client_id: client_id.to_string(),
                secret: None,
                enabled: true,
                protocol: AuthProtocol::OpenIdConnect,
                public_client: true,
                service_account_enabled: false,
                direct_access_grants_enabled: false,
                oauth_device_code_grant_enabled: false,
                token_exchange_enabled: false,
                client_type: ClientType::Public,
                require_pkce: true,
                consent_required: true,
                registration_source: ClientRegistrationSource::MetadataDocument,
            })
            .await?;

        assign_scopes(
            self.scope_repository.as_ref(),
            self.scope_mapping_repository.as_ref(),
            scope.id(),
            client.id,
            None,
        )
        .await?;

        Ok(client)
    }

    async fn sync_redirect_uris(
        &self,
        client_id: uuid::Uuid,
        wanted: &[String],
    ) -> Result<(), CoreError> {
        let current: Vec<RedirectUri> = self
            .redirect_uri_repository
            .get_by_client_id(client_id)
            .await?;

        for stale in current.iter().filter(|uri| !wanted.contains(&uri.value)) {
            self.redirect_uri_repository
                .delete(client_id, stale.id)
                .await?;
        }

        for uri in wanted {
            match current.iter().find(|existing| &existing.value == uri) {
                None => {
                    self.redirect_uri_repository
                        .create_redirect_uri(client_id, uri.clone(), true)
                        .await?;
                }
                Some(existing) if !existing.enabled => {
                    self.redirect_uri_repository
                        .update_enabled(client_id, existing.id, true)
                        .await?;
                }
                Some(_) => {}
            }
        }

        Ok(())
    }

    async fn resolve_document(
        &self,
        scope: &RealmScope,
        client_id: &str,
        redirect_uri: &str,
        allowed_hosts: &[String],
        source_ip: Option<IpAddr>,
    ) -> Result<(), ClientMetadataError> {
        let url = parse_client_id_url(client_id, self.allow_cleartext)?;
        let host = url.host_str().ok_or(ClientMetadataError::InvalidUrl)?;

        ensure_host_allowed(host, allowed_hosts)?;

        let existing = self
            .find_existing(scope, client_id)
            .await
            .map_err(|error| {
                warn!(error = ?error, "could not look up a metadata document client");
                ClientMetadataError::FetchFailed("the client could not be looked up")
            })?;

        if existing.is_none() {
            self.admit_new_client(scope, source_ip).await?;
        }

        let fetched = self.fetcher.fetch(client_id).await?;
        validate_document(client_id, &fetched.document, redirect_uri)?;

        self.upsert(scope, host, &fetched.document, existing)
            .await
            .map_err(|error| match error {
                CoreError::InvalidClient => ClientMetadataError::ClientIdTaken,
                other => {
                    warn!(error = ?other, "could not store a metadata document client");
                    ClientMetadataError::FetchFailed("the client could not be stored")
                }
            })
    }
}

impl<F, C, RU, CS, CSM> ClientMetadataResolverImpl<F, C, RU, CS, CSM>
where
    F: ClientMetadataDocumentFetcher,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn admit_new_client(
        &self,
        scope: &RealmScope,
        source_ip: Option<IpAddr>,
    ) -> Result<(), ClientMetadataError> {
        let ip = source_ip.unwrap_or(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
        if !self.rate_limiter.allow(ip) {
            return Err(ClientMetadataError::RateLimited);
        }

        let existing = self
            .client_repository
            .count_by_source(scope.id(), ClientRegistrationSource::MetadataDocument)
            .await
            .map_err(|error| {
                warn!(error = ?error, "could not count metadata document clients");
                ClientMetadataError::FetchFailed("the client could not be looked up")
            })?;

        if existing >= MAX_METADATA_DOCUMENT_CLIENTS_PER_REALM {
            return Err(ClientMetadataError::RealmLimitReached);
        }

        Ok(())
    }
}

fn display_name(document: &ClientMetadataDocument, host: &str) -> String {
    document
        .client_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(host)
        .chars()
        .take(MAX_CLIENT_NAME_CHARS)
        .collect()
}

fn rename(name: String) -> UpdateClientRequest {
    UpdateClientRequest {
        name: Some(name),
        client_id: None,
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
    }
}

impl<F, C, RU, CS, CSM> ClientMetadataResolver for ClientMetadataResolverImpl<F, C, RU, CS, CSM>
where
    F: ClientMetadataDocumentFetcher,
    C: ClientRepository,
    RU: RedirectUriRepository,
    CS: ClientScopeRepository,
    CSM: ClientScopeMappingRepository,
{
    async fn resolve(
        &self,
        realm: &RealmScope,
        client_id: &str,
        redirect_uri: &str,
        source_ip: Option<IpAddr>,
    ) -> Result<(), CoreError> {
        let Some(settings) = realm.realm().settings.as_ref() else {
            return Ok(());
        };

        if !settings.cimd_enabled || !is_metadata_client_id(client_id, self.allow_cleartext) {
            return Ok(());
        }

        self.resolve_document(
            realm,
            client_id,
            redirect_uri,
            &settings.cimd_allowed_hosts,
            source_ip,
        )
        .await
        .map_err(|error| {
            warn!(%client_id, %error, "refused a metadata document client");
            CoreError::from(error)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::domain::aegis::mocks::{
        MockClientScopeMappingRepository, MockClientScopeRepository,
    };
    use crate::domain::client::ports::{MockClientRepository, MockRedirectUriRepository};
    use crate::domain::client_metadata::entities::FetchedClientMetadata;
    use crate::domain::client_metadata::ports::MockClientMetadataDocumentFetcher;
    use crate::domain::common::services::tests::create_test_realm_with_name;
    use crate::domain::realm::entities::{Realm, RealmSetting, Unscoped};

    fn code(result: Result<(), CoreError>) -> &'static str {
        match result {
            Ok(()) => "ok",
            Err(CoreError::InvalidClient) => "invalid_client",
            Err(CoreError::InvalidRedirectUri) => "invalid_redirect_uri",
            Err(_) => "other",
        }
    }

    const URL: &str = "https://app.example/client.json";
    const CALLBACK: &str = "https://app.example/callback";
    const IP: Option<IpAddr> = Some(IpAddr::V4(std::net::Ipv4Addr::new(198, 51, 100, 7)));

    type Resolver = ClientMetadataResolverImpl<
        MockClientMetadataDocumentFetcher,
        MockClientRepository,
        MockRedirectUriRepository,
        MockClientScopeRepository,
        MockClientScopeMappingRepository,
    >;

    fn scope(configure: impl FnOnce(&mut RealmSetting)) -> RealmScope {
        let mut realm: Realm = create_test_realm_with_name("acme");
        let mut settings = RealmSetting::new(realm.id, None);
        configure(&mut settings);
        realm.settings = Some(settings);
        RealmScope::from_realm(realm)
    }

    fn enabled() -> RealmScope {
        scope(|settings| settings.cimd_enabled = true)
    }

    fn document(client_id: &str, redirect_uris: &[&str]) -> FetchedClientMetadata {
        FetchedClientMetadata {
            document: ClientMetadataDocument {
                client_id: client_id.to_string(),
                client_name: Some("App".to_string()),
                redirect_uris: redirect_uris.iter().map(|uri| uri.to_string()).collect(),
                token_endpoint_auth_method: Some("none".to_string()),
            },
            max_age: Duration::from_secs(60),
        }
    }

    fn fetcher_returning(fetched: FetchedClientMetadata) -> MockClientMetadataDocumentFetcher {
        let mut fetcher = MockClientMetadataDocumentFetcher::new();
        fetcher.expect_fetch().times(1).returning(move |_| {
            let fetched = fetched.clone();
            Box::pin(async move { Ok(fetched) })
        });
        fetcher
    }

    fn resolver(
        fetcher: MockClientMetadataDocumentFetcher,
        clients: MockClientRepository,
        redirects: MockRedirectUriRepository,
    ) -> Resolver {
        resolver_limited(fetcher, clients, redirects, IpRateLimiter::per_minute(100))
    }

    fn resolver_limited(
        fetcher: MockClientMetadataDocumentFetcher,
        clients: MockClientRepository,
        redirects: MockRedirectUriRepository,
        limiter: IpRateLimiter,
    ) -> Resolver {
        let mut scopes = MockClientScopeRepository::new();
        scopes
            .expect_find_by_realm_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        ClientMetadataResolverImpl::new(
            Arc::new(fetcher),
            Arc::new(clients),
            Arc::new(redirects),
            Arc::new(scopes),
            Arc::new(MockClientScopeMappingRepository::new()),
            Arc::new(limiter),
            false,
        )
    }

    fn missing_client() -> MockClientRepository {
        let mut clients = MockClientRepository::new();
        clients
            .expect_get_by_client_id()
            .returning(|_, _| Box::pin(async { Err(CoreError::NotFound) }));
        clients
            .expect_count_by_source()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        clients
    }

    #[tokio::test]
    async fn nothing_happens_when_the_realm_has_not_enabled_documents() {
        let resolver = resolver(
            MockClientMetadataDocumentFetcher::new(),
            MockClientRepository::new(),
            MockRedirectUriRepository::new(),
        );

        let result = resolver.resolve(&scope(|_| {}), URL, CALLBACK, IP).await;

        assert_eq!(code(result), "ok");
    }

    #[tokio::test]
    async fn a_plain_client_id_is_left_to_the_normal_lookup() {
        let resolver = resolver(
            MockClientMetadataDocumentFetcher::new(),
            MockClientRepository::new(),
            MockRedirectUriRepository::new(),
        );

        assert_eq!(
            code(
                resolver
                    .resolve(&enabled(), "my-client", CALLBACK, IP)
                    .await
            ),
            "ok"
        );
    }

    #[tokio::test]
    async fn a_host_outside_the_allowlist_is_refused_before_any_fetch() {
        let resolver = resolver(
            MockClientMetadataDocumentFetcher::new(),
            MockClientRepository::new(),
            MockRedirectUriRepository::new(),
        );
        let realm = scope(|settings| {
            settings.cimd_enabled = true;
            settings.cimd_allowed_hosts = vec!["trusted.example".to_string()];
        });

        assert_eq!(
            code(resolver.resolve(&realm, URL, CALLBACK, IP).await),
            "invalid_client"
        );
    }

    #[tokio::test]
    async fn a_document_whose_client_id_differs_is_refused() {
        let resolver = resolver(
            fetcher_returning(document("https://other.example/c.json", &[CALLBACK])),
            missing_client(),
            MockRedirectUriRepository::new(),
        );

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "invalid_client"
        );
    }

    #[tokio::test]
    async fn an_unlisted_redirect_uri_is_refused() {
        let resolver = resolver(
            fetcher_returning(document(URL, &[CALLBACK])),
            missing_client(),
            MockRedirectUriRepository::new(),
        );

        assert_eq!(
            code(
                resolver
                    .resolve(&enabled(), URL, "https://app.example/other", IP)
                    .await
            ),
            "invalid_redirect_uri"
        );
    }

    #[tokio::test]
    async fn a_valid_document_creates_a_public_consenting_pkce_client() {
        let mut clients = missing_client();
        clients
            .expect_create_client()
            .times(1)
            .withf(|request| {
                request.client_id == URL
                    && request.name == "App"
                    && request.public_client
                    && request.require_pkce
                    && request.consent_required
                    && !request.token_exchange_enabled
                    && !request.service_account_enabled
                    && !request.direct_access_grants_enabled
                    && request.secret.is_none()
                    && request.registration_source == ClientRegistrationSource::MetadataDocument
            })
            .returning(|request| {
                Box::pin(async move {
                    let mut client =
                        Client::from_realm_and_client_id(request.realm_id, request.client_id);
                    client.name = request.name;
                    client.registration_source = request.registration_source;
                    Ok(client)
                })
            });

        let mut redirects = MockRedirectUriRepository::new();
        redirects
            .expect_get_by_client_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        redirects
            .expect_create_redirect_uri()
            .times(1)
            .withf(|_, value, enabled| value == CALLBACK && *enabled)
            .returning(|client_id, value, enabled| {
                Box::pin(async move {
                    Ok(RedirectUri {
                        id: uuid::Uuid::new_v4(),
                        client_id,
                        value,
                        enabled,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
                })
            });

        let resolver = resolver(
            fetcher_returning(document(URL, &[CALLBACK])),
            clients,
            redirects,
        );

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "ok"
        );
    }

    #[tokio::test]
    async fn a_url_already_taken_by_an_admin_client_is_refused() {
        let mut clients = MockClientRepository::new();
        clients
            .expect_get_by_client_id()
            .returning(|client_id, realm_id| {
                Box::pin(async move {
                    Ok(Unscoped::new(Client::from_realm_and_client_id(
                        realm_id, client_id,
                    )))
                })
            });

        let resolver = resolver(
            fetcher_returning(document(URL, &[CALLBACK])),
            clients,
            MockRedirectUriRepository::new(),
        );

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "invalid_client"
        );
    }

    #[tokio::test]
    async fn a_realm_at_its_metadata_client_cap_refuses_new_urls_before_any_fetch() {
        let mut clients = MockClientRepository::new();
        clients
            .expect_get_by_client_id()
            .returning(|_, _| Box::pin(async { Err(CoreError::NotFound) }));
        clients.expect_count_by_source().returning(|_, source| {
            assert_eq!(source, ClientRegistrationSource::MetadataDocument);
            Box::pin(async { Ok(MAX_METADATA_DOCUMENT_CLIENTS_PER_REALM) })
        });

        let resolver = resolver(
            MockClientMetadataDocumentFetcher::new(),
            clients,
            MockRedirectUriRepository::new(),
        );

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "invalid_client"
        );
    }

    #[tokio::test]
    async fn new_urls_from_one_address_are_rate_limited_before_any_fetch() {
        let resolver = resolver_limited(
            MockClientMetadataDocumentFetcher::new(),
            missing_client(),
            MockRedirectUriRepository::new(),
            IpRateLimiter::per_minute(1),
        );
        assert!(resolver.rate_limiter.allow(IP.expect("ip")));

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "invalid_client"
        );
    }

    #[tokio::test]
    async fn a_known_client_is_not_charged_to_the_new_client_limiter() {
        let mut clients = MockClientRepository::new();
        clients
            .expect_get_by_client_id()
            .returning(|client_id, realm_id| {
                Box::pin(async move {
                    let mut client = Client::from_realm_and_client_id(realm_id, client_id);
                    client.registration_source = ClientRegistrationSource::MetadataDocument;
                    client.name = "App".to_string();
                    Ok(Unscoped::new(client))
                })
            });
        let mut redirects = MockRedirectUriRepository::new();
        redirects
            .expect_get_by_client_id()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        redirects
            .expect_create_redirect_uri()
            .returning(|client_id, value, enabled| {
                Box::pin(async move {
                    Ok(RedirectUri {
                        id: uuid::Uuid::new_v4(),
                        client_id,
                        value,
                        enabled,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    })
                })
            });

        let resolver = resolver_limited(
            fetcher_returning(document(URL, &[CALLBACK])),
            clients,
            redirects,
            IpRateLimiter::per_minute(1),
        );
        assert!(resolver.rate_limiter.allow(IP.expect("ip")));

        assert_eq!(
            code(resolver.resolve(&enabled(), URL, CALLBACK, IP).await),
            "ok"
        );
    }
}
