use std::str::FromStr;
use std::sync::Arc;

use chrono::Utc;
use ferriskey_saml::authn::{AbsoluteUri, AuthnRequest};
use ferriskey_saml::response::render_signed_response;
use ferriskey_security::jwt::ports::KeyStoreRepository;
use tracing::{error, warn};

use crate::domain::authentication::entities::{
    AuthOutput, AuthProtocol, AuthSession, AuthSessionParams,
};
use crate::domain::authentication::ports::AuthSessionRepository;
use crate::domain::client::entities::saml::SpEntityId;
use crate::domain::client::ports::ClientRepository;
use crate::domain::common::entities::app_errors::CoreError;
use crate::domain::realm::ports::RealmRepository;
use crate::domain::saml::entities::{
    AssertionBlueprint, FinishSsoInput, SamlAssertionDelivery, SamlSsoError, StartSsoInput,
    build_assertion_attributes, build_response_descriptor, derive_name_id, generate_element_id,
    idp_entity_id, needs_user_attributes, record_authn_request_id, recorded_authn_request_id,
    resolve_assertion_consumer_service_url, sso_continue_url,
};
use crate::domain::saml::ports::{SamlService, SamlServiceProviderRepository};
use crate::domain::user::ports::{UserAttributeRepository, UserRepository};

#[derive(Clone, Debug)]
pub struct SamlServiceImpl<R, C, S, U, UA, A, K>
where
    R: RealmRepository,
    C: ClientRepository,
    S: SamlServiceProviderRepository,
    U: UserRepository,
    UA: UserAttributeRepository,
    A: AuthSessionRepository,
    K: KeyStoreRepository,
{
    pub(crate) realm_repository: Arc<R>,
    pub(crate) client_repository: Arc<C>,
    pub(crate) service_provider_repository: Arc<S>,
    pub(crate) user_repository: Arc<U>,
    pub(crate) user_attribute_repository: Arc<UA>,
    pub(crate) auth_session_repository: Arc<A>,
    pub(crate) keystore_repository: Arc<K>,
}

impl<R, C, S, U, UA, A, K> SamlServiceImpl<R, C, S, U, UA, A, K>
where
    R: RealmRepository,
    C: ClientRepository,
    S: SamlServiceProviderRepository,
    U: UserRepository,
    UA: UserAttributeRepository,
    A: AuthSessionRepository,
    K: KeyStoreRepository,
{
    pub fn new(
        realm_repository: Arc<R>,
        client_repository: Arc<C>,
        service_provider_repository: Arc<S>,
        user_repository: Arc<U>,
        user_attribute_repository: Arc<UA>,
        auth_session_repository: Arc<A>,
        keystore_repository: Arc<K>,
    ) -> Self {
        Self {
            realm_repository,
            client_repository,
            service_provider_repository,
            user_repository,
            user_attribute_repository,
            auth_session_repository,
            keystore_repository,
        }
    }
}

pub fn format_login_url(client_id: &str, redirect_uri: &str, relay_state: Option<&str>) -> String {
    format!(
        "?client_id={}&redirect_uri={}&state={}",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(relay_state.unwrap_or_default()),
    )
}

impl<R, C, S, U, UA, A, K> SamlService for SamlServiceImpl<R, C, S, U, UA, A, K>
where
    R: RealmRepository,
    C: ClientRepository,
    S: SamlServiceProviderRepository,
    U: UserRepository,
    UA: UserAttributeRepository,
    A: AuthSessionRepository,
    K: KeyStoreRepository,
{
    async fn start_sso(&self, input: StartSsoInput) -> Result<AuthOutput, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let request = AuthnRequest::parse(&input.authn_request).map_err(SamlSsoError::from)?;

        let issuer = SpEntityId::from_str(request.issuer.as_str())
            .map_err(|_| SamlSsoError::UnknownServiceProvider(request.issuer.to_string()))?;

        let config = self
            .service_provider_repository
            .get_by_entity_id(realm.id, issuer.clone())
            .await?
            .ok_or_else(|| SamlSsoError::UnknownServiceProvider(issuer.to_string()))?;

        let client = self
            .client_repository
            .get_by_id(realm.id, config.client_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.enabled {
            warn!(
                client_id = %client.client_id,
                "rejecting a saml authn request: the service provider is disabled"
            );

            return Err(CoreError::InvalidClient);
        }

        let protocol = client.protocol.parse::<AuthProtocol>().map_err(|reason| {
            warn!(
                client_id = %client.client_id,
                %reason,
                "rejecting a saml authn request for a client whose protocol is unknown"
            );

            CoreError::InvalidClient
        })?;

        if protocol != AuthProtocol::Saml {
            warn!(
                client_id = %client.client_id,
                %protocol,
                "rejecting a saml authn request: this endpoint only serves saml clients"
            );

            return Err(CoreError::InvalidClient);
        }

        resolve_assertion_consumer_service_url(
            request
                .assertion_consumer_service_url
                .as_ref()
                .map(AbsoluteUri::as_str),
            &config.acs_url,
        )
        .map_err(|rejection| {
            warn!(
                client_id = %client.client_id,
                %rejection,
                "rejecting a saml authn request: the assertion would leave for an unregistered address"
            );

            rejection
        })?;

        let redirect_uri = sso_continue_url(&input.public_base_url, &realm.name);

        let session = self
            .auth_session_repository
            .create(&AuthSession::new(AuthSessionParams {
                realm_id: realm.id,
                client_id: client.id,
                protocol: AuthProtocol::Saml,
                redirect_uri: redirect_uri.clone(),
                response_type: None,
                scope: None,
                state: input.relay_state.clone(),
                nonce: record_authn_request_id(&request.id),
                user_id: None,
                code: None,
                authenticated: false,
                webauthn_challenge: None,
                webauthn_challenge_issued_at: None,
                compass_flow_id: None,
                code_challenge: None,
                code_challenge_method: None,
            }))
            .await
            .map_err(|_| CoreError::SessionCreateError)?;

        Ok(AuthOutput {
            login_url: format_login_url(
                &client.client_id,
                &redirect_uri,
                input.relay_state.as_deref(),
            ),
            session,
        })
    }

    async fn idp_signing_certificate(&self, realm_name: String) -> Result<String, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let keypair = self
            .keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        keypair
            .certificate_base64_der()
            .map_err(|reason| CoreError::InvalidKey(reason.to_string()))
    }

    async fn finish_sso(&self, input: FinishSsoInput) -> Result<SamlAssertionDelivery, CoreError> {
        let realm = self
            .realm_repository
            .get_by_name(&input.realm_name)
            .await?
            .ok_or(CoreError::InvalidRealm)?;

        let auth_session = self
            .auth_session_repository
            .get_by_code(input.authorization_code)
            .await
            .map_err(|_| CoreError::MissingAuthorizationCode)?
            .ok_or(CoreError::InvalidAuthorizationCode)?;

        if auth_session.protocol != AuthProtocol::Saml {
            warn!(
                auth_session_id = %auth_session.id,
                protocol = %auth_session.protocol,
                "rejecting a saml continuation: the code was minted for another protocol"
            );

            return Err(SamlSsoError::NotASamlAuthentication.into());
        }

        if auth_session.realm_id != realm.id {
            warn!(
                session_realm = ?auth_session.realm_id,
                request_realm = ?realm.id,
                "rejecting a saml continuation: the code was issued for a different realm"
            );

            return Err(CoreError::InvalidAuthorizationCode);
        }

        if auth_session.authenticated {
            warn!(
                auth_session_id = %auth_session.id,
                "rejecting a saml continuation: the code has already been redeemed"
            );

            return Err(CoreError::InvalidAuthorizationCode);
        }

        if Utc::now() >= auth_session.expires_at {
            return Err(CoreError::SessionExpired);
        }

        let in_response_to = recorded_authn_request_id(&auth_session)?;
        let user_id = auth_session
            .user_id
            .ok_or(CoreError::InvalidAuthorizationCode)?;

        let client = self
            .client_repository
            .get_by_id(realm.id, auth_session.client_id)
            .await
            .map_err(|_| CoreError::InvalidClient)?;

        if !client.enabled {
            return Err(CoreError::InvalidClient);
        }

        let config = self
            .service_provider_repository
            .get_by_client_id(client.id)
            .await?
            .ok_or(CoreError::SamlConfigNotFound)?;

        let mappers = self
            .service_provider_repository
            .get_attribute_mappers(client.id)
            .await?;

        let user = self.user_repository.get_by_id(user_id).await?;

        if !user.enabled {
            return Err(CoreError::UserDisabled);
        }

        let user_attributes = if needs_user_attributes(&mappers) {
            self.user_attribute_repository
                .list_by_user_id(user.id)
                .await?
        } else {
            Vec::new()
        };

        let keypair = self
            .keystore_repository
            .get_or_generate_key(realm.id)
            .await
            .map_err(|_| CoreError::RealmKeyNotFound)?;

        let certificate = keypair
            .certificate_base64_der()
            .map_err(|reason| CoreError::InvalidKey(reason.to_string()))?;

        let session_index = auth_session.id.to_string();

        let descriptor = build_response_descriptor(AssertionBlueprint {
            response_id: generate_element_id()?,
            assertion_id: generate_element_id()?,
            in_response_to,
            idp_entity_id: idp_entity_id(&input.public_base_url, &realm.name),
            sp_entity_id: config.sp_entity_id.clone(),
            acs_url: config.acs_url.clone(),
            name_id: derive_name_id(config.name_id_format, &user, &session_index)?,
            session_index,
            attributes: build_assertion_attributes(&mappers, &user, &user_attributes),
            issued_at: Utc::now(),
        })?;

        self.auth_session_repository
            .update_authenticated(auth_session.id, true)
            .await
            .map_err(|reason| {
                error!(
                    auth_session_id = %auth_session.id,
                    %reason,
                    "refusing to issue a saml assertion the authorization code could not be spent for"
                );

                CoreError::SessionNotFound
            })?;

        let signed_response =
            render_signed_response(&descriptor, &keypair.private_key, &certificate).map_err(
                |reason| {
                    error!(
                        client_id = %client.client_id,
                        %reason,
                        "failed to sign a saml response"
                    );

                    CoreError::InternalServerError
                },
            )?;

        Ok(SamlAssertionDelivery {
            acs_url: config.acs_url,
            signed_response,
            relay_state: auth_session.state,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    use std::sync::OnceLock;

    use ferriskey_security::SecurityError;
    use mockall::predicate::eq;
    use uuid::Uuid;

    use crate::domain::authentication::entities::AuthenticationError;
    use crate::domain::client::entities::saml::{
        ClientSamlConfig, NameIdFormat, SamlAttributeSource, SamlConfigSettings,
    };
    use crate::domain::client::entities::{Client, ClientType, MaintenanceSessionStrategy};
    use crate::domain::client::ports::MockClientRepository;
    use crate::domain::jwt::entities::JwtKeyPair;
    use crate::domain::realm::entities::{Realm, RealmId};
    use crate::domain::realm::ports::MockRealmRepository;
    use crate::domain::saml::entities::fixtures::{acs_url, sp_entity_id};
    use crate::domain::saml::entities::{
        SamlAssertionDelivery, needs_user_attributes, record_authn_request_id,
    };
    use crate::domain::saml::ports::MockSamlServiceProviderRepository;
    use crate::domain::user::entities::User;
    use crate::domain::user::ports::{MockUserAttributeRepository, MockUserRepository};

    use crate::domain::authentication::ports::MockAuthSessionRepository;

    const REALM_NAME: &str = "master";
    const PUBLIC_BASE_URL: &str = "https://auth.example.com";
    const SP_ENTITY_ID: &str = "https://chat.example.com/saml/sp/1";
    const REGISTERED_ACS: &str = "https://chat.example.com/omniauth/saml/callback?account_id=1";

    struct FixedKeyStore {
        keypair: JwtKeyPair,
    }

    impl KeyStoreRepository for FixedKeyStore {
        async fn get_or_generate_key(
            &self,
            _realm_id: RealmId,
        ) -> Result<JwtKeyPair, SecurityError> {
            Ok(self.keypair.clone())
        }
    }

    fn signing_material() -> &'static (String, String, String) {
        static MATERIAL: OnceLock<(String, String, String)> = OnceLock::new();

        MATERIAL.get_or_init(|| {
            let (private_pem, public_pem) =
                JwtKeyPair::generate().expect("the test suite needs one rsa key pair");
            let certificate = JwtKeyPair::self_signed_certificate(&private_pem, "auth.example.com")
                .expect("the test suite needs one self-signed certificate");

            (private_pem, public_pem, certificate)
        })
    }

    fn key_store() -> FixedKeyStore {
        let (private_pem, public_pem, certificate) = signing_material();

        FixedKeyStore {
            keypair: JwtKeyPair::from_pem(
                private_pem,
                public_pem,
                certificate,
                Uuid::new_v4(),
                Uuid::new_v4(),
            )
            .expect("the generated pem must load"),
        }
    }

    fn realm() -> Realm {
        Realm {
            id: RealmId::default(),
            name: REALM_NAME.to_string(),
            display_name: None,
            settings: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn saml_client(realm_id: RealmId) -> Client {
        Client {
            id: Uuid::new_v4(),
            enabled: true,
            client_id: "chatwoot".to_string(),
            secret: None,
            realm_id,
            protocol: AuthProtocol::Saml.as_str().to_string(),
            public_client: true,
            service_account_enabled: false,
            direct_access_grants_enabled: false,
            oauth_device_code_grant_enabled: false,
            require_pkce: false,
            client_type: ClientType::Public,
            name: "Chatwoot".to_string(),
            redirect_uris: None,
            access_token_lifetime: None,
            refresh_token_lifetime: None,
            id_token_lifetime: None,
            temporary_token_lifetime: None,
            maintenance_enabled: false,
            maintenance_reason: None,
            maintenance_session_strategy: MaintenanceSessionStrategy::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn saml_config(
        realm_id: RealmId,
        client_id: Uuid,
        name_id_format: NameIdFormat,
    ) -> ClientSamlConfig {
        ClientSamlConfig::new(
            realm_id,
            client_id,
            SamlConfigSettings {
                sp_entity_id: sp_entity_id(SP_ENTITY_ID),
                acs_url: acs_url(REGISTERED_ACS),
                name_id_format,
                sign_assertions: true,
                sign_documents: false,
                want_authn_requests_signed: false,
            },
        )
    }

    fn authn_request(acs: Option<&str>) -> String {
        let acs_attribute = acs
            .map(|value| format!(r#" AssertionConsumerServiceURL="{value}""#))
            .unwrap_or_default();

        format!(
            r#"<samlp:AuthnRequest xmlns:samlp="urn:oasis:names:tc:SAML:2.0:protocol" xmlns:saml="urn:oasis:names:tc:SAML:2.0:assertion" ID="_324d8ca747d9c07921d2abe9df447832" Version="2.0" IssueInstant="2026-08-25T18:51:18.334Z" Destination="https://auth.example.com/realms/master/protocol/saml"{acs_attribute}><saml:Issuer>{SP_ENTITY_ID}</saml:Issuer></samlp:AuthnRequest>"#
        )
    }

    pub(crate) fn saml_auth_session(recorded_request_id: Option<String>) -> AuthSession {
        AuthSession {
            id: Uuid::new_v4(),
            realm_id: RealmId::default(),
            client_id: Uuid::new_v4(),
            protocol: AuthProtocol::Saml,
            redirect_uri: format!("{PUBLIC_BASE_URL}/realms/{REALM_NAME}/protocol/saml/continue"),
            response_type: None,
            scope: None,
            state: None,
            nonce: recorded_request_id,
            user_id: Some(Uuid::new_v4()),
            code: Some("AUTH_CODE".to_string()),
            authenticated: false,
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            webauthn_challenge: None,
            webauthn_challenge_issued_at: None,
            compass_flow_id: None,
            code_challenge: None,
            code_challenge_method: None,
        }
    }

    fn authenticated_user(realm_id: RealmId, user_id: Uuid) -> User {
        let mut user = crate::domain::saml::entities::fixtures::user(realm_id);
        user.id = user_id;
        user
    }

    struct Harness {
        realm: Realm,
        client: Client,
        realm_repository: MockRealmRepository,
        client_repository: MockClientRepository,
        service_provider_repository: MockSamlServiceProviderRepository,
        user_repository: MockUserRepository,
        user_attribute_repository: MockUserAttributeRepository,
        auth_session_repository: MockAuthSessionRepository,
    }

    impl Harness {
        fn new() -> Self {
            let realm = realm();
            let client = saml_client(realm.id);

            Self {
                realm,
                client,
                realm_repository: MockRealmRepository::new(),
                client_repository: MockClientRepository::new(),
                service_provider_repository: MockSamlServiceProviderRepository::new(),
                user_repository: MockUserRepository::new(),
                user_attribute_repository: MockUserAttributeRepository::new(),
                auth_session_repository: MockAuthSessionRepository::new(),
            }
        }

        fn with_realm(mut self) -> Self {
            let realm = self.realm.clone();

            self.realm_repository
                .expect_get_by_name()
                .with(eq(REALM_NAME))
                .returning(move |_| {
                    let realm = realm.clone();
                    Box::pin(async move { Ok(Some(realm)) })
                });

            self
        }

        fn with_client(mut self) -> Self {
            let client = self.client.clone();

            self.client_repository
                .expect_get_by_id()
                .returning(move |_, _| {
                    let client = client.clone();
                    Box::pin(async move { Ok(client) })
                });

            self
        }

        fn with_registered_service_provider(mut self, name_id_format: NameIdFormat) -> Self {
            let config = saml_config(self.realm.id, self.client.id, name_id_format);
            let by_entity_id = config.clone();

            self.service_provider_repository
                .expect_get_by_entity_id()
                .returning(move |_, _| {
                    let config = by_entity_id.clone();
                    Box::pin(async move { Ok(Some(config)) })
                });

            self.service_provider_repository
                .expect_get_by_client_id()
                .returning(move |_| {
                    let config = config.clone();
                    Box::pin(async move { Ok(Some(config)) })
                });

            self
        }

        fn build(
            self,
        ) -> SamlServiceImpl<
            MockRealmRepository,
            MockClientRepository,
            MockSamlServiceProviderRepository,
            MockUserRepository,
            MockUserAttributeRepository,
            MockAuthSessionRepository,
            FixedKeyStore,
        > {
            SamlServiceImpl::new(
                Arc::new(self.realm_repository),
                Arc::new(self.client_repository),
                Arc::new(self.service_provider_repository),
                Arc::new(self.user_repository),
                Arc::new(self.user_attribute_repository),
                Arc::new(self.auth_session_repository),
                Arc::new(key_store()),
            )
        }
    }

    fn start_input(relay_state: Option<&str>, acs: Option<&str>) -> StartSsoInput {
        StartSsoInput {
            realm_name: REALM_NAME.to_string(),
            authn_request: authn_request(acs),
            relay_state: relay_state.map(str::to_string),
            public_base_url: PUBLIC_BASE_URL.to_string(),
        }
    }

    fn refusal<T>(result: Result<T, CoreError>, context: &str) -> CoreError {
        match result {
            Ok(_) => panic!("{context}"),
            Err(rejection) => rejection,
        }
    }

    fn finish_input() -> FinishSsoInput {
        FinishSsoInput {
            realm_name: REALM_NAME.to_string(),
            authorization_code: "AUTH_CODE".to_string(),
            public_base_url: PUBLIC_BASE_URL.to_string(),
        }
    }

    #[tokio::test]
    async fn a_started_sso_parks_the_login_on_our_own_continue_endpoint() {
        let mut harness = Harness::new()
            .with_realm()
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        harness
            .auth_session_repository
            .expect_create()
            .returning(|session| {
                let session = session.clone();
                Box::pin(async move { Ok(session) })
            });

        let output = harness
            .build()
            .start_sso(start_input(Some("relay-me"), Some(REGISTERED_ACS)))
            .await
            .expect("a registered service provider starts an sso");

        assert_eq!(
            output.session.redirect_uri,
            "https://auth.example.com/realms/master/protocol/saml/continue"
        );
        assert_eq!(output.session.protocol, AuthProtocol::Saml);
        assert_eq!(output.session.state.as_deref(), Some("relay-me"));
        assert_eq!(output.session.response_type, None);
        assert_eq!(output.session.scope, None);
    }

    #[tokio::test]
    async fn a_started_sso_records_the_inbound_request_id_so_it_can_be_echoed_back() {
        let mut harness = Harness::new()
            .with_realm()
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        harness
            .auth_session_repository
            .expect_create()
            .returning(|session| {
                let session = session.clone();
                Box::pin(async move { Ok(session) })
            });

        let output = harness
            .build()
            .start_sso(start_input(None, None))
            .await
            .expect("a registered service provider starts an sso");

        assert_eq!(
            crate::domain::saml::entities::recorded_authn_request_id(&output.session)
                .expect("the inbound request id must survive the round trip")
                .as_str(),
            "_324d8ca747d9c07921d2abe9df447832"
        );
    }

    #[tokio::test]
    async fn a_started_sso_accepts_an_absent_relay_state_because_saml_makes_it_optional() {
        let mut harness = Harness::new()
            .with_realm()
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        harness
            .auth_session_repository
            .expect_create()
            .returning(|session| {
                let session = session.clone();
                Box::pin(async move { Ok(session) })
            });

        let output = harness
            .build()
            .start_sso(start_input(None, None))
            .await
            .expect("a service provider may omit the relay state");

        assert_eq!(output.session.state, None);
    }

    #[tokio::test]
    async fn a_started_sso_hands_the_login_page_the_client_and_the_relay_state() {
        let mut harness = Harness::new()
            .with_realm()
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        harness
            .auth_session_repository
            .expect_create()
            .returning(|session| {
                let session = session.clone();
                Box::pin(async move { Ok(session) })
            });

        let output = harness
            .build()
            .start_sso(start_input(Some("a b&c"), None))
            .await
            .expect("a registered service provider starts an sso");

        assert_eq!(
            output.login_url,
            "?client_id=chatwoot&redirect_uri=https%3A%2F%2Fauth.example.com%2Frealms%2Fmaster%2Fprotocol%2Fsaml%2Fcontinue&state=a%20b%26c"
        );
    }

    #[tokio::test]
    async fn an_authn_request_from_an_unregistered_issuer_starts_nothing() {
        let mut harness = Harness::new().with_realm();

        harness
            .service_provider_repository
            .expect_get_by_entity_id()
            .returning(|_, _| Box::pin(async move { Ok(None) }));

        let rejection = refusal(
            harness.build().start_sso(start_input(None, None)).await,
            "an unknown issuer cannot start an sso",
        );

        assert!(matches!(rejection, CoreError::SamlConfigNotFound));
    }

    #[tokio::test]
    async fn an_authn_request_naming_an_attacker_chosen_acs_url_starts_nothing() {
        let harness = Harness::new()
            .with_realm()
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        let rejection = refusal(
            harness
                .build()
                .start_sso(start_input(None, Some("https://evil.example.com/steal")))
                .await,
            "an unregistered acs url cannot start an sso",
        );

        assert!(
            matches!(rejection, CoreError::InvalidRedirectUri),
            "the session must never be created, or the assertion would later be delivered"
        );
    }

    #[tokio::test]
    async fn an_authn_request_for_a_disabled_service_provider_starts_nothing() {
        let mut harness = Harness::new().with_realm();
        harness.client.enabled = false;
        let harness = harness
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        let rejection = refusal(
            harness.build().start_sso(start_input(None, None)).await,
            "a disabled client cannot start an sso",
        );

        assert!(matches!(rejection, CoreError::InvalidClient));
    }

    #[tokio::test]
    async fn an_authn_request_for_an_openid_connect_client_starts_nothing() {
        let mut harness = Harness::new().with_realm();
        harness.client.protocol = AuthProtocol::OpenIdConnect.as_str().to_string();
        let harness = harness
            .with_client()
            .with_registered_service_provider(NameIdFormat::EmailAddress);

        let rejection = refusal(
            harness.build().start_sso(start_input(None, None)).await,
            "an openid-connect client cannot start an sso",
        );

        assert!(matches!(rejection, CoreError::InvalidClient));
    }

    #[tokio::test]
    async fn a_malformed_authn_request_starts_nothing() {
        let harness = Harness::new().with_realm();

        let mut input = start_input(None, None);
        input.authn_request = "<not-a-saml-request/>".to_string();

        let rejection = refusal(
            harness.build().start_sso(input).await,
            "a malformed request cannot start an sso",
        );

        assert!(matches!(rejection, CoreError::InvalidRequest));
    }

    struct FinishHarness {
        harness: Harness,
        session: AuthSession,
        user: User,
    }

    impl FinishHarness {
        fn new(
            name_id_format: NameIdFormat,
            mappers: Vec<crate::domain::client::entities::saml::SamlAttributeMapper>,
        ) -> Self {
            let mut harness = Harness::new()
                .with_realm()
                .with_client()
                .with_registered_service_provider(name_id_format);

            let mut session = saml_auth_session(record_authn_request_id(
                &crate::domain::saml::entities::fixtures::request_id(
                    "_324d8ca747d9c07921d2abe9df447832",
                ),
            ));
            session.realm_id = harness.realm.id;
            session.client_id = harness.client.id;

            let user = authenticated_user(
                harness.realm.id,
                session.user_id.expect("the fixture session carries a user"),
            );

            let needs_attributes = needs_user_attributes(&mappers);
            harness
                .service_provider_repository
                .expect_get_attribute_mappers()
                .returning(move |_| {
                    let mappers = mappers.clone();
                    Box::pin(async move { Ok(mappers) })
                });

            if needs_attributes {
                harness
                    .user_attribute_repository
                    .expect_list_by_user_id()
                    .returning(|_| Box::pin(async move { Ok(Vec::new()) }));
            }

            Self {
                harness,
                session,
                user,
            }
        }

        fn expecting_consumption(mut self) -> Self {
            self.harness
                .auth_session_repository
                .expect_update_authenticated()
                .with(eq(self.session.id), eq(true))
                .times(1)
                .returning(|_, _| Box::pin(async move { Ok(()) }));

            self
        }

        async fn finish(mut self) -> Result<SamlAssertionDelivery, CoreError> {
            let session = self.session.clone();
            self.harness
                .auth_session_repository
                .expect_get_by_code()
                .returning(move |_| {
                    let session = session.clone();
                    Box::pin(async move { Ok(Some(session)) })
                });

            let user = self.user.clone();
            self.harness
                .user_repository
                .expect_get_by_id()
                .returning(move |_| {
                    let user = user.clone();
                    Box::pin(async move { Ok(user) })
                });

            self.harness.build().finish_sso(finish_input()).await
        }
    }

    #[tokio::test]
    async fn a_finished_sso_answers_the_inbound_request_in_both_places_that_bind_the_assertion() {
        let delivery = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new())
            .expecting_consumption()
            .finish()
            .await
            .expect("a consumable saml code yields an assertion");

        assert_eq!(
            delivery
                .signed_response
                .matches(r#"InResponseTo="_324d8ca747d9c07921d2abe9df447832""#)
                .count(),
            2,
            "the envelope and the subject confirmation must both name the request they answer"
        );
    }

    #[tokio::test]
    async fn a_finished_sso_targets_the_registered_acs_url_and_the_registered_audience() {
        let delivery = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new())
            .expecting_consumption()
            .finish()
            .await
            .expect("a consumable saml code yields an assertion");

        assert_eq!(delivery.acs_url, acs_url(REGISTERED_ACS));
        assert!(
            delivery
                .signed_response
                .contains(&format!(r#"Destination="{REGISTERED_ACS}""#))
        );
        assert!(
            delivery
                .signed_response
                .contains(&format!("<saml:Audience>{SP_ENTITY_ID}</saml:Audience>"))
        );
    }

    #[tokio::test]
    async fn a_finished_sso_is_signed_and_carries_the_realm_certificate() {
        let delivery = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new())
            .expecting_consumption()
            .finish()
            .await
            .expect("a consumable saml code yields an assertion");

        assert!(delivery.signed_response.contains("<ds:Signature"));
        assert!(delivery.signed_response.contains("<ds:X509Certificate>"));
    }

    #[tokio::test]
    async fn a_finished_sso_carries_the_mapped_attributes() {
        let delivery = FinishHarness::new(
            NameIdFormat::EmailAddress,
            vec![
                crate::domain::saml::entities::fixtures::mapper(
                    "email",
                    SamlAttributeSource::Email,
                ),
                crate::domain::saml::entities::fixtures::mapper(
                    "first_name",
                    SamlAttributeSource::FirstName,
                ),
            ],
        )
        .expecting_consumption()
        .finish()
        .await
        .expect("a consumable saml code yields an assertion");

        assert!(delivery.signed_response.contains(r#"Name="email""#));
        assert!(delivery.signed_response.contains(r#"Name="first_name""#));
        assert!(delivery.signed_response.contains("alice@example.com"));
    }

    #[tokio::test]
    async fn a_finished_sso_echoes_the_relay_state_the_service_provider_sent() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.state = Some("relay-me".to_string());

        let delivery = harness
            .expecting_consumption()
            .finish()
            .await
            .expect("a consumable saml code yields an assertion");

        assert_eq!(delivery.relay_state.as_deref(), Some("relay-me"));
    }

    #[tokio::test]
    async fn an_authorization_code_from_an_openid_connect_login_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.protocol = AuthProtocol::OpenIdConnect;

        let rejection = refusal(
            harness.finish().await,
            "an openid-connect code must not be redeemable for an assertion",
        );

        assert!(matches!(rejection, CoreError::InvalidAuthorizationCode));
    }

    #[tokio::test]
    async fn an_already_consumed_authorization_code_yields_no_second_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.authenticated = true;

        let rejection = refusal(
            harness.finish().await,
            "a spent code must not mint a second assertion",
        );

        assert!(matches!(rejection, CoreError::InvalidAuthorizationCode));
    }

    #[tokio::test]
    async fn an_expired_authorization_code_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.expires_at = Utc::now() - chrono::Duration::seconds(1);

        let rejection = refusal(
            harness.finish().await,
            "an expired session must not mint an assertion",
        );

        assert!(matches!(rejection, CoreError::SessionExpired));
    }

    #[tokio::test]
    async fn an_authorization_code_from_another_realm_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.realm_id = RealmId::default();

        let rejection = refusal(
            harness.finish().await,
            "a code from another realm must not be signed with this realm's key",
        );

        assert!(matches!(rejection, CoreError::InvalidAuthorizationCode));
    }

    #[tokio::test]
    async fn an_unknown_authorization_code_yields_no_assertion() {
        let mut harness = Harness::new().with_realm();

        harness
            .auth_session_repository
            .expect_get_by_code()
            .returning(|_| Box::pin(async move { Ok(None) }));

        let rejection = refusal(
            harness.build().finish_sso(finish_input()).await,
            "an unknown code must not mint an assertion",
        );

        assert!(matches!(rejection, CoreError::InvalidAuthorizationCode));
    }

    #[tokio::test]
    async fn a_login_that_never_completed_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.user_id = None;

        let rejection = refusal(
            harness.finish().await,
            "a session with no authenticated user must not mint an assertion",
        );

        assert!(matches!(rejection, CoreError::InvalidAuthorizationCode));
    }

    #[tokio::test]
    async fn a_session_that_recorded_no_request_id_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.session.nonce = None;

        let rejection = refusal(
            harness.finish().await,
            "an assertion that answers nothing could be replayed anywhere",
        );

        assert!(matches!(rejection, CoreError::InvalidSession));
    }

    #[tokio::test]
    async fn a_disabled_user_yields_no_assertion() {
        let mut harness = FinishHarness::new(NameIdFormat::EmailAddress, Vec::new());
        harness.user.enabled = false;

        let rejection = refusal(
            harness.finish().await,
            "a disabled user must not mint an assertion",
        );

        assert!(matches!(rejection, CoreError::UserDisabled));
    }

    #[tokio::test]
    async fn a_failed_lookup_of_the_code_never_reads_as_a_valid_login() {
        let mut harness = Harness::new().with_realm();

        harness
            .auth_session_repository
            .expect_get_by_code()
            .returning(|_| Box::pin(async move { Err(AuthenticationError::InternalServerError) }));

        let rejection = refusal(
            harness.build().finish_sso(finish_input()).await,
            "a lookup failure must not mint an assertion",
        );

        assert!(matches!(rejection, CoreError::MissingAuthorizationCode));
    }
}
